# Settle the terminal redraw test before it measures

Jason requested a `codex duo` mission on 2026-10-26 to fix a flaky test and open a PR. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-terminal-redraw-test-settle` on `fix/terminal-redraw-test-settle`, based on `origin/main` at `b74ff53b7087be80a941e78110ea11a7c6bf24f9`. The root checkout stays on `main`, and this worktree uses its own `target/`.

## Problem

`terminal_redraw_tests::terminal_output_reuses_split_siblings_and_title_changes_update_sidebar` in `crates/runner-app/src/terminal_redraw_tests.rs` fails intermittently in CI:

- macOS on `main` at `b74ff53b`, run 40289082995;
- Windows on `main` at `9ea12ce8`, run 40156132031;
- macOS on `nightly-remote` at `0270ad64`, run 40066529511.

The same tree as the first failure passed macOS CI an hour earlier on its PR.

Two assertions fail:

- line 110, the first output burst: `unchanged view re-rendered: pane:p3: {"pane:p1": 1, "pane:p3": 1, "terminal:busy-shell": 1, "terminal:direct-agent": 1}` (macOS, twice);
- line 272, the drawer phase: `idle pane must stay cached beside a busy drawer: {"pane:p1": 1, "pane:p3": 1, "terminal:busy-shell": 2, "terminal:direct-agent": 1}` (Windows).

Each pane is a `CachedRegion` (`surfaces/render_cache.rs`) that observes `NativeRoot`, so any root notify re-renders every pane. In both failures every pane repainted once, so the root was notified once inside a measured window.

An investigation on 2026-10-26 with temporary `eprintln!` probes, since removed, found this:

- The test starts measuring before setup has settled. The in-process daemon delivers events on real threads, and `run_until_parked` does not wait for them.
- The last setup events are `window_focus_map` broadcasts. `NativeRoot::report_current_subjects` (`surfaces/windowing.rs`) reports the window's subjects when the tab attaches; the daemon broadcasts the map; `AppStore` maps the event to `StoreRefreshKind::Activity`; `revisions.activity` changes; `handle_app_store_update` (`main.rs`) notifies the root. Terminal `Resized` acknowledgments from the frame worker (`runner-terminal/src/terminal.rs`, `start_frames`) and label changes in the wake loop (`app_store.rs`) also arrive asynchronously during setup.
- Isolated runs pass: 40 of 40, and 40 of 40 with every core loaded. Running 24 copies of the test binary in parallel, one process each as nextest does, reproduced the line-110 failure once in 240 runs.
- The reproduced failure had no terminal label change inside the window, but did have an extra `Resized seq=3` acknowledgment during setup. Which late event notified the root was not confirmed. `window_focus_map` → activity refresh is the leading suspect.

Two side observations, unconfirmed as causes:

- In the drawer phase, `busy-shell` is shown in a pane and in the drawer at once, and its `Resized` frames alternate between `52x28` and `108x11` many times. The two views may be fighting over the terminal's size. That could explain the Windows failure at line 272, where `busy-shell` painted twice.
- With 8 to 24 parallel copies, most runs time out after 5 seconds in `TerminalModel::flush_events` (`runner-terminal/src/terminal/model.rs`, the `recv_timeout(...).unwrap()`), which the test calls in its title phase. This has not failed in CI.

## Outcome

The test measures only what it means to measure. Before each measured window (after `counts.0.borrow_mut().clear()`), the app and daemon have settled, so nothing pending from setup or the previous phase lands inside it. Keep every existing render-count assertion and its strictness. The fix belongs in the test or its test-only helpers unless investigation shows a product bug.

Find out which event actually lands inside the window, using probes or a deterministic reproduction, and fix that cause rather than only adding delay. Acceptable approaches include:

- a bounded settle step that repeats `run_until_parked` and clock advances until the store's revisions and the render counters stop changing;
- waiting for the specific outstanding work, such as the focus map with the attached tab's subjects or the final resize acknowledgment.

Avoid an unbounded loop, a fixed real-time sleep presented as the fix, `#[ignore]`, retries, or relaxed assertions.

For the drawer-phase resize alternation: determine whether it is a product bug where two views of one terminal fight over its size. If so, report it in the PR and to Jason with evidence, and do not fix it in this mission, unless it is the direct cause of the line-272 failure and the fix is small and clearly correct. The same applies to the `flush_events` timeout: fix it only if the cause is in the test.

Non-goals: no UI or behavior changes, no other tests, no CI configuration changes, no new frameworks.

## Validation

Before running checks, list what changed and why each check covers it.

- Reproduce the flake before the fix the way the investigation did: build the test binary and run many copies of this one test in parallel as separate processes, for example 240 runs with `xargs -P 24`. Record the failure rate, telling the line-110/272 failures apart from the `flush_events` timeout.
- After the fix, run at least 720 parallel runs with no line-110 or line-272 failure, and report the counts.
- Run `cargo test -p runner-app --bin Runner terminal_redraw_tests`, `make clippy` and `cargo fmt --all --check`.
- No live app checks are needed: this changes a test, and no running app behavior changes. If a product fix becomes in scope, the reviewer decides on live checks under Test Scope in `AGENTS.md`.
- Drive the macOS and Windows CI checks green.

## Authorization and handoff

Jason authorized this `codex duo` mission: one coder and one reviewer, no other agents or internal subagents. The coder implements and validates, and requests review through Runner on the working-tree diff. After a clean review, the coder commits, pushes this branch and opens one PR against `main`, then drives CI green on both platforms.

Fold this brief into the fix commit; one focused commit is expected. Fold review and CI fixes into the commit they correct, and use `git push --force-with-lease` when rewriting pushed history. Do not add tool-specific co-author trailers. Rebase only when `AGENTS.md` requires it; never merge `main` into the branch.

The PR description states the cause found, the before and after reproduction counts, and any product issue observed but not fixed. Stop at an open PR with green CI, and report the PR URL and final SHA through Runner. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. When waiting for a teammate, finish the turn and rely on Runner's pushed inbox notification.
