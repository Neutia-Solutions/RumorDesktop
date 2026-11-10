ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
PYTHON ?= python3
export PYTHONDONTWRITEBYTECODE := 1
export RUSTUP_AUTO_INSTALL := 0

.PHONY: help check check-fast check-rust check-full check-ui check-daemon capture-ui run-desktop run-daemon

help:
	@printf '%s\n' 'check-fast: repository/fixture validation and tooling tests' 'check: check-fast plus Rust checks when a workspace exists' 'check-rust: require workspace; format, Clippy and tests offline/locked' 'check-ui: require desktop keyboard integration tests' 'check-daemon: require daemon lifecycle integration tests' 'check-full: all repository, Rust and application integration gates' 'capture-ui: Rune PNG/tree capture; UI_BIN=/absolute/path/to/binary' 'run-desktop / run-daemon: foreground development commands; no service installation'

check-fast:
	cd "$(ROOT)" && $(PYTHON) -m unittest discover -s tools -p 'test_*.py'
	cd "$(ROOT)" && $(PYTHON) tools/check_repository.py
	cd "$(ROOT)" && $(PYTHON) tools/check_github.py

check: check-fast
	@if test -f "$(ROOT)/Cargo.toml"; then $(MAKE) -C "$(ROOT)" check-rust; else printf '%s\n' 'Repository-only checks passed. No Cargo workspace exists; application validation has NOT run.'; fi

check-rust:
	@test -f "$(ROOT)/Cargo.toml" || { printf '%s\n' 'Missing Cargo workspace. Complete the Rune/toolchain spike first.' >&2; exit 1; }
	cd "$(ROOT)" && $(PYTHON) tools/check_repository.py
	cd "$(ROOT)" && cargo fmt --all -- --check
	cd "$(ROOT)" && cargo clippy --workspace --all-targets --locked --offline -- -D warnings
	cd "$(ROOT)" && cargo test --workspace --all-targets --locked --offline

check-ui: check-rust
	@test -f "$(ROOT)/crates/desktop/tests/keyboard.rs" || { printf '%s\n' 'Missing Rune keyboard integration suite; see docs/RUNE-VERIFICATION.md.' >&2; exit 1; }
	cd "$(ROOT)" && cargo test -p rumor-desktop --test keyboard --locked --offline

check-daemon: check-rust
	@test -f "$(ROOT)/crates/daemon/tests/lifecycle.rs" || { printf '%s\n' 'Missing daemon lifecycle integration suite; see tests/README.md.' >&2; exit 1; }
	cd "$(ROOT)" && cargo test -p rumord --test lifecycle --locked --offline

check-full: check-fast check-ui check-daemon

capture-ui:
	cd "$(ROOT)" && $(PYTHON) tools/capture_ui.py "$(UI_BIN)"

run-desktop:
	@test -f "$(ROOT)/crates/desktop/Cargo.toml" || { printf '%s\n' 'Rune client is not implemented yet.' >&2; exit 1; }
	cd "$(ROOT)" && cargo run -p rumor-desktop --locked --offline

run-daemon:
	@test -f "$(ROOT)/crates/daemon/Cargo.toml" || { printf '%s\n' 'Daemon is not implemented yet; no service was started.' >&2; exit 1; }
	cd "$(ROOT)" && cargo run -p rumord --locked --offline
