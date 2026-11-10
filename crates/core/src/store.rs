//! Transactional article, feedback and pending-download state owned by the daemon.

use std::{path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::protocol::{Article, MAX_PAGE_ITEMS, Publication};

/// Durable interest labels remain independent of seen/read state and queue edits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Preference {
    Hide,
    HeadlineOnly,
    ReadLater,
}

/// Explicit user operations whose preference/queue effect can be undone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Action {
    Feedback(Preference),
    Queue(bool),
    Undo,
}

/// State persisted separately from publisher metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct UserState {
    pub seen: bool,
    pub preference: Option<Preference>,
    pub queued: bool,
    pub read: bool,
    pub position: u32,
}

/// A locally identified article and its independent user state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredArticle {
    pub id: i64,
    pub provider: String,
    pub article: Article,
    pub state: UserState,
}

/// Storage failures are mapped to safe client errors at the IPC boundary.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("invalid storage input: {0}")]
    Invalid(&'static str),
    #[error("article not found")]
    NotFound,
    #[error("database schema is newer than this application")]
    UnsupportedSchema,
    #[error("database operation failed")]
    Database(#[from] rusqlite::Error),
    #[error("stored document is invalid")]
    Document(#[from] serde_json::Error),
}

/// A connection with a single owner; the daemon must supply a private managed path.
pub struct Store {
    connection: Connection,
}

impl Store {
    /// Open/create storage and migrate atomically, without erasing unknown/newer data.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > 1 {
            return Err(StoreError::UnsupportedSchema);
        }
        connection.pragma_update(None, "journal_mode", "WAL")?;
        if version == 0 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(
                "CREATE TABLE articles (
                    id INTEGER PRIMARY KEY, provider TEXT NOT NULL, external_id TEXT NOT NULL,
                    metadata TEXT NOT NULL, sort_time INTEGER NOT NULL, sort_subsecond INTEGER NOT NULL,
                    UNIQUE(provider, external_id));
                CREATE INDEX article_order ON articles(sort_time DESC, sort_subsecond DESC, id DESC);
                CREATE TABLE user_state (
                    article_id INTEGER PRIMARY KEY REFERENCES articles(id), document TEXT NOT NULL);
                CREATE TABLE discoveries (
                    article_id INTEGER NOT NULL REFERENCES articles(id), subscription TEXT NOT NULL,
                    PRIMARY KEY(article_id, subscription));
                CREATE TABLE checkpoints (
                    provider TEXT NOT NULL, subscription TEXT NOT NULL, token TEXT,
                    PRIMARY KEY(provider, subscription));
                CREATE TABLE actions (
                    id INTEGER PRIMARY KEY, article_id INTEGER NOT NULL REFERENCES articles(id),
                    previous_preference TEXT, previous_queued INTEGER NOT NULL,
                    undone INTEGER NOT NULL DEFAULT 0);
                CREATE INDEX action_order ON actions(article_id, undone, id DESC);
                CREATE TABLE pending_downloads (
                    article_id INTEGER PRIMARY KEY REFERENCES articles(id));
                PRAGMA user_version = 1;",
            )?;
            transaction.commit()?;
        }
        Ok(Self { connection })
    }

    /// Import a page and its checkpoint as one transaction, preserving existing user state.
    pub fn import_page(
        &mut self,
        provider: &str,
        subscription: &str,
        articles: &[Article],
        checkpoint: Option<&str>,
        discovered_at: i64,
    ) -> Result<(), StoreError> {
        validate_key(provider, 256)?;
        validate_key(subscription, 4096)?;
        if articles.len() > MAX_PAGE_ITEMS || discovered_at < 0 {
            return Err(StoreError::Invalid("invalid page size or observation time"));
        }
        if let Some(token) = checkpoint {
            validate_key(token, 65536)?;
        }
        for article in articles {
            article.validate().map_err(StoreError::Invalid)?;
        }
        let transaction = self.connection.transaction()?;
        for article in articles {
            let published = match &article.published {
                Some(Publication::Instant(value)) => chrono::DateTime::parse_from_rfc3339(value)
                    .ok()
                    .map(|date| (date.timestamp(), date.timestamp_subsec_nanos())),
                Some(Publication::Date(value)) => {
                    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                        .ok()
                        .and_then(|date| date.and_hms_opt(0, 0, 0))
                        .map(|date| (date.and_utc().timestamp(), 0))
                }
                None => None,
            };
            transaction.execute(
                "INSERT INTO articles(provider, external_id, metadata, sort_time, sort_subsecond) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(provider, external_id) DO UPDATE SET metadata=excluded.metadata,
                 sort_time=CASE WHEN ?6 THEN excluded.sort_time ELSE articles.sort_time END,
                 sort_subsecond=CASE WHEN ?6 THEN excluded.sort_subsecond ELSE articles.sort_subsecond END",
                params![provider, article.id, serde_json::to_string(article)?, published.unwrap_or((discovered_at, 0)).0, published.unwrap_or((discovered_at, 0)).1, published.is_some()])?;
            let id: i64 = transaction.query_row(
                "SELECT id FROM articles WHERE provider=?1 AND external_id=?2",
                params![provider, article.id],
                |row| row.get(0),
            )?;
            transaction.execute(
                "INSERT OR IGNORE INTO user_state VALUES (?1, ?2)",
                params![id, serde_json::to_string(&UserState::default())?],
            )?;
            transaction.execute(
                "INSERT OR IGNORE INTO discoveries VALUES (?1, ?2)",
                params![id, subscription],
            )?;
        }
        transaction.execute("INSERT INTO checkpoints VALUES (?1, ?2, ?3) ON CONFLICT(provider,subscription) DO UPDATE SET token=excluded.token", params![provider, subscription, checkpoint])?;
        transaction.commit()?;
        Ok(())
    }

    /// Return a bounded newest-first page with deterministic tie breaking.
    /// Date-only sort keys use a deterministic day boundary; metadata retains its precision.
    pub fn list(&self, limit: u32, offset: u32) -> Result<Vec<StoredArticle>, StoreError> {
        if limit == 0 || limit > 200 || offset > 100_000 {
            return Err(StoreError::Invalid("invalid pagination"));
        }
        let mut statement = self.connection.prepare("SELECT a.id,a.provider,a.metadata,s.document FROM articles a JOIN user_state s ON a.id=s.article_id ORDER BY a.sort_time DESC,a.sort_subsecond DESC,a.id DESC LIMIT ?1 OFFSET ?2")?;
        let rows = statement.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut output = Vec::new();
        for row in rows {
            let (id, provider, article, state) = row?;
            output.push(StoredArticle {
                id,
                provider,
                article: serde_json::from_str(&article)?,
                state: serde_json::from_str(&state)?,
            });
        }
        Ok(output)
    }

    /// Read the independent state for one existing article.
    pub fn state(&self, id: i64) -> Result<UserState, StoreError> {
        let text: Option<String> = self
            .connection
            .query_row(
                "SELECT document FROM user_state WHERE article_id=?1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(serde_json::from_str(&text.ok_or(StoreError::NotFound)?)?)
    }

    /// Atomically change preference/queue state, undo history and pending download work.
    pub fn act(&mut self, id: i64, action: Action) -> Result<UserState, StoreError> {
        let mut state = self.state(id)?;
        let transaction = self.connection.transaction()?;
        match action {
            Action::Undo => {
                let previous: Option<(i64, Option<String>, bool)> = transaction.query_row("SELECT id,previous_preference,previous_queued FROM actions WHERE article_id=?1 AND undone=0 ORDER BY id DESC LIMIT 1", [id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()?;
                if let Some((event, preference, queued)) = previous {
                    state.preference = preference
                        .map(|value| serde_json::from_str(&value))
                        .transpose()?;
                    state.queued = queued;
                    transaction.execute("UPDATE actions SET undone=1 WHERE id=?1", [event])?;
                }
            }
            change => {
                let before = state.clone();
                match change {
                    Action::Feedback(preference) => {
                        state.preference = Some(preference);
                        state.queued = preference == Preference::ReadLater;
                    }
                    Action::Queue(queued) => state.queued = queued,
                    Action::Undo => unreachable!(),
                }
                if before.preference != state.preference || before.queued != state.queued {
                    transaction.execute("INSERT INTO actions(article_id,previous_preference,previous_queued) VALUES (?1,?2,?3)", params![id, before.preference.map(|value| serde_json::to_string(&value)).transpose()?, before.queued])?;
                }
            }
        }
        transaction.execute(
            "UPDATE user_state SET document=?2 WHERE article_id=?1",
            params![id, serde_json::to_string(&state)?],
        )?;
        if state.queued {
            transaction.execute("INSERT OR IGNORE INTO pending_downloads VALUES (?1)", [id])?;
        } else {
            transaction.execute("DELETE FROM pending_downloads WHERE article_id=?1", [id])?;
        }
        transaction.commit()?;
        Ok(state)
    }

    /// Change exposure/reading position without manufacturing an interest label or queue edit.
    pub fn mark_reading(
        &mut self,
        id: i64,
        seen: bool,
        read: bool,
        position: u32,
    ) -> Result<UserState, StoreError> {
        let mut state = self.state(id)?;
        state.seen = seen;
        state.read = read;
        state.position = position;
        self.connection.execute(
            "UPDATE user_state SET document=?2 WHERE article_id=?1",
            params![id, serde_json::to_string(&state)?],
        )?;
        Ok(state)
    }

    /// Return the next bounded batch of durable downloads without changing their state.
    pub fn pending_downloads(&self) -> Result<Vec<i64>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT article_id FROM pending_downloads ORDER BY article_id LIMIT 200")?;
        Ok(statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?)
    }

    /// Return the last atomically accepted provider/subscription checkpoint.
    pub fn checkpoint(
        &self,
        provider: &str,
        subscription: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(self
            .connection
            .query_row(
                "SELECT token FROM checkpoints WHERE provider=?1 AND subscription=?2",
                params![provider, subscription],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }
}

fn validate_key(value: &str, maximum: usize) -> Result<(), StoreError> {
    if value.trim().is_empty() || value.len() > maximum || value.contains('\0') {
        Err(StoreError::Invalid("empty, oversized or NUL key"))
    } else {
        Ok(())
    }
}
