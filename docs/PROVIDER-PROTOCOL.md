# Provider protocol version 1

The public `rumor-core` crate defines source-independent wire types, validation and a Linux process host. The `rumor-fixture-provider` executable supplies synthetic data for integration tests. No real provider, browser integration, daemon or desktop UI is implemented here yet.

## Wire and process contract

An explicitly configured absolute executable path is launched without a shell. Arguments remain literal. Each invocation receives one UTF-8 JSON request followed by a newline on stdin; stdin then closes. It returns exactly one newline-terminated JSON response on stdout and closes stdout. Version and request ID must match; unknown fields, malformed frames and trailing output are rejected. A valid response is the operation's completion record; do not use post-response exit housekeeping as another commit step.

The whole exchange has a configurable nonzero deadline of at most five minutes; requests and responses are each limited to 1 MiB including the newline. Discovery pages contain at most 1,000 articles. The host bounds reads before allocating a complete frame, discards stderr rather than risking a diagnostic/credential leak or full pipe, and terminates the owned process group on completion, error, timeout or cancellation. Normal invocations must not leave persistent workers behind. Resume state belongs in durable checkpoints rather than unmanaged background children.

Executables are deliberately installed trusted code. Process groups and typed JSON are not a security sandbox: the child runs as the same OS user and currently inherits the caller's environment. Do not run unknown plugins or pass ambient secrets to the hosting process. A future browser manager must own its lifecycle explicitly rather than escaping process cleanup.

## Operations and results

`Request` contains `version`, a nonzero `id`, and a tagged `operation`. Operations are `describe`, `discover` with subscription/checkpoint, `fetch` with article ID/URL, and `resume` with an opaque token. Obtain capabilities through `describe` before offering unsupported operations to the user.

`Response` contains `version`, matching `id` and a tagged `result`:

- `description`: runtime provider ID/name and discover/fetch/resume capabilities.
- `discovery`: article metadata and an optional next-page/checkpoint token. The consumer commits accepted items and checkpoint atomically.
- `content`: matching article ID, text or **untrusted HTML**, and complete/partial/unavailable status. Empty bodies cannot claim completeness. Extraction/completeness claims still need actual quality checks before offline-ready presentation.
- `needs_attention`: HTTP(S) URL, resume token and typed login/consent/check/browser-required reason. The user handles the interaction; the consumer persists/resumes the job without creating duplicate articles.
- `failure`: a typed availability/authentication/rate-limit/temporary state and optional retry interval capped at one day. Arbitrary remote error text is excluded from failure messages.

Article metadata includes provider-local ID, HTTP(S) URL without embedded credentials, title, summary, optional section and publication date precision. Publication values are RFC3339 instants, calendar dates or absent; the consumer must not fabricate an instant for date-only metadata. URL syntax validation is not network destination authorization: actual fetchers must also enforce configured origins, redirect rules and private-address restrictions.

## Validation

Run `make check` for the repository gate and locked offline Rust tests/Clippy/format checks. The provider integration suite drives actual child processes for discovery/content, partial results, intervention/resume, malformed/oversized/trailing output, ID/version mismatch, deadlines and cancellation with descendant cleanup. Synthetic tests do not prove source access or article completeness.

This workspace is currently tested on Rust 1.95.0 using cached pinned dependencies. That is a tested core toolchain, **not a compatible Rune pin**. Rune still requires a separately successful build spike before adding its dependency.
