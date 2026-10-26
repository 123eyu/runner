# Terminal redraw test settling

Shipped in [0.18.1](https://github.com/yicheng47/runner/releases/tag/v0.18.1) through [PR #1039](https://github.com/yicheng47/runner/pull/1039). The results below retain the original candidate and failures; archiving adds no new live-test pass.

Candidate: `fix/terminal-redraw-test-settle`, based on `b74ff53b7087be80a941e78110ea11a7c6bf24f9`. Automated macOS investigation and validation on 2026-10-26; no live app checks were selected or run because only test code and a test-support helper change.

## Selected checks

- Run the original test in 240 separate processes at concurrency 24 to reproduce cross-phase renders and distinguish them from terminal event timeouts.
- Trace daemon events, root store reactions, focus-map notifications, resize frames and measurement boundaries with temporary probes to identify outstanding setup work. Remove all probes before final validation.
- Run the changed test in 720 separate processes at concurrency 24 to exercise the same scheduling pressure against every original assertion.
- Run `cargo test -p runner-app --bin Runner terminal_redraw_tests` to verify the actual Cargo test entry point; run `make clippy` and `cargo fmt --all --check` for workspace/type/format gates.
- Check both macOS and Windows PR CI after working-tree review.

## Investigation

The unchanged test failed 9/240 runs (3.75%): six line-110 sibling assertions (2.5%), two empty first-burst render counts, and one empty drawer-burst render count. There were zero line-272 idle-pane failures and zero `flush_events` timeouts. The run completed in 5.1 seconds.

A second 240-run batch with temporary probes reproduced three line-110 failures and one empty drawer-burst count (236 passed). In all three sibling failures, the last setup store reaction changed `terminal_wake` from 0 to 1 immediately before clearing the counters; the measured window began and ended with identical store revisions. The first two also received a late `direct-agent` resize acknowledgment (`seq=3`, `52x43`) just before that wake. Setup invalidation was already queued before measurement but painted afterward. This narrows the observed cause beyond the original focus-map hypothesis: waiting for revisions alone misses outstanding renders. The probes also confirmed the independent focus-map listener calls `sync_subject_ownership`, notifying the root even when store revisions are unchanged.

The test injects output directly with `TerminalMirror::test_feed`, using sequence numbers unrelated to the daemon's resize sequence. Real resize acknowledgments are still delivered and wake viewers even when their sequence is stale. The fixture now cancels and joins its frame workers before building the window, so these acknowledgments cannot race manually fed output. Terminal model title handling, metadata refreshes, store subscriptions and the UI rendering/cache paths remain exercised. Before each of the four measured phases, a two-second-bounded helper drains GPUI tasks and advances its clock by 10 ms until both store revisions and render counters remain unchanged across five turns. Every original render-count assertion is unchanged.

## Product observation left out of scope

With the same `busy-shell` displayed in its pane and drawer, probes recorded repeated alternating resize acknowledgments: `seq=15 52x28`, `seq=16 108x11`, continuing through `seq=32` in one passing run, including frames inside its measured window. `surfaces/panes.rs` gives the drawer `resize_owner = scrollable`, while the pane checks `PaneLayout::is_resize_owner`; that function considers only pane leaves and ignores the drawer. Both appearances therefore claim ownership. This is a product resize-ownership bug, distinct from the observed line-110 setup invalidation. No product fix is included; the test-only frame-worker isolation prevents the unrelated resize feedback from contaminating this renderer test.

The reported `TerminalModel::flush_events` timeout did not reproduce in either investigation batch or the final stress batch. Its implementation is unchanged.

## Results

| Check | Result |
| --- | --- |
| Original, 240 processes, concurrency 24 | 231 pass; 6 line-110 failures; 0 line-272 failures; 0 flush timeouts; 3 empty-burst failures |
| Original with temporary probes, 240 processes, concurrency 24 | 236 pass; 3 line-110 failures; 0 line-272 failures; 0 flush timeouts; 1 empty-burst failure |
| Fixed, 720 processes, concurrency 24 | 720 pass; 0 failures of any kind; 9.1 seconds |
| `cargo test -p runner-app --bin Runner terminal_redraw_tests` | Pass (1 test; also built with `--message-format=json` to locate its executable) |
| `cargo fmt --all --check` | Pass |
| `make clippy` | Pass (workspace/all-targets and updater/all-targets) |
| macOS and Windows PR CI | Tracked in the PR checks after the clean working-tree review |

The process harness used Python `ThreadPoolExecutor(max_workers=24)` to invoke the compiled `Runner` test executable with `--exact terminal_redraw_tests::terminal_output_reuses_split_siblings_and_title_changes_update_sidebar --nocapture`. Each invocation had a 30-second process deadline and separate captured output. Local raw evidence is under `/tmp/terminal-redraw-before`, `/tmp/terminal-redraw-probed`, and `/tmp/terminal-redraw-after`; the harness is `/tmp/terminal-redraw-stress.py`. No load generators or app processes were started.

## Initial investigation from the mission brief

The following is the 2026-10-26 pre-mission account, preserved from the [original brief](https://github.com/yicheng47/runner/blob/15b8eb72f5232f7639125190865b9d5bc821c80e/docs/impls/briefs/terminal-redraw-test-settle.md#problem). Its hypotheses and timeout observations precede the investigation above; the later batches narrowed the render cause and did not reproduce those timeouts.

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
