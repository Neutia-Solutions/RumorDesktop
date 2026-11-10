# GitHub workflow and repository visibility

## Repository boundary

| Public RumorDesktop | Each private adapter repository |
|---|---|
| Rune client, daemon, core data/state/model logic | Source discovery and article-fetching implementations |
| Versioned adapter protocol, host interfaces and synthetic contract fixtures | Feed/API adapters, selectors, extraction rules and browser automation |
| Generic sanitization/validation and synthetic test provider | Source-specific captured test material when retention is allowed |
| Public issues, CI and public app artifacts | Implementation issues, CI and private adapter artifacts |

The public repo is **MIT licensed**. Plugin implementations are maintained separately. Provider inventories, access research, personal plans and internal handoffs stay outside this public checkout. No private checkout, Git URL, token, path dependency or optional Cargo feature may be required to resolve, build, test or package the public app. Cargo may resolve optional dependencies into a lockfile, so an optional private Git dependency is not an acceptable boundary.

Use separately installed trusted adapter executables with a versioned JSON subprocess protocol. This replaces the earlier compiled-in adapter proposal. Public core defines the interface and mock/synthetic provider; private repositories implement it. Negotiate protocol version/capabilities and reject incompatibility clearly. The public app remains buildable with no adapters and offers a useful no-adapters-installed state. A subprocess is a packaging boundary, not a security sandbox. Implement its resource/input/lifecycle controls alongside the protocol.

Private adapter builds may depend on a pinned published public SDK/version, or its documented contract until a real SDK exists. Public CI never fetches private repositories. Install/run private executables from user-controlled runtime configuration outside the checkout. Keep browser credentials and profiles on the owner's machine. Do not bundle private binaries into public releases, debug symbols, source archives or artifact uploads.

The known-path repository guard rejects private implementation/runtime paths and submodules. It cannot recognize confidential code renamed into an ordinary public file and is not a secret scanner. Review the complete diff/history and use GitHub secret scanning/push protection before publication.

## Issues, PRs and review

Use the supplied bug and feature/task issue forms for source-independent application behavior. Provider requests, identities, access findings and implementation discussions stay in private workspaces/repositories. Public cross-repo work references a sanitized protocol task, not a confidential private issue URL. Private adapter issues may link public contract issues.

Use short-lived branches from `main`; a small PR links its acceptance criteria, actual checks, compatibility/recovery details and public/private boundary review. A completed feature must meet its criteria before closing its issue. CODEOWNERS routes review to `@Dev-Addict`, verified as the connected account; ensure that account has repository write access if hosted under an organization. It does not impose a second-person approval requirement.

The sole maintainer reviews the final revision, CI and resolved feedback before merging. Native Codex review may be configured on the public repository, with `@codex review` used only when a review request is authorized. Private-repo review findings must not be pasted into public discussions. Never run privileged follow-up workflows against untrusted PR code.

## CI and dependency updates

`CI / repository-checks` runs on PRs, pushes to main, merge groups and manual dispatch. It runs the current repository/tooling tests, YAML/workflow/form checks, fixture and known-path privacy validation. Permissions are contents-read; checkout credentials are not persisted, actions are pinned by full SHA, and no private repository token, cache or artifact upload is configured. Fork PRs run on GitHub-hosted disposable runners with no private data. No `pull_request_target` or privileged `workflow_run` execution is used.

`CI / rust-checks` is skipped until a real Cargo workspace exists. Once present, it provisions the declared tested toolchain, fetches locked public dependencies explicitly, then runs locked offline Rust checks. This setup does not claim an application or GUI pass. Add the real keyboard/daemon integration suites and appropriate graphics/system prerequisites to CI when those implementations exist; do not insert dummy passing tests.

Development YAML parsing uses **PyYAML 6.0.3**, pinned in `tools/requirements.txt`; no runtime application dependency is added. Install it deliberately in your development Python environment before `make check`, or use a local ignored `.venv`. Validation itself does not install dependencies. CI performs the documented explicit install. Actions pins currently correspond to checkout 7.0.1 and setup-python 7.1.0; update the SHA through review, not moving action tags.

One Dependabot configuration proposes weekly updates for Actions and the Python tooling dependency, without automerge. Add Cargo updates after a real workspace exists; handle Rune revision/toolchain upgrades deliberately with rendering and keyboard checks. Do not add a second update bot. GitHub-side configuration must still be verified after hosting.

## Hosting settings to apply when repositories are created

No repository was created, pushed, made public/private or configured remotely by this local setup.

For RumorDesktop:

1. Create the public repo only from reviewed public files and history. Check that sibling adapters are not staged or embedded. Keep this repository's MIT license; private adapter licenses remain unset.
2. Enable Issues, Actions, private vulnerability reporting, secret scanning and push protection where available. Test the private report path in [SECURITY.md](../SECURITY.md) before relying on it. GitHub settings do not come into force merely because this document exists.
3. Add the labels below. Issue-form labels/release categories need existing repository labels. Disable unused features if they become noise; no Discussions/project board is required initially.
4. Protect `main` against deletion and force pushes; require PRs, resolved conversations and the actual successful `repository-checks` context once its first hosted run identifies it. Add `rust-checks` as required only when the application job really runs. Do not require an external or code-owner approval that the sole maintainer cannot provide. Keep bypasses narrow and avoid auto-merge initially.
5. Set Actions default token permissions to read-only. Approve fork workflows deliberately, restrict actions to reviewed/pinned providers, and keep private adapter/registry/browser credentials out of this repository and its environments. Do not use a self-hosted machine with personal profiles for public fork builds.
6. Configure native review separately if wanted. Confirm CODEOWNERS routing, issue forms, a fork PR run and merge-blocking behavior on GitHub. Local syntax checks do not prove these settings.

For **every adapter repository**: create it as private, verify access/inheritance, enable only trusted collaborators and private Actions/artifacts, and keep implementation issues there. No license file is selected. Never reuse a public release workflow that could upload private binaries or diagnostics. Define its own tests and CI when real adapter code exists; public templates do not authorize disclosure of adapter material.

Suggested labels:

| Label | Purpose |
|---|---|
| bug | Defect and fix |
| enhancement | Feature or new behavior |
| documentation | Documentation-only changes |
| dependencies | Reviewed dependency updates |
| breaking-change | Compatibility/migration changes |
| skip-changelog | Exclude maintenance noise from release notes |

## Releases

Use reviewed `v0.x.y` tags and draft GitHub releases when the app is ready. `.github/release.yml` groups generated notes; it does not create or publish releases. Include known limitations, tested platforms, protocol version/compatibility and upgrade instructions. Public packages include only the public app and allowed assets. Do not bundle private adapters, runtime databases, browser state, reading feedback or private build logs.

Release automation, signing/notarization, installer updates and automatic publishing are intentionally absent until real binaries and a distribution plan exist. Private adapters have separate versioning/releases and distribution permissions. Confirm both repo visibility and release asset destination before each upload.

## References

- [GitHub issue-form syntax](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms).
- [GitHub Actions secure use](https://docs.github.com/en/actions/reference/security/secure-use).
- [Private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).
- [Dependabot options](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference).
- [Licensing a repository](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository).
