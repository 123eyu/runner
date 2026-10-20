# #962 mission feed validation

This change replaces the mission feed's eager element tree with a virtual GPUI list, updates grouped blocks on ordinary tail appends, and caches parsed markdown per event. The scope is feed rendering, scroll anchoring, ask-card height changes and selection persistence. It does not change agent runtimes, the daemon or session lifecycle.

## Selected checks

- Compare the incremental projection with `group_feed_blocks` after each append, including grouping boundaries, hidden signals, relayed human signals and out-of-order delivery, because projection drift would hide or duplicate feed content.
- Verify duplicate suppression and sorted insertion because event delivery can overlap replay.
- Use a 2,000-message fixture and test-only parse/build counters to verify that warm rendering does not parse old markdown and that typing builds only visible rows. These assert work counts rather than elapsed time.
- Exercise GPUI list item counts, follow-tail, scrolled-up appends, the new-messages pill, human appends, ask-card invalidation, mission switches and selection persistence because these rely on the replaced scroller's state.
- Run the `runner-app` tests, workspace Clippy, macOS updater Clippy, formatting and whitespace checks as required by the mission brief. macOS and Windows CI remain required after review.

## Live scope

If a QA slot joins, run the brief's owned development-mission checks for smooth scrolling and typing, bottom anchoring and the new-messages pill, drag selection and copy after virtualization, ask-card resizing and window reflow. Select MIS-FEED-01, MIS-FEED-02 and MIS-LIFE-02 from `docs/tests/regression/missions.md` because feed content, pending state, dividers and tab-switch replay are affected. The coder and reviewer do not launch the development app. Automated checks do not establish a native live pass.

## Results

Local gates passed on 2026-10-20. The complete `runner-app` test command passed 770 tests before rebase and 771 after rebasing onto `origin/main` at `71b76025`, with three existing opt-in smoke/benchmark tests ignored. The focused feed tests also cover the empty-mission panel filling the remaining viewport and shrinking when content arrives.

| Command | Exit code |
| --- | ---: |
| `cargo test --locked -p runner-app --profile ci feed_ -- --nocapture` | 0 |
| `cargo test --locked -p runner-app --profile ci` | 0 |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 |
| `cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings` | 0 |
| `cargo fmt --all --check` | 0 |
| `git diff --check` | 0 |

The reviewer reported `NO REMAINING MUST-FIX ISSUES` on 2026-10-20 after re-reviewing the complete working-tree diff. macOS/Windows CI results are tracked by the pull request checks. QA joined after the clean working-tree review; the incomplete native checkpoint below belongs to the exact pushed candidate.

## Timing sample

Throwaway measurement on 2026-10-20, Apple M5 Pro, macOS 26.6.2, Cargo `ci` profile (workspace unoptimized, dependencies at opt-level 1). The existing GPUI test harness held 2,000 alternating-author messages with paragraphs, lists and fenced code in a 600 × 700 surface. The pre-change eager renderer was temporarily restored from `a9d3b290` for comparison with the virtual renderer. Ten warm redraws were averaged; append and keystroke are single samples. The temporary timing code was removed and is not a test gate.

| Operation | Eager | Virtual |
| --- | ---: | ---: |
| Build feed element tree (before layout) | 35.696 ms | 0.002595 ms |
| Redraw including layout | 372.846 ms | 0.586 ms |
| One append plus redraw | 368.842 ms | 0.593 ms |
| One composer keystroke plus redraw | 752.307 ms | 1.175 ms |

The virtual feed built six visible blocks and parsed zero markdown on that keystroke. The eager renderer built 4,000 blocks across its redraws. This does not justify splitting the composer into a separate entity. Session-status, ask and goal projections remain full projections as the brief permits. The timings describe the synthetic fixture, not a native live pass.

The first throwaway timing attempt built elements outside a GPUI draw and exited 101 on the harness's leaked-element-handle assertion after collecting timings. It was replaced with the actual draw-based comparison above, which exited 0.

## Working-tree review follow-up

The first review identified three failures that the initial tests missed: zero list overdraw prevented upward wheel scrolling into unmeasured history; GPUI ignored horizontal list padding; and its scrollbar maximum omitted vertical padding, making the near-bottom threshold too generous. The fixes use 200px of bounded overdraw, put horizontal padding on the enclosing div, and derive the end distance from the last row plus both vertical paddings at the current zoom.

