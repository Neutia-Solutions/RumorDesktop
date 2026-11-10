# Stack and reproducibility

## Current state

This repository contains agent/GitHub tooling plus a Rust workspace with a source-independent provider contract, Linux subprocess host and executable synthetic provider. Rust 1.95.0 is pinned and tested for this core slice, with a Cargo-generated lockfile. The Rune client and daemon remain unimplemented; do not present protocol tests as a desktop application build. See [provider protocol](docs/PROVIDER-PROTOCOL.md).

Python 3.11+ (including `tomllib`), Git, Make and the pinned development requirement in `tools/requirements.txt` run the repository tools. PyYAML is used for real YAML parsing rather than a hand-written parser; install the requirement explicitly before validation. Python is development tooling; the application remains Rust. Run commands from this repository root; the Makefile also resolves paths from its own location when invoked with `make -f`.

## Accepted application stack

| Layer | Decision |
|---|---|
| Desktop | Rune from `https://github.com/anomalyco/rune`; exact revision pending a successful capability spike. |
| Background | Separate `rumord` process; user-level service integration initially on Linux. |
| Shared logic | Rust core modules; no Rune dependency in daemon/core. |
| Communication | Versioned local IPC; user-owned Unix socket initially; no TCP listener required. |
| Persistence/search | SQLite/FTS5, owned by daemon; migrations and last-good-copy preservation. |
| Sources | Versioned public subprocess protocol; one private repository per adapter. Feed/API/browser implementation code is private. Browser driver is not selected. |
| Learning | Local classifier and explicit feedback; implementation/library selection follows source feasibility. |

Initial runtime target: the owner's Linux/Wayland machine. Headless GPU rendering still needs an appropriate graphics adapter/backend. Desktop, mobile and web support claims require separate verification.

## Dependency and toolchain policy

1. At the first Cargo workspace, add `rust-toolchain.toml` with an **exact tested** release and `rustfmt`/`clippy` components. Declare the minimum supported Rust version in the workspace package metadata. Commit `Cargo.lock` for the application workspace; validation uses `--locked --offline` and never resolves new versions implicitly.
2. Pin Rune using `git = "https://github.com/anomalyco/rune"` and a full 40-character `rev`. Do not use a moving branch/tag or a registry package named `rune`. Centralize it in workspace dependencies.
3. Use exact direct registry requirements (`=x.y.z`) and the lockfile for transitives. Local workspace/path dependencies remain local. Record why a new dependency exists and any system/build-time requirements.
4. Dependency upgrades are explicit changes: update pins/lockfile, run the relevant checks and inspect actual Rune rendering/keyboard behavior. Keep the previous working revision recorded in the task/handoff. The single Dependabot configuration covers Actions and Python tooling; Cargo/Rune automation waits for a real workspace. No private adapter dependency, including optional/path dependencies outside this repository, belongs in the public dependency graph.
5. Never claim that a revision or toolchain is tested from a successful documentation lookup. The repository gate rejects a Cargo workspace without toolchain/lockfile inputs and rejects floating direct dependencies.

## Observed environment and current constraint

On 10 November 2026, local `rustc`, Cargo, Clippy and rustfmt were available; the default compiler reported 1.97.1, with a separate 1.95.0 toolchain also installed. Git retrieval of Rune failed because `github.com` could not resolve; no Rune checkout is cached. Upstream's current [toolchain file](https://github.com/anomalyco/rune/blob/dev/rust-toolchain.toml) declares 1.99.1. These observations do not establish a compatible, build-tested Rune pin.

The core workspace has since built and passed process integration tests on the separately installed 1.95.0 toolchain with cached dependencies. Its genuine generated lockfile does not include Rune. Resolving/building Rune remains blocked on dependency retrieval; update the toolchain deliberately if the actual selected Rune revision requires it. Do not confuse the core toolchain pin with a tested Rune version.

## Commands

See `make help`. `make check` runs repository tests/validation and, once a workspace exists, formatting, Clippy and Rust tests. `make check-rust` explicitly requires a workspace. `make check-ui`, `make check-daemon` and `make check-full` require the documented application test targets; their current missing-prerequisite failures are intentional and must be reported.

`make run-desktop` and `make run-daemon` will run the named Cargo packages in the foreground once implemented. They never install/enable a service. The first implementation must implement an explicit fixture mode before tests run either binary against app data.
