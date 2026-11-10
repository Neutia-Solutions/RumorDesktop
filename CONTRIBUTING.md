# Contributing

Start with [AGENTS.md](AGENTS.md), [STACK.md](STACK.md) and the relevant feature acceptance criteria. Use `main` and small task branches; no Gitflow or submodules. Inspect status and preserve existing work. Keep one purpose per change; Conventional Commit titles are preferred when committing is part of the authorized task.

## Workflow

1. For substantial work, copy [the task template](docs/tasks/TEMPLATE.md) into `docs/tasks/` and fill in concrete behavior and verification. Small edits need no ceremony. Reuse accepted decisions instead of requesting routine approval again.
2. Trace affected callers and trust boundaries. Keep the daemon's database ownership and the independent user states intact. Use [fixture conventions](tests/README.md); never collect real browser/account data for test convenience.
3. Implement the smallest complete change. Use doc comments on public Rust contracts; no ordinary/narrating code comments. Update affected docs and fixtures. New behavior gets meaningful tests, not tests that merely restate constants.
4. Run `make check`. Add `make check-ui` for UI work and `make check-daemon` for daemon/IPC work; run `make check-full` when integration scope warrants it. Live publisher probes require a separate, explicit scope and do not belong in deterministic checks.
5. Review the diff, including untracked files during bootstrap. Record actual validation and gaps. For a pause or transfer, use [the handoff template](docs/handoffs/TEMPLATE.md) and save the completed private handoff outside this checkout; put transient logs under ignored `.agent-state/` or `artifacts/`.

## Development boundaries

Local checks do not install toolchains/dependencies, start a service, authenticate to a site, publish a repo or mutate host settings. Missing required prerequisites fail clearly. Install/update dependencies deliberately under the host's permissions, then run the checks again; do not add online fallback to validation commands.

Do not edit generated dependency lockfiles by hand. Use Cargo to regenerate once a compatible toolchain and dependencies are available. Follow [the dependency policy](STACK.md) when introducing the first workspace or upgrading Rune.

GitHub issue forms, a PR template, CI, CODEOWNERS, release-note categories and one Dependabot configuration are prepared locally. Hosted execution/settings remain unverified. Hooks and repository-specific skills remain deferred. See [GitHub workflow](docs/GITHUB-WORKFLOW.md) for main-branch settings, review and publication boundaries.

## Public contributions and license

RumorDesktop is [MIT licensed](LICENSE). Contributions should be suitable for inclusion under that license and retain required third-party notices. Separately maintained plugins are not covered by this repository's license. Keep provider identities, inventories, access research, implementation work and confidential evidence outside this checkout and its public issues. Public tests use synthetic providers and must build without private checkouts, binaries or credentials.

Use the bug/feature forms, the PR template, and [SECURITY.md](SECURITY.md) for vulnerability reports. Public PRs should document a clean clone/build path and protocol compatibility when relevant.