New regressions use real upward wheel events on the 2,000-message fixture, assert the actual list and message-group bounds at the 24px side inset, and check both 100px and 40px from the bottom at 100% and 150% zoom. The focused feed command passed after these fixes. The timing table above was refreshed against the corrected implementation; that throwaway run exited 0. The earlier working-tree review was not a clean verdict. Re-review resolved all three findings and reported `NO REMAINING MUST-FIX ISSUES` before PR mode.

## QA checkpoint — 2026-10-20, candidate `17dac27`, incomplete

The human authorized the brief's focused native checks, paused the first run, and explicitly resumed QA at approximately 12:16 China time. At 12:18 the coder relayed the human's expansion to #947 and requested a safe stop before changing the candidate. This checkpoint is **Incomplete**, with no overall live pass and no established product regression. The replacement candidate needs the complete selected live checks.

Candidate: `/Users/jason/repos/yicheng47/runner/.worktrees/fix-962-virtualize-mission-feed`, branch `fix/962-virtualize-mission-feed`, HEAD `17dac27e85545e9278600d3c2e81e9c1c5447703`, base `71b76025a9b0b834782f5997d18eacd63bdcfdda`. The tree was clean at initial build and resume; empty diff SHA-256 was `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. `make run` successfully built the debug app; its raw process was deliberately terminated to use the procedure's byte-identical native wrapper, making the make process exit 2 after SIGTERM. The retained wrapper is `/private/tmp/runner-962-qa.c1vsm083/Runner 962 QA.app`, bundle ID `com.wycstudios.runner.qa962`. App SHA-256: `fbc3c052008b51f7173fd67aff1259ab3682855e66a02ae93b94ed1972a391ce`. Source CLI, installed development CLI and daemon-reported executable SHA-256 match `5a42b20d4ab51c46d2830319ca77c80ac6af3df10f9497cab7f957e99a7a6656`. `status --json` reports `0.16.6 (dev)` and `com.wycstudios.runner-dev/runnerd.sock`. Identity was rechecked before resumed UI actions.

Environment: Apple arm64, macOS 26.6.2 (25G83), rustc 1.97.1, Cargo 1.97.1, GNU Make 3.81, Codex CLI 0.163.0. The first owned role used the candidate catalog's `gpt-5.4-mini`, low effort; its terminal displayed a retirement notice before executing the task. It was stopped without accepting a global model change. A second owned role explicitly selected `gpt-6-luna`, low effort, and executed the bounded task. This is a fixture setup limitation without attribution to #962. Windows native testing is outside this assigned run; the coder reported green macOS/Windows CI separately.

Method: followed the brief and the full-smoke procedure's identity, isolation, native control and cleanup rules. External product CLI actions used the absolute development CLI with `RUNNER_CREW_ID`, `RUNNER_MISSION_ID`, `RUNNER_HANDLE` and `RUNNER_EVENT_LOG` removed. App launches also omitted those variables and `NO_COLOR`. Native control bound to the wrapper's distinct bundle ID; New chat was visibly opened and dismissed with Cancel before trusting UI actions. GPUI exposed window/menu accessibility nodes, so content observations use native screenshots. No database was opened.

Evidence root: `/private/tmp/runner-962-qa.c1vsm083/evidence`; the parent scratch root retains `plan.md`, `ledger.json`, the CLI driver, seed script and private configuration snapshots. Owned mission `01M5BJZ8NN400WDR2BJN8MW2JH` uses canonical `/private/tmp/runner-962-qa.c1vsm083/mission`. The public development CLI posted 1,500 mixed messages, markers `QA962-0000` through `QA962-1499`, with 1,500 distinct returned event IDs. No agent turns seeded them. Public feed captures contain the first 500 events: the CLI caps each page at 500 and pagination was unfinished at the checkpoint. Native screenshots show first and final markers. Same-author messages form a long group; this fixture alone does not establish performance with 1,500 independent blocks. An attempted public `mission_goal` section signal was refused and not bypassed; no section signal was created. The Luna fixture posted two neighbouring messages and an actual ask, question ID `01M5BRH5FPE1CXGM8B6AW6WET9`. This host has no supported background push watcher; its temporary foreground follower was stopped and public snapshots were used for the bounded fixture.

| Case | Steps and observations | Classification | Evidence paths relative to evidence root |
| --- | --- | --- | --- |
| Identity and native control | Rechecked SHA/clean tree, wrapper/source hashes, development endpoint and daemon hash; opened New chat and clicked Cancel. | Passed | `candidate.txt`, `build-identity.json`, `resume-status.json`, `resume-daemon.json`, `native-new-chat.png` |
| Seed fixture | Posted 1,500 mixed messages through the public CLI with distinct returned IDs; native first/final markers are visible. Full paginated readback remains to capture. | Passed | `cli-actions.jsonl`, `seed-summary.json`, `seed-feed.json`, `checkpoint-summary.json` |
| Initial landing at bottom | Opened the owned mission through native search; final messages 1497–1499 and the trailing empty-mission panel were visible. | Passed | `open-long-feed.png` |
| Upward wheel input into old history | Scrolled upward three pages to messages 1470–1475, then used large wheel input into the first group, showing 0000–0006. No blank feed or stalled command was observed. This does not establish frame rate or the complete smooth-scroll case. | Passed | `wheel-up-older-history.png`, `wheel-deep-history.png` |
| Complete top-to-bottom smooth scrolling and typing | Upward traversal observed; return traversal and composer typing unfinished at the requested stop. | Skipped | `wheel-up-older-history.png`, `wheel-deep-history.png` |
| Bottom/near-bottom follow and pill/manual-bottom actions | New messages pill appeared while older messages remained visible after the slot appended. Controlled anchor comparison, bottom follow and pill/manual-bottom actions remain untested. | Skipped | `wheel-deep-history.png`, `checkpoint-mission-stopped-slots.json` |
| Selection and Cmd-C persistence | Not started before replacement-candidate request. | Skipped | No case evidence |
| Ask resize and window reflow | One actual pending ask exists; answering and native resize were not started. | Skipped | `checkpoint-mission-stopped-slots.json` |
| 40px side inset | No native measurement completed. | Skipped | No case evidence |
| MIS-FEED-01 | Mixed content/authors rendered and public state contains a pending ask. Persistence across switches and hidden-event filter checks remain unfinished. | Skipped | `open-long-feed.png`, `wheel-deep-history.png`, `checkpoint-mission-stopped-slots.json` |
| MIS-LIFE-02 | Switched Feed and the first owned slot, which showed a retired-model notice. Luna reached structured Idle with a conversation key; output/history/status and replay-duplicate assertions remain unfinished. | Skipped | `probe-startup-terminal.png`, `checkpoint-mission-stopped-slots.json`, `resume-cli-actions.jsonl` |
| MIS-FEED-02 | Both owned slots stopped for checkpoint. Archive/divider assertions remain untested; fixture retained for reuse. | Skipped | `checkpoint-mission-stopped-slots.json` |

Cleanup: owned sessions `01M5BRFGEPGN9DTZHFYT1M5NRK` and `01M5BRGNW6J6P1KHDYJ9JJM15Q` are stopped; public mission state reports zero live sessions. The owned app/development daemon stopped, the wrapper was unregistered from LaunchServices, and process inventory confirms no owned app, daemon, seed helper, follower or Codex PID survives. Only the new trust entry for the owned mission directory was removed; parsed Codex configuration after cleanup equals the prelaunch configuration. The mission, pending ask, two role definitions (`qa962-probe`, `qa962-luna`), conversations and scratch evidence remain for reuse. No pre-existing session was operated; no role, crew, branch, worktree, conversation or scratch artifact was deleted. Cleanup evidence: `checkpoint-mission-stopped-slots.json`, `checkpoint-stopped-processes.json`, `checkpoint-process-inventory.json`, `codex-trust-cleanup.diff`, `codex-cleanup-summary.json`.

## Added scope — #947 mission feed scrollbar

Jason folded #947 into this PR on 2026-10-20. QA released `17dac27e` before edits; the checkpoint above remains incomplete evidence for that earlier candidate. The added feature reuses the shared app scrollbar and keeps its existing visuals and API. The spec is `docs/features/947-mission-feed-scrollbar.md` and the guide change is `docs/guide/concepts/missions.md`.

Selected checks connect the new behavior to the virtual list: fitting content must hide the scrollbar; a long history needs estimated height without building off-screen rows; track clicks and dragging must reach unmeasured history; gutter wheel events must move the list; drag metrics must stay frozen as row heights are discovered, and release outside the gutter must unfreeze them. The harness also checks the 10px gutter, unchanged content insets, resize recovery of height hints, preserved selection, scrolled-up append/pill behavior, returning to the bottom and mission reset. The previous #962 work-count and scroll tests remain in scope. Live QA adds scrollbar track/drag/gutter-wheel checks and independent-block coverage to the existing selected cases and retained same-author fixture.

The first focused #947 run caught missing visible scrollbar state; GPUI discards height hints during initial/width layout, and metrics notifications must follow painting. The adapter now restores hints after width changes and updates the cached scrollbar metrics after paint. The focused command `cargo test --locked -p runner-app --profile ci feed_ -- --nocapture` then exited 0 (16 app tests and one matching bundle test). The complete `runner-app` test command also exited 0 (773 tests passed, three existing opt-in tests ignored). Both workspace Clippy commands listed in Results, including updater, exited 0; formatting and whitespace checks exited 0. The added-scope review and follow-up are recorded below. Earlier green CI belongs to `17dac27e`; the expanded candidate still needs macOS/Windows CI and live QA.

A fresh throwaway comparison including the scrollbar exited 0 on the same Apple M5 Pro/macOS 26.6.2, Cargo `ci`, 2,000 mixed-markdown message fixture and methodology described above. Eager versus virtual-with-scrollbar: element-tree build 36.453ms versus 0.004108ms; redraw including layout 373.085ms versus 0.591ms; append plus redraw 375.999ms versus 1.192ms; composer keystroke 755.606ms versus 1.077ms. The keystroke built six visible blocks and parsed zero markdown. Timing code was removed; it is not a gate and does not replace native QA.

### Scrollbar review follow-up

The first added-scope review found that GPUI splices give appended or replaced off-screen rows no height hint, excluding that tail from scrollbar navigation. Changed rows now use `remeasure_items`, retaining their previous height estimate and scroll anchor. Count-changing splices mark hints for repair; the next scrollbar mouse-down or gutter-wheel gesture repairs them and refreshes metrics before the shared control handles the event. The full hint traversal happens once at that interaction boundary, without adding a full-feed traversal to ordinary appends or composer edits.

A new regression starts with 50 blocks, scrolls to the top, grows the measured off-screen last group without losing its height estimate, then appends 1,000 alternating-author blocks while preserving the viewport and pill. Real track-click and thumb-drag events reach the newly appended region while building fewer than 100 blocks for the track jump. The focused feed command exited 0 (17 app tests plus one matching bundle test); the complete `runner-app` command exited 0 (774 passed, three existing ignored). Workspace Clippy, updater Clippy, formatting and whitespace commands in Results all exited 0 again. The reviewer independently reran focused feed tests, formatting and whitespace checks (all exit 0), reviewed the expanded working-tree diff and reported `NO REMAINING MUST-FIX ISSUES` on 2026-10-20. Publication and the replacement exact-SHA QA handoff follow that clean verdict.

The reviewed history rebased without conflicts onto `origin/main` at `f6ab9ef6` (workspace version 0.16.7). The complete `runner-app` command passed again with 773 tests and three existing ignored; main removed one unrelated sidebar test. Workspace Clippy, updater Clippy, formatting and whitespace checks all exited 0 after rebase. Expanded-candidate CI and native QA remain pending at publication.

## Replacement-candidate QA checkpoint — `0f454bd`, incomplete

Native QA resumed against the coder's frozen reviewed/pushed candidate `0f454bd1849d2ccc1525757287d757e127e53615`, based on `f6ab9ef6a10db72d21013014abb251b968bd3c50`, on the same feature branch/worktree. Both initial and post-build git status were clean; empty diff SHA-256 remained `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. Environment remained macOS 26.6.2 arm64, rustc/Cargo 1.97.1 and Codex 0.163.0; runtime was the retained owned `gpt-6-luna` low slot. Local gate evidence belongs to the coder; expanded CI was pending at handoff.

