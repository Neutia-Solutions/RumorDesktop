use rumor_core::{
    protocol::{Article, Publication},
    store::{Action, Preference, Store},
};

fn article(id: &str) -> Article {
    Article {
        id: id.into(),
        url: format!("https://news.invalid/{id}"),
        title: "A synthetic café notice".into(),
        summary: String::new(),
        published: Some(Publication::Instant("2026-11-10T09:00:00Z".into())),
        section: None,
    }
}

#[test]
fn refresh_and_restart_preserve_feedback_provenance_and_jobs() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader.db");
    let mut store = Store::open(&path).unwrap();
    store
        .import_page("synthetic", "home", &[article("a")], Some("page2"), 100)
        .unwrap();
    let id = store.list(10, 0).unwrap()[0].id;
    store
        .act(id, Action::Feedback(Preference::ReadLater))
        .unwrap();
    store.mark_reading(id, true, true, 42).unwrap();
    let mut updated = article("a");
    updated.title = "Updated title".into();
    store
        .import_page("synthetic", "world", &[updated], None, 200)
        .unwrap();
    assert_eq!(store.list(10, 0).unwrap().len(), 1);
    drop(store);
    let store = Store::open(&path).unwrap();
    let saved = store.list(10, 0).unwrap().pop().unwrap();
    assert_eq!(saved.article.title, "Updated title");
    assert_eq!(saved.state.preference, Some(Preference::ReadLater));
    assert!(saved.state.queued && saved.state.seen && saved.state.read);
    assert_eq!(saved.state.position, 42);
    let readback = rusqlite::Connection::open(&path).unwrap();
    let memberships: i64 = readback
        .query_row(
            "SELECT count(*) FROM discoveries WHERE article_id=?1",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(memberships, 2);
    assert_eq!(store.pending_downloads().unwrap(), vec![id]);
    assert_eq!(
        store.checkpoint("synthetic", "home").unwrap().as_deref(),
        Some("page2")
    );
}

#[test]
fn invalid_page_never_partially_imports_or_advances_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("reader.db")).unwrap();
    store
        .import_page("synthetic", "home", &[article("a")], Some("old"), 100)
        .unwrap();
    let mut bad = article("bad");
    bad.url = "file:///private".into();
    assert!(
        store
            .import_page("synthetic", "home", &[article("b"), bad], Some("new"), 200)
            .is_err()
    );
    assert_eq!(store.list(10, 0).unwrap().len(), 1);
    assert_eq!(
        store.checkpoint("synthetic", "home").unwrap().as_deref(),
        Some("old")
    );
}

#[test]
fn queue_undo_does_not_corrupt_feedback_or_reading_state() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("reader.db")).unwrap();
    store
        .import_page("synthetic", "home", &[article("a")], None, 100)
        .unwrap();
    let id = store.list(10, 0).unwrap()[0].id;
    store
        .act(id, Action::Feedback(Preference::ReadLater))
        .unwrap();
    let removed = store.act(id, Action::Queue(false)).unwrap();
    assert_eq!(removed.preference, Some(Preference::ReadLater));
    assert!(!removed.queued);
    assert!(store.pending_downloads().unwrap().is_empty());
    store.mark_reading(id, true, true, 20).unwrap();
    assert!(store.act(id, Action::Undo).unwrap().queued);
    let original = store.act(id, Action::Undo).unwrap();
    assert_eq!(original.preference, None);
    assert!(!original.queued);
    assert!(original.seen && original.read);
    assert_eq!(original.position, 20);
    assert_eq!(store.act(id, Action::Undo).unwrap(), original);
}

#[test]
fn repeated_identical_feedback_creates_no_extra_undo_step() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("reader.db")).unwrap();
    store
        .import_page("synthetic", "home", &[article("a")], None, 100)
        .unwrap();
    let id = store.list(10, 0).unwrap()[0].id;
    for _ in 0..2 {
        store
            .act(id, Action::Feedback(Preference::HeadlineOnly))
            .unwrap();
    }
    assert_eq!(store.act(id, Action::Undo).unwrap().preference, None);
    let state = store.mark_reading(id, true, false, 4).unwrap();
    assert_eq!(state.preference, None);
    assert!(!state.queued);
}

#[test]
fn unknown_and_date_only_values_keep_precision_with_stable_order() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("reader.db")).unwrap();
    let mut unknown = article("unknown");
    unknown.published = None;
    let mut date_only = article("date");
    date_only.published = Some(Publication::Date("2026-11-10".into()));
    store
        .import_page("synthetic", "home", &[unknown.clone(), date_only], None, 1)
        .unwrap();
    store
        .import_page("synthetic", "home", &[unknown], None, 2_000_000_000)
        .unwrap();
    let rows = store.list(10, 0).unwrap();
    assert_eq!(rows[0].article.id, "date");
    assert!(matches!(
        rows[0].article.published,
        Some(Publication::Date(_))
    ));
    assert!(rows[1].article.published.is_none());
    assert!(store.list(0, 0).is_err());
    assert!(store.act(999, Action::Queue(true)).is_err());
}

#[test]
fn refuses_newer_schema_without_modifying_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    assert!(Store::open(&path).is_err());
    let connection = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        2
    );
}

#[test]
fn fractional_publication_seconds_sort_chronologically() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("reader.db")).unwrap();
    let mut newer = article("newer");
    newer.published = Some(Publication::Instant("2026-11-10T09:00:00.900Z".into()));
    let mut older = article("older");
    older.published = Some(Publication::Instant("2026-11-10T09:00:00.100Z".into()));
    store
        .import_page("synthetic", "home", &[newer, older], None, 1)
        .unwrap();
    assert_eq!(store.list(10, 0).unwrap()[0].article.id, "newer");
}
