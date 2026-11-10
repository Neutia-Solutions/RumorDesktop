//! Synthetic provider executable for isolated host/protocol integration checks.

use std::io::{self, BufRead, Read, Write};

use rumor_core::protocol::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = std::env::args().nth(1).unwrap_or_default();
    if scenario == "worker" {
        std::thread::sleep(std::time::Duration::from_secs(60));
        return Ok(());
    }
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(MAX_FRAME_BYTES as u64 + 1)
        .read_until(b'\n', &mut bytes)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err("request too large".into());
    }
    let request: Request = serde_json::from_slice(&bytes)?;
    request.validate()?;
    match scenario.as_str() {
        "descendant" => {
            let mut worker = std::process::Command::new(std::env::current_exe()?)
                .arg("worker")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            let path = std::env::args().nth(2).ok_or("missing PID path")?;
            std::fs::write(path, format!("{} {}", std::process::id(), worker.id()))?;
            let _ = worker.wait()?;
            return Ok(());
        }
        "hang" => {
            std::thread::sleep(std::time::Duration::from_secs(60));
            return Ok(());
        }
        "oversized" => {
            io::stdout().write_all(&vec![b'x'; MAX_FRAME_BYTES + 1])?;
            return Ok(());
        }
        "malformed" => {
            println!("not JSON");
            return Ok(());
        }
        "empty" => return Ok(()),
        _ => {}
    }
    let result = if scenario == "intervention" {
        ProviderResult::NeedsAttention {
            url: "https://news.invalid/login".into(),
            token: "resume-1".into(),
            reason: AttentionReason::Login,
        }
    } else {
        match &request.operation {
            Operation::Describe => ProviderResult::Description {
                provider_id: "synthetic".into(),
                name: "Synthetic provider".into(),
                capabilities: vec![Capability::Discover, Capability::Fetch, Capability::Resume],
            },
            Operation::Discover { .. } | Operation::Resume { .. } => ProviderResult::Discovery {
                articles: vec![Article {
                    id: "synthetic-1".into(),
                    url: "https://news.invalid/story".into(),
                    title: "A synthetic story".into(),
                    summary: "A complete fixture metadata item.".into(),
                    published: Some(Publication::Instant("2026-11-10T09:00:00Z".into())),
                    section: Some("local".into()),
                }],
                checkpoint: Some("page-2".into()),
            },
            Operation::Fetch { article_id, .. } => ProviderResult::Content {
                article_id: article_id.clone(),
                body: include_str!("../../../tests/fixtures/reader.html").into(),
                format: ContentFormat::Html,
                status: if scenario == "partial" {
                    ContentStatus::Partial
                } else {
                    ContentStatus::Complete
                },
            },
        }
    };
    let response = Response {
        version: if scenario == "wrong-version" {
            VERSION + 1
        } else {
            VERSION
        },
        id: if scenario == "wrong-id" {
            request.id + 1
        } else {
            request.id
        },
        result,
    };
    let mut output = io::stdout().lock();
    serde_json::to_writer(&mut output, &response)?;
    writeln!(output)?;
    if scenario == "extra" {
        writeln!(output, "extra")?;
    }
    Ok(())
}