`make run` successfully built Runner `0.16.7 (dev)` (CLI build 11.54s, app build 18.24s); make later exited 2 when its raw app was deliberately stopped for the native wrapper. New evidence lives under `/private/tmp/runner-947-qa.tie9pz1_/evidence`, with the plan/ledger and private prelaunch configuration snapshots in its parent. New native wrapper: `/private/tmp/runner-947-qa.tie9pz1_/Runner 947 QA.app`, bundle ID `com.wycstudios.runner.qa947`. Source/wrapper app SHA-256: `b002eaf9efa8e29dc88c625da84ee7cff88671db9bfa5587f12b8847913724b5`. Source/installed CLI and daemon SHA-256: `21ab0a79c2878abae819cd58fa7e8b0a24ef7b8eac43c20616b8c2e2a519f42c`. Public status confirmed the development namespace/socket before native checks. External app/CLI launches stripped the implementation mission environment and `NO_COLOR` as in the earlier run. No database access or product/test code edits occurred.

The retained owned long-feed mission and Luna session were reused. A new fitting-content mission, `01M5BSW5W5G7KA25D7JV704TE8`, uses `/private/tmp/runner-947-qa.tie9pz1_/fitting`; no agent was added to it. The source procedure's native binding was revalidated with menu open/cancel and a visible zoom reset. Saved zoom was 130%; the layout checks used 100%, with restoration still due at cleanup. Public feed pagination captured all 1,533 initial events, including exactly 1,500 seeded markers. These messages contain 107,723 characters in total; the largest individual message in that snapshot is 686 characters.

