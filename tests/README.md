# Tests and fixtures

Executable tooling tests cover repository validation, fixtures and capture-runner failure/isolation. Rust integration tests additionally drive the real synthetic provider through the bounded process host, including deadlines and descendant cleanup. **They do not validate article extraction, daemon recovery, model behavior or Rune interaction.** Run tooling with `make check-fast` and Rust checks with `make check-rust`, after explicitly provisioning pinned prerequisites. The YAML/privacy gate is not hosted Actions validation or a secret scan.

## Fixture conventions

All checked-in examples under `fixtures/` are authored synthetic data, listed in [the manifest](fixtures/manifest.json). Use reserved `.invalid` URLs, fixed timestamps and fictional content. Record new entries with path, kind, `provenance: synthetic` and the scenario they support. The checker validates JSON/RSS syntax and contained paths; HTML stays deliberately capable of representing hostile or challenge pages.

The reader fixture contains Unicode, an article link, a table and code. The challenge fixture represents a non-article page. The feed includes a tracking URL and a stable GUID for future identity/checkpoint regressions. Intended outcomes in the manifest describe future assertions, not passing application tests.

For tests requiring files, sockets or SQLite, use `tempfile::TempDir` or a standard-library temporary-directory helper once Rust dependencies are selected. Never use the owner's actual XDG app directories, cookie store or databases. Avoid fixed `/tmp` names and fixed ports. Integration tests should own/stop their child processes and use bounded waits. Do not install a systemd service in a test.

## Application gates to implement with the application

| Target | Required suite | Meaningful assertions |
|---|---|---|
| `make check-ui` | `crates/desktop/tests/keyboard.rs` in package `rumor-desktop` | Drive actual Rune keys; assert selected article, focus, label/queue command, text-input behavior, dialog cancellation/focus restoration and keyboard-accessible settings/source setup. |
| `make check-daemon` | `crates/daemon/tests/lifecycle.rs` in package `rumord` | Real child daemon with isolated data/socket paths; UI disconnect does not stop jobs; restart preserves state; a second daemon is rejected; partial work resumes once. |
| `make check-rust` | Workspace unit/integration tests | Identity, feedback/queue atomicity, parser/sanitizer failures, duplicate-group corrections, classifier leakage and artifact rollback as each feature is built. |

Add these suites when their code exists; do not create empty or ignored tests to turn the gates green. Headless render capture is separate from semantic assertions. See [Rune verification](../docs/RUNE-VERIFICATION.md).

Live-source probes are opt-in manual experiments with a recorded access scope, source, date and account/quota assumptions. Use them to measure coverage and extraction; keep real article bodies, account data, logs and browser state out of fixtures and Git. An HTTP 200 or valid XML alone is not proof of complete source support.
