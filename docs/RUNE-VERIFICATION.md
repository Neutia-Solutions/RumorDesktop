# Rune verification

This workflow separates **capture tooling**, **application assertions** and **real desktop inspection**. Only the first is implemented in this agent setup. There is no compiled Rune client or keyboard integration suite yet, and no render/keyboard pass has been claimed.

## First implementation prerequisite

Retrieve/build a compatible Rune revision and pin it with the toolchain and lockfile per [STACK.md](../STACK.md). Resolve the current network/toolchain constraint before introducing an untested dependency pin. Follow the selected revision's own documentation; do not assume its latest `dev` API matches an older checkout.

Upstream documents headless PNG/tree capture, scene selection, fixed virtual time and scripted keyboard events in its [agent tooling reference](https://github.com/anomalyco/rune/blob/dev/crates/rune/README.md#agent-tooling-headless-no-window-same-renderer-fixed-virtual-clock). These are researched capabilities, not locally exercised APIs. Headless GPU execution still needs a compatible graphics device/backend.

## App integration contract

Implement a `fixture-inbox` scene using the checked-in synthetic fixtures. It must never connect to the owner's daemon, open a publisher browser, access personal data or make network requests. Resolve fixtures from compile-time/project resources rather than relying on the process working directory. Test this isolation with the app implementation; the capture runner's temporary XDG directories are not a security sandbox.

Provide stable element keys and appropriate semantic roles/labels. Wire the pinned Rune CLI's scene, theme, snapshot, tree and virtual-time options. Implement `crates/desktop/tests/keyboard.rs` as actual input-driven application tests, not a JSON fixture comparison or a test of a separate fake command model. Keep the package named `rumor-desktop` to match the Makefile.

## Capture command

Once a compatible fixture-only binary exists:

```sh
make capture-ui UI_BIN="$PWD/target/debug/rumor-desktop"
```

`tools/capture_ui.py` requests a light and dark PNG plus JSON tree at virtual time zero, using new temporary XDG data/config/cache/state paths and a separate temporary working directory. It uses a new ignored `artifacts/rune/capture-*` directory for each run, checks output existence/header/JSON shape, bounds execution and propagates failure. Inspect the actual images/tree; artifact syntax is not visual, accessibility or keyboard validation. A missing binary fails immediately. No service is started by the runner itself.

For motion/scroll states, add scenario scripts matching the **tested pinned Rune schema** when the UI exists; do not commit guessed scripts from a changing upstream API. Drive `j/k`, pane navigation, `gg/G`, search, feedback actions, input editing and dialog cancellation. Capture states after input, not just a static initial screen.

## Semantic gate

```sh
make check-ui
```

The suite must assert resulting selected IDs, focused control, feedback/queue commands, modal state and restored focus. Cover normal typing in text fields, cancelled multi-key prefixes, no focus traps, source setup/settings and links in the reader. Use temporary daemon state or a fixture-only IPC implementation where appropriate; test the real daemon boundary separately. No pointer action should be required for Rumor operations.

## Native inspection and handoff

On Linux/Wayland, inspect actual keyboard focus, text selection/copy, links, scrolling, Unicode/table/code rendering, narrow layout, themes and accessibility behavior. Verify closing Rune leaves the daemon active and reopening reconnects. Record screenshot/tree paths, commands, environment, exact Rune/toolchain revisions, observed behavior and gaps in the [handoff](handoffs/TEMPLATE.md).

`make check-ui` and `make check-full` currently fail for missing application prerequisites. Do not replace them with empty passing suites. Capture-runner unit tests simulate process outputs to verify orchestration only; they are not Rune tests.
