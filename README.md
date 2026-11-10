# RumorDesktop

A local-first desktop news reader built with Rust and Rune. The application is [MIT licensed](LICENSE).

Review headlines in publication order, record whether you want to hide an item, see only its headline, or read it later, and read saved articles in the app. A local preference model learns from explicit feedback while preserving chronological ordering.

The interface must be entirely keyboard-operable, with Vim-style defaults. A separate daemon owns background jobs and persistence, continuing after the reader window closes. Provider integrations use a versioned plugin interface; the public app builds and tests without private plugins or credentials.

## Development status

Repository/GitHub tooling, a versioned provider contract and a bounded Linux subprocess host are implemented and tested with an executable synthetic provider. The Rune application and daemon remain unimplemented; no desktop build or real source integration is claimed.

Install the development requirement from `tools/requirements.txt` and provision the pinned Rust toolchain/dependencies explicitly, then run `make help` and `make check`. Rust validation uses the generated lockfile and offline mode. The full application gate still fails until the desktop and daemon suites exist.

## Documentation

- [Feature list](FEATURES.md).
- [Agent instructions](AGENTS.md), [stack](STACK.md) and [contribution guide](CONTRIBUTING.md).
- [Architecture](docs/ARCHITECTURE.md), [Rune verification](docs/RUNE-VERIFICATION.md) and [test conventions](tests/README.md).
- [Provider wire/process contract](docs/PROVIDER-PROTOCOL.md).
- [GitHub workflow](docs/GITHUB-WORKFLOW.md) and [security reporting](SECURITY.md).

This repository contains source-independent product documentation, public interfaces and synthetic fixtures. Provider inventories, access research, internal planning and personal handoffs belong outside this checkout.