| Case | Steps and observations | Classification | Evidence paths relative to the new evidence root |
| --- | --- | --- | --- |
| Candidate identity/native binding | Verified commit/diff, byte-identical wrapper, source/installed CLI and daemon hashes, development endpoint and visible native zoom change. | Passed | `candidate.txt`, `build-identity.json`, `status.json`, `daemon-status.json`, `process-identity.txt`, `initial-100-percent.png` |
| Full retained fixture readback | Used public feed `--since` pagination in 500-event pages; 1,533 initial events have distinct IDs and include all 1,500 seeded markers. | Passed | `initial-full-feed.json`, `pre-fix-checkpoint-summary.json` |
| Fitting-content scrollbar | Opened the owned small mission; started divider, short goal and no-role panel fit with no feed thumb/track visible. | Passed | `fitting-no-scrollbar.png` |
| Pending card across owned routes | Opened fitting mission, returned to long mission; pending question, choices, authors and neighbouring text remained visible. | Passed | `long-feed-bottom.png`, `pending-after-route-switch.png` |
| Native terminal reattachment | Opened retained Luna tab; saved ACK/history, input cursor and runtime/model/cwd rendered; card showed Idle. This is a bounded observation, not a complete MIS-LIFE-02 pass. | Passed | `luna-resumed-terminal.png`, `turn-state.json` |
| Complete smooth scrolling/typing | The coder relayed human feedback that the feed still feels slow, specifically with large post blocks. No controlled active-frame measurement or typing latency evidence was completed. The performance issue remains open. | Failed | `947-long-feed-lag-diagnosis.md`, `947-long-feed-lag.sample.txt`, Runner coder message 2026-10-20T04:44:36Z |
| Independent-block append and scroll anchoring | Prepared a bounded real-slot CLI batch and entered it through native terminal input, but submission was not established and the script did not execute. The draft was cancelled with native Ctrl-C before human handover. No append/pill pass is claimed. | Skipped | `bounded-turn-input.png`, `append-anchor-before.png`, `human-handoff.json` |
| Track/drag/gutter wheel, outside release, bottom/pill/manual bottom, selection/copy, answer remeasurement/window reflow, complete MIS-FEED-01/MIS-FEED-02/MIS-LIFE-02 | Unfinished when the human took over perceived-smoothness testing and the coder requested release for the finer-grained rendering fix. | Skipped | `human-handoff.json`, `pre-fix-checkpoint-mission.json` |

