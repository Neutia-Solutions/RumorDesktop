use std::{ffi::OsString, path::PathBuf, time::Duration};

use rumor_core::{
    host::{HostError, ProviderCommand},
    protocol::*,
};

fn provider(scenario: &str) -> ProviderCommand {
    ProviderCommand {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_rumor-fixture-provider")),
        arguments: vec![OsString::from(scenario)],
        timeout: Duration::from_secs(2),
    }
}

fn discover() -> Request {
    Request::new(
        1,
        Operation::Discover {
            subscription: "synthetic".into(),
            checkpoint: None,
        },
    )
}

#[tokio::test]
async fn describes_discovers_fetches_and_resumes_with_real_child() {
    let description = provider("")
        .call(&Request::new(1, Operation::Describe))
        .await
        .unwrap();
    assert!(matches!(
        description.result,
        ProviderResult::Description { .. }
    ));
    let discovery = provider("").call(&discover()).await.unwrap();
    match discovery.result {
        ProviderResult::Discovery {
            articles,
            checkpoint,
        } => {
            assert_eq!(articles[0].id, "synthetic-1");
            assert_eq!(checkpoint.as_deref(), Some("page-2"));
        }
        _ => panic!("wrong response type"),
    }
    let fetch = Request::new(
        2,
        Operation::Fetch {
            article_id: "synthetic-1".into(),
            url: "https://news.invalid/story".into(),
        },
    );
    let content = provider("").call(&fetch).await.unwrap();
    assert!(matches!(
        content.result,
        ProviderResult::Content {
            status: ContentStatus::Complete,
            ..
        }
    ));
    let partial = provider("partial").call(&fetch).await.unwrap();
    assert!(matches!(
        partial.result,
        ProviderResult::Content {
            status: ContentStatus::Partial,
            ..
        }
    ));
    let intervention = provider("intervention").call(&discover()).await.unwrap();
    let ProviderResult::NeedsAttention { token, .. } = intervention.result else {
        panic!("missing intervention")
    };
    let resumed = provider("")
        .call(&Request::new(3, Operation::Resume { token }))
        .await
        .unwrap();
    assert!(matches!(resumed.result, ProviderResult::Discovery { .. }));
}

#[tokio::test]
async fn rejects_malformed_oversized_mismatched_and_extra_output() {
    for scenario in [
        "malformed",
        "oversized",
        "empty",
        "wrong-version",
        "wrong-id",
        "extra",
    ] {
        assert!(
            provider(scenario).call(&discover()).await.is_err(),
            "{scenario}"
        );
    }
}

#[tokio::test]
async fn deadline_is_bounded_and_errors_do_not_echo_provider_payloads() {
    let mut command = provider("hang");
    command.timeout = Duration::from_millis(50);
    assert!(matches!(
        command.call(&discover()).await,
        Err(HostError::Timeout)
    ));
    let error = provider("malformed").call(&discover()).await.unwrap_err();
    assert!(!error.to_string().contains("not JSON"));
}

#[tokio::test]
async fn rejects_invalid_local_input_without_spawning() {
    let command = ProviderCommand {
        executable: "/no/such/executable".into(),
        arguments: vec![],
        timeout: Duration::from_secs(1),
    };
    let bad = Request::new(
        1,
        Operation::Fetch {
            article_id: "a".into(),
            url: "https://secret:token@news.invalid/".into(),
        },
    );
    assert!(matches!(
        command.call(&bad).await,
        Err(HostError::InvalidRequest(_))
    ));
    let relative = ProviderCommand {
        executable: "relative".into(),
        ..command
    };
    assert!(matches!(
        relative.call(&discover()).await,
        Err(HostError::InvalidExecutable)
    ));
}

#[test]
fn semantic_validation_preserves_dates_and_rejects_wrong_content() {
    let article = Article {
        id: "a".into(),
        url: "https://news.invalid/a".into(),
        title: "Title".into(),
        summary: String::new(),
        published: Some(Publication::Date("2026-02-30".into())),
        section: None,
    };
    assert!(article.validate().is_err());
    let response = Response {
        version: VERSION,
        id: 1,
        result: ProviderResult::Content {
            article_id: "other".into(),
            body: String::new(),
            format: ContentFormat::Text,
            status: ContentStatus::Complete,
        },
    };
    assert!(
        response
            .validate(&Request::new(
                1,
                Operation::Fetch {
                    article_id: "a".into(),
                    url: "https://news.invalid/a".into()
                }
            ))
            .is_err()
    );
}

#[test]
fn unknown_nested_publication_fields_are_rejected() {
    let response = serde_json::json!({
        "version": VERSION, "id": 1,
        "result": { "type": "discovery", "checkpoint": null, "articles": [{
            "id": "synthetic-1", "url": "https://news.invalid/a", "title": "Title", "summary": "",
            "published": { "precision": "date", "value": "2026-11-10", "unexpected": "accepted" },
            "section": null
        }] }
    });
    assert!(serde_json::from_value::<Response>(response).is_err());
}

async fn process_ids(path: &std::path::Path) -> Vec<u32> {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(text) = std::fs::read_to_string(path) {
                let ids: Vec<u32> = text
                    .split_whitespace()
                    .map(|id| id.parse().unwrap())
                    .collect();
                if ids.len() == 2 {
                    return ids;
                }
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}

async fn assert_processes_dead(ids: &[u32]) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let running = ids.iter().any(|id| {
                std::fs::read_to_string(format!("/proc/{id}/stat")).is_ok_and(|stat| {
                    !stat
                        .rsplit_once(')')
                        .unwrap()
                        .1
                        .trim_start()
                        .starts_with('Z')
                })
            });
            if !running {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("owned process survived cleanup");
}

#[tokio::test]
async fn cancellation_terminates_provider_and_descendants() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("pids");
    let mut command = provider("descendant");
    command.arguments.push(path.as_os_str().to_owned());
    let task = tokio::spawn(async move { command.call(&discover()).await });
    let ids = process_ids(&path).await;
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_processes_dead(&ids).await;
}

#[tokio::test]
async fn timeout_terminates_provider_and_descendants() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("pids");
    let mut command = provider("descendant");
    command.arguments.push(path.as_os_str().to_owned());
    command.timeout = Duration::from_millis(300);
    let task = tokio::spawn(async move { command.call(&discover()).await });
    let ids = process_ids(&path).await;
    assert!(matches!(task.await.unwrap(), Err(HostError::Timeout)));
    assert_processes_dead(&ids).await;
}
