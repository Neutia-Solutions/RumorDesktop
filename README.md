# RumorDesktop

A local-first desktop news reader built with Rust and Rune. The application is [MIT licensed](LICENSE).

Review headlines in publication order, record whether you want to hide an item, see only its headline, or read it later, and read saved articles in the app. A local preference model learns from explicit feedback while preserving chronological ordering.

The interface must be entirely keyboard-operable, with Vim-style defaults. A separate daemon owns background jobs and persistence, continuing after the reader window closes. Provider integrations use a versioned plugin interface; the public app builds and tests without private plugins or credentials.

## Development status

Repository and GitHub workflow tooling is implemented. The Rune application, daemon and tested runtime dependency pins are not implemented yet. No application build or source integration is claimed.

Install the development requirement from `tools/requirements.txt` explicitly, then run `make help` and `make check`. Without a Cargo workspace, checks report repository-only validation; the full application gate fails until its prerequisites exist.

## Documentation

- [Feature list](FEATURES.md).
- [Agent instructions](AGENTS.md), [stack](STACK.md) and [contribution guide](CONTRIBUTING.md).
- [Architecture](docs/ARCHITECTURE.md), [Rune verification](docs/RUNE-VERIFICATION.md) and [test conventions](tests/README.md).
- [GitHub workflow](docs/GITHUB-WORKFLOW.md) and [security reporting](SECURITY.md).

This repository contains source-independent product documentation, public interfaces and synthetic fixtures. Provider inventories, access research, internal planning and personal handoffs belong outside this checkout.