The coder's read-only source/fixture diagnosis establishes an unbounded rendering path: one `MessageGroup` holds all 1,500 seeded messages; a visible virtual row clones the complete group and builds every event's markdown elements. One large individual event similarly builds every parsed markdown block. The alternating-author automated fixture does not cover these cases. The five-second sample caught the main thread waiting for events, so it does not establish active-frame CPU attribution or latency. Keep the human-reported failure and this distinction: finer presentation granularity is required, but the relative contribution to each perceived slowdown is not quantified. The next candidate must add both same-author-group and single-large-post coverage and rerun the selected live checks.

Checkpoint status is **Incomplete**, with no live performance pass. The human has the computer; QA requested confirmation that the manual trial is finished before stopping the app they are using. At this checkpoint the owned app PID 27362, development daemon PID 23436 and the retained Luna session intentionally remain available for that trial. No load/append helper or feed follower is running. Cleanup, zoom restoration, owned trust-entry removal and explicit worktree release remain pending and must be recorded before a coder fix/build.

## Finer presentation rows — manual iteration

Jason removed the QA slot and requested continued iteration until the feed feels right. Before this follow-up the coder confirmed that the old QA app PID 27362 was absent and no QA app/build helper was running. The incomplete checkpoints above remain historical records; no subsequent QA pass is implied, and the broader cleanup items remain unverified. Attempts stay uncommitted until Jason confirms the native behavior.

