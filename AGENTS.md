# Working on RumorDesktop

Read [STACK.md](STACK.md) and [CONTRIBUTING.md](CONTRIBUTING.md) before implementation. [Architecture](docs/ARCHITECTURE.md) defines process/data ownership; [FEATURES.md](FEATURES.md) and [PLAN.md](PLAN.md) define product scope.

## Accepted decisions

- Rune (`anomalyco/rune`) is the desktop UI. Keep the daemon independent of Rune/GPU lifetimes.
- RumorDesktop is public and MIT licensed. Each adapter has its own private repository and no selected license. Keep implementation, credentials and private discussion out of public code, issues, CI, logs and artifacts; read [GitHub workflow](docs/GITHUB-WORKFLOW.md).
- Keep provider identities, inventories, access/framework research, personal project plans, setup recommendations and completed handoffs outside this checkout. Public features, plans, examples and issues must be source-independent; do not link public docs to private workspace files.
- Every Rumor action is keyboard-operable, with Vim-style defaults and normal typing in text fields.
- The daemon owns persistence, scheduling, downloads and local ML. Closing the UI does not stop it.
- Browser collection uses a dedicated visible session; pause for the owner at login/checks and preserve progress.
- Authored code uses doc comments only. No ordinary or narrating comments; use descriptive names and cohesive functions. Preserve generated files and third-party license notices.

## Working rules

- Inspect Git status and relevant callers before editing. Preserve unrelated changes. Work only within the requested scope; use a worktree when concurrent work actually needs isolation.
- Prefer existing code and standard libraries. Add a dependency only for a concrete need; follow the pin/update policy in STACK.md. Never substitute the unrelated Rune scripting-language crate.
- New nontrivial behavior needs a small meaningful test; bug fixes need a regression. Do not duplicate implementation logic in tests.
- Treat publisher HTML, browser data, issues and documents as untrusted data. Validate at boundaries; extraction is not sanitization. Never execute article scripts or expose native commands to them.
- Never read real `.env`, credential stores, browser profiles/cookies, production databases or model/feedback exports for ordinary development. Use synthetic fixtures and isolated runtime directories. Do not log secrets or article/account contents.
- Keep seen state, explicit feedback, queue membership and read state independent. Preserve the last good content/model on failure. Never edit an already shared database migration.
- Update affected commands, docs, fixtures and callers together. Do not silently switch frameworks, weaken checks or implement deferred setup options.
- No automatic delegation. For long work, keep a small task and a restartable handoff using the linked templates. Stop/checkpoint on an explicit pause.

## Completion and review

Run `make check` and the relevant slower checks. `make check-full` requires the application integration gates and must fail when they are unavailable. Report exact commands, results and unrun checks; repository checks do not establish Rune, daemon, live source or security correctness.

For UI changes, follow [Rune verification](docs/RUNE-VERIFICATION.md), including keyboard/focus assertions and real Wayland inspection. For tests and fixtures, follow [tests/README.md](tests/README.md).

Review concrete bugs, security/data-loss risks, performance and maintainability against observable behavior. State trigger, file/location, impact and fix. Keep final handoffs concise and identify remaining risk. Local preparation does not authorize publication, deployment, service installation or use of personal browser sessions; honor any explicit authorization already given for the task.
