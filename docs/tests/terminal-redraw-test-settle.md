# Terminal redraw test settling

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