Changed behavior and selected checks: canonical feed groups remain intact, while the virtual list now indexes individual messages and sections of up to four parsed markdown blocks. This removes whole-group cloning and prevents a post containing many markdown blocks from building them all on each draw. A separate block-to-row map keeps ask invalidation correct; fallback rebuilding anchors to event ID plus section index. Markdown spacing and selection offsets share the full renderer’s layout calculation. Tests cover a 2,000-message same-author group, a 2,000-paragraph single post, one-key markdown-build counts, parse reuse, append/pill anchoring and copy across sections after scrolling away and back. Existing scrollbar, grouping, row-size, zoom and selection tests remain selected because the presentation index changed. The complete app tests, both Clippy commands, formatting and whitespace gates are required again before the next manual candidate.

Parsing and section metadata are cached once per event when preparing presentation rows, including off-screen messages, because their section counts determine list indices and estimates. This adds initial projection work but removes the unbounded element-building path. The timing sample includes projection and first draw rather than hiding that initial cost.

Throwaway comparison against `0f454bd1` on the same Apple M5 Pro/macOS 26.6.2, Cargo `ci` profile and 600 × 700 existing headless harness: 2,000 same-author mixed-markdown messages, or a single post with 2,000 paragraphs. Ten warm redraws were averaged; one-key input is a single sample. Both comparison commands exited 0, and modified production/test files were restored byte-for-byte with no timing code retained. No native app was launched or driven.

| Fixture and operation | Previous virtual group row | Finer presentation rows |
| --- | ---: | ---: |
| Same-author group: prepare projection | 1.893 ms | 5.950 ms |
| Same-author group: first draw | 524.382 ms | 3.432 ms |
| Same-author group: warm redraw | 171.134 ms | 0.622 ms |
| Same-author group: one key | 359.147 ms | 1.194 ms |
| Single long post: prepare projection | 0.937 ms | 1.555 ms |
| Single long post: first draw | 122.287 ms | 2.130 ms |
| Single long post: warm redraw | 37.421 ms | 0.556 ms |
| Single long post: one key | 73.740 ms | 1.093 ms |

Warm redraws built 8,000 markdown blocks for the old same-author group and 2,000 for the old single post, versus 20 in each new case. One key built 16,000/4,000 respectively, versus 40 in each new case. Warm parsing was zero throughout. Logs: `/tmp/962-fine-timing-previous.log` and `/tmp/962-fine-timing-fine.log`; the throwaway driver is `/tmp/962-fine-feed-timing.py`. These samples quantify the confirmed rendering path; they do not establish native smoothness or a complete live pass. An individual markdown block, such as a long fenced code block or one unbroken paragraph, remains a single layout unit in this iteration.

