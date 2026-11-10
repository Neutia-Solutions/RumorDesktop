//! Versioned one-request/one-response JSON Lines protocol for installed providers.

use serde::{Deserialize, Serialize};

/// Protocol version supported by this host.
pub const VERSION: u32 = 1;
/// Maximum wire frame including the newline delimiter.
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
/// Maximum number of articles returned in one discovery page.
pub const MAX_PAGE_ITEMS: usize = 1000;

/// A correlated operation sent to an explicitly configured provider executable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub id: u64,
    pub operation: Operation,
}

impl Request {
    /// Create a request at the host's supported protocol version.
    pub fn new(id: u64, operation: Operation) -> Self {
        Self {
            version: VERSION,
            id,
            operation,
        }
    }

    /// Validate local input before a process is launched.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION || self.id == 0 {
            return Err("invalid request version or ID");
        }
        match &self.operation {
            Operation::Describe => Ok(()),
            Operation::Discover {
                subscription,
                checkpoint,
            } => {
                bounded_text(subscription, 4096)?;
                if let Some(token) = checkpoint {
                    bounded_text(token, 65536)?;
                }
                Ok(())
            }
            Operation::Fetch { article_id, url } => {
                bounded_text(article_id, 4096)?;
                validate_url(url)
            }
            Operation::Resume { token } => bounded_text(token, 65536),
        }
    }
}

/// Operations are isolated process invocations; resume carries an opaque durable token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Describe,
    Discover {
        subscription: String,
        checkpoint: Option<String>,
    },
    Fetch {
        article_id: String,
        url: String,
    },
    Resume {
        token: String,
    },
}

/// A provider response carries the original request ID and protocol version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub version: u32,
    pub id: u64,
    pub result: ProviderResult,
}

impl Response {
    /// Correlate a reply and validate data before persistence or presentation.
    pub fn validate(&self, request: &Request) -> Result<(), &'static str> {
        if self.version != VERSION || self.id != request.id {
            return Err("response version or ID mismatch");
        }
        match &self.result {
            ProviderResult::Description {
                provider_id,
                name,
                capabilities,
            } => {
                if !matches!(request.operation, Operation::Describe) {
                    return Err("unexpected description response");
                }
                bounded_text(provider_id, 256)?;
                bounded_text(name, 512)?;
                if capabilities.len() > 3 {
                    return Err("too many capabilities");
                }
            }
            ProviderResult::Discovery {
                articles,
                checkpoint,
            } => {
                if !matches!(
                    request.operation,
                    Operation::Discover { .. } | Operation::Resume { .. }
                ) {
                    return Err("unexpected discovery response");
                }
                if articles.len() > MAX_PAGE_ITEMS {
                    return Err("discovery page too large");
                }
                for article in articles {
                    article.validate()?;
                }
                if let Some(token) = checkpoint {
                    bounded_text(token, 65536)?;
                }
            }
            ProviderResult::Content {
                article_id,
                body,
                format: _,
                status,
            } => {
                if !matches!(
                    request.operation,
                    Operation::Fetch { .. } | Operation::Resume { .. }
                ) {
                    return Err("unexpected content response");
                }
                bounded_text(article_id, 4096)?;
                if let Operation::Fetch {
                    article_id: expected,
                    ..
                } = &request.operation
                    && article_id != expected
                {
                    return Err("content article ID mismatch");
                }
                if *status == ContentStatus::Complete && body.trim().is_empty() {
                    return Err("complete article has no content");
                }
            }
            ProviderResult::NeedsAttention {
                url,
                token,
                reason: _,
            } => {
                if matches!(request.operation, Operation::Describe) {
                    return Err("description cannot require interaction");
                }
                validate_url(url)?;
                bounded_text(token, 65536)?;
            }
            ProviderResult::Failure {
                code: _,
                retry_after_seconds,
            } => {
                if retry_after_seconds.is_some_and(|seconds| seconds > 86400) {
                    return Err("retry interval too large");
                }
            }
        }
        Ok(())
    }
}

/// Structured results contain no arbitrary diagnostic text or credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProviderResult {
    Description {
        provider_id: String,
        name: String,
        capabilities: Vec<Capability>,
    },
    Discovery {
        articles: Vec<Article>,
        checkpoint: Option<String>,
    },
    Content {
        article_id: String,
        body: String,
        format: ContentFormat,
        status: ContentStatus,
    },
    NeedsAttention {
        url: String,
        token: String,
        reason: AttentionReason,
    },
    Failure {
        code: FailureCode,
        retry_after_seconds: Option<u64>,
    },
}

/// Supported provider operations, negotiated through description.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Discover,
    Fetch,
    Resume,
}

/// Machine-readable reason for an owner-assisted browser interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttentionReason {
    Login,
    Consent,
    Check,
    BrowserRequired,
}

/// Recoverable source states are distinct from protocol/transport failures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureCode {
    Unavailable,
    Blocked,
    LoginRequired,
    RateLimited,
    Temporary,
}

/// Available content format; HTML is untrusted and must be sanitized by the reader.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentFormat {
    Text,
    Html,
}

/// Body completeness is an explicit provider claim, not inferred from length.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Complete,
    Partial,
    Unavailable,
}

/// Original publication dates retain their precision rather than inventing times.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "precision",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Publication {
    Instant(String),
    Date(String),
}

/// Normalized metadata, independent of any named publisher or source implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Article {
    pub id: String,
    pub url: String,
    pub title: String,
    pub summary: String,
    pub published: Option<Publication>,
    pub section: Option<String>,
}

impl Article {
    /// Reject malformed metadata before it enters the app's durable state.
    pub fn validate(&self) -> Result<(), &'static str> {
        bounded_text(&self.id, 4096)?;
        bounded_text(&self.title, 4096)?;
        validate_url(&self.url)?;
        if self.summary.len() > 65536 {
            return Err("summary too large");
        }
        if let Some(section) = &self.section {
            bounded_text(section, 4096)?;
        }
        match &self.published {
            Some(Publication::Instant(value)) => {
                chrono::DateTime::parse_from_rfc3339(value)
                    .map_err(|_| "invalid publication instant")?;
            }
            Some(Publication::Date(value)) => {
                chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                    .map_err(|_| "invalid publication date")?;
            }
            None => {}
        }
        Ok(())
    }
}

fn bounded_text(value: &str, maximum: usize) -> Result<(), &'static str> {
    if value.trim().is_empty() || value.len() > maximum || value.contains('\0') {
        Err("text is empty, oversized or contains NUL")
    } else {
        Ok(())
    }
}

fn validate_url(value: &str) -> Result<(), &'static str> {
    bounded_text(value, 8192)?;
    let parsed = url::Url::parse(value).map_err(|_| "invalid URL")?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err("URL must be HTTP(S) without embedded credentials");
    }
    Ok(())
}