The focused feed command passed 19 app tests and one matching bundle test. The complete `cargo test --locked -p runner-app --profile ci` command exited 0 (775 passed, three existing ignored). Both workspace Clippy commands listed in Results exited 0; formatting and whitespace checks exited 0. `cargo build --locked -p runner-app` exited 0 and produced `target/debug/Runner` (9.82s incremental build). Logs are `/tmp/962-fine-{feed-tests,app-tests,clippy,clippy-updater,build}.log`. The attempt remains uncommitted for working-tree review and Jason’s manual trial; the coder did not launch the native app. Separately, macOS and Windows CI passed for the previously pushed `0f454bd1` in [run 39370757087](https://github.com/yicheng47/runner/actions/runs/39370757087); those checks do not cover this uncommitted follow-up.

### Presentation review follow-up

The first review of the finer rows identified two must-fix issues. The continuation spacer used 35 physical pixels while the actual avatar scales in rems, shifting text at non-default zoom. It now uses the avatar’s rem width; a 150% test compares both another message and a later section of the same post with the header row’s text x-position and width.

Four parsed blocks also did not bound the paragraph and table child loops: one paragraph can contain thousands of newline-separated lines, and one table can contain thousands of rows. Presentation sections now contain up to four pieces: a paragraph line, a table row, or an atomic block. Paragraph byte ranges, global selection indices and table-row cell offsets are prepared once. Table row pieces retain one shared horizontal scroll handle and consistent table width, with the header rendered once. No paragraph-line or table-row prefix scan runs per visible row.

The selected regressions now include a 2,000-line single paragraph and a 2,000-row two-column table. A counter at the prepared-text-line renderer counts actual lines/cells, including all table cells, so an unbounded table cannot pass by reporting one markdown block. Both cases exercise one-key work counts, parse reuse and selection/copy after scrolling away and back. Table checks additionally exercise real horizontal wheel input, alignment across visible rows, horizontal position persistence after vertical virtualization and clamping when the window widens. The focused command passed 21 app tests and one matching bundle test (exit 0). Full gates, timing refresh and re-review follow this fix. The earlier four-block iteration and its timings remain historical evidence rather than a clean review verdict.

The refreshed throwaway comparison uses the same complete mission-workspace harness and matched fixtures on both `0f454bd1` and the corrected working tree. Both commands exited 0 and all temporary source changes were restored byte-for-byte. The reviewer’s smaller standalone probe had different absolute timings; the following table compares like-for-like workspace runs on the same machine/profile/method described above.

| Fixture | Previous warm redraw | Corrected warm redraw | Previous key | Corrected key | Previous/new lines or cells built per key |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2,000 same-author messages | 155.960 ms | 0.671 ms | 313.371 ms | 1.279 ms | 16,000 / 40 |
| One post, 2,000 paragraphs | 34.298 ms | 0.441 ms | 68.837 ms | 0.803 ms | 4,000 / 40 |
| One paragraph, 2,000 lines | 18.937 ms | 0.528 ms | 38.261 ms | 1.020 ms | 4,000 / 56 |
| One table, 2,000 two-cell body rows | 76.411 ms | 1.108 ms | 154.580 ms | 2.158 ms | 8,004 / 84 |

Corrected projection preparation was 7.631/1.700/1.177/1.264ms for these fixtures; first draw was 3.703/1.704/2.108/4.153ms, respectively. Warm parsing remained zero. Logs: `/tmp/962-pieces-timing-{previous,fine}.log`; throwaway driver: `/tmp/962-pieces-feed-timing.py`. These are synthetic measurements, not a native pass.

After the fixes, `cargo test --locked -p runner-app --profile ci` exited 0 (777 passed, three existing ignored), including an ask appended after a split post whose presentation-row index differs from its canonical block index. Workspace Clippy and updater Clippy commands from Results exited 0, as did formatting and whitespace checks. `cargo build --locked -p runner-app` exited 0 (7.24s incremental build); the manual candidate is `target/debug/Runner`. Logs: `/tmp/962-pieces-{feed-tests,app-tests,clippy,clippy-updater,build}.log`.

The reviewer reported `NO REMAINING MUST-FIX ISSUES` for working-tree diff `de4f74849338fcee2dced05778830b3c0f19e6d01f6f285456e134d0e98fb2d9` on 2026-10-20. Independent focused feed tests passed 21 app tests plus one bundle test; formatting and whitespace checks exited 0. A standalone probe importing the production markdown renderer verified mixed Unicode/CRLF paragraph, heading, table, list and footer layout parity against the full renderer at widths 220/520 and zoom 100%/150%, including total height and table-row positions/sizes. It also verified spacer alignment and the previously failing compound-loop fixtures’ warm redraw improvement. Probe: `/tmp/962-pieces-review-probe.rs`; gate log: `/tmp/962-pieces-reviewer-feed-tests.log`. These headless checks do not establish a native performance pass. The reviewed debug binary SHA-256 is `9cc7026211fc1512cc9e7686178d7643c9c91de7ec59df99ce0100cd84928c0c`.

Jason subsequently reported “visual smoothness passed” and confirmed that no QA slot was needed. This accepts the native performance behavior after the reviewed build was provided; it does not turn the earlier incomplete QA checkpoints into complete functional or regression passes. Jason authorized merging after CI passes on the final published candidate. The accepted changes can now be folded into the mission commit and pushed; final-candidate macOS and Windows CI remain required before merge.
