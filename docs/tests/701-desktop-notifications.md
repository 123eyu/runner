# 701 — desktop notifications validation

## Changed behavior and selected checks

- Saved notification event switches and the Notifications settings pane: settings defaults/round-trip tests and SET-AGENT-01 verify old-file compatibility, navigation and the new controls.
- Direct chat and mission status edges, content and replacement: pure rule/controller tests plus NOTIF-1, NOTIF-5 and NOTIF-6 verify one alert per entered state and the switch defaults.
- Visibility, withdrawal and click routing across windows, tabs, split panes and mission tabs: controller tests, NOTIF-2 through NOTIF-4, NOTIF-7, WS-PANE-02 and MIS-LIFE-02 cover the view-change hooks.
- Live mission questions and answers: rule/controller tests, NOTIF-7 and MIS-ASK-01 cover event subscription and feed routing.
- System notification permissions and shutdown: controller tests and NOTIF-8/NOTIF-9 cover settings links, restored permission and withdrawal on quit.
- Required automated gates: workspace nextest, workspace Clippy with and without updater, formatting and diff whitespace checks cover the cross-platform build and existing tests affected by app startup/settings changes.

## Revised notification scope

Jason initially changed the policy after the prior clean review and recalled QA handoff: all new posts are background-only; chat input/errors stay on and chat completion defaults off; mission notifications have a prominent enable switch off by default, with preserved event choices; per-slot completion is removed in favor of one whole-mission Busy → Idle alert. Pencil defaults and expanded states were updated and saved before product code changed. At 06:54 UTC on 2026-10-13, Jason restored chat completion to ON by default. At 07:36 UTC he finalized four independent, always-visible mission switches, all OFF, removing the mission master, and committed/pushed the design on main as `ce2b8f81`. The current candidate follows these final defaults; coder did not edit or commit the design for either correction.

Selected current automated checks cover foreground suppression on other tabs/routes/windows with no background replay; default background chat completion and its off/withdraw/re-enable behavior; each mission switch independently enabling and withdrawing only its event, with no replay on re-enabling; mission idle baseline/edges/repeats/busy withdrawal and Feed routing. Native NOTIF-1 through NOTIF-9, the final four mission switches and Windows remain for Jason on the PR build. The partial QA section below names the earlier tested candidate and retained evidence; it does not validate the final switch change.

## Final independent mission switch checks

The branch was rebased onto `origin/main` at `ee8f1e62`, including the final design in `ce2b8f81` and #889. Rebase conflicts in the plan index and appended workspace tests were resolved by retaining both crews’ entries and every #889 test. The mission Feed/slot/close/load/asynchronous-removal view hooks remain; #889’s `role_joined`/`role_removed` refresh behavior and dynamic session lists are preserved. A task-only recovery stash was kept, with its SHA and file/patch backup under `target/validation/701/pre-rebase/`; no product commit was made before review.

The old-settings test asserts all four mission choices OFF and no serialized `enabled` key, then round-trips independent ON choices. The controller test turns on each mission kind alone, verifies other kinds stay silent, switches it off to withdraw its notification, and re-enables without replaying. With several kinds posted, disabling one preserves the others. The pane renders all four rows in one card. Chat posting, whole-mission idle edges, click routing and target withdrawal paths are unchanged except removal of the master gate.

The joining-slot regression uses #889’s current session list and persistent mission roster in an isolated fixture. It installs against an idle mission, introduces a new agent slot waiting for approval, verifies the fresh `@buddy` handle and Busy projection, then verifies one mission-idle notification only after the joined slot reports idle. Its unread completion does not create a slot Finished alert. The first post-rebase focused run exited 100 for fixture mismatches: the shared seed now names its slot `agent` rather than its role template, and the new projection fixture needed idle events for both initial handles plus its new member in the persistent roster. These were corrected without changing runtime rules; the failure log is `rebased-notification-tests-initial.log`.

Required post-rebase gates ran on macOS on 2026-10-13. Logs and matching command exit files live under `target/validation/701/`.

| Check | Exit | Result / log |
| --- | --- | --- |
| Focused notification/controller checks | 0 | 17 passed; `rebased-notification-tests.log` |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,291 passed, 6 skipped; `rebased-workspace-nextest.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `rebased-workspace-clippy.log` |
| Same Clippy command with `--features updater` | 0 | `rebased-workspace-clippy-updater.log` |
| `cargo fmt --all --check` | 0 | `rebased-fmt-check.log` |
| `git diff --check` | 0 | `rebased-diff-check.log` |

The reviewer posted `NO REMAINING MUST-FIX ISSUES` on 2026-10-13 at 08:13:55 UTC for the rebased working-tree candidate: HEAD `4ad950710abba8b363160b31f9870b1c656ea01e`, diff SHA-256 `5de2ff88d692d591a9a4fa75708c884c2def3176f8037986b99390e37c8ad787`. Static review confirmed independent mission settings, unchanged notification rules apart from the removed master gate, and preserved #889 integration. Live QA remains Incomplete.

## Three-commit validation

The authorized history was shaped into settings, direct chats and missions on top of `ee8f1e62`. Each step was built and validated separately with the required commands. The completed product files were checked byte-for-byte against the reviewed candidate; only this validation record was extended afterward. Logs and actual exit files use `commit-1-`, `commit-2-` and `commit-3-` under `target/validation/701/`.

| Step | Workspace nextest | Clippy | Clippy with updater | Format | Diff |
| --- | --- | --- | --- | --- | --- |
| Settings | 0; 2,275 passed, 6 skipped | 0 | 0 | 0 | 0 |
| Direct chats | 0; 2,284 passed, 6 skipped | 0 | 0 | 0 | 0 |
| Missions / complete feature | 0; 2,291 passed, 6 skipped | 0 | 0 | 0 | 0 |

The first standalone direct-chat build exited 101 because the split test snapshot retained a mission-only argument and omitted the new active-window field in two fixtures. These test declarations were corrected and the complete step rerun successfully; the failure log is `commit-2-workspace-nextest-initial.log`. The final reviewed product source was restored for the mission step and remained unchanged.

## Earlier checks (chat completion on, mission master)

Run on macOS on 2026-10-13 after the chat completion default was restored to on. Logs live under `target/validation/701/`; each gate's exit code was captured from the command itself in its matching `.exit` file.

| Check | Exit | Result / log |
| --- | --- | --- |
| Focused notification/controller checks | 0 | 16 passed; `quiet-notification-tests.log` |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,269 passed, 6 skipped; `workspace-nextest.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `workspace-clippy.log` |
| Same Clippy command with `--features updater` | 0 | `workspace-clippy-updater.log` |
| `cargo fmt --all --check` | 0 | `fmt-check.log` |
| `git diff --check` | 0 | `diff-check.log` |

The old-settings test now asserts completion on and round-trips an explicit off choice. The completion controller test verifies posting under defaults, withdrawal and suppression when switched off, no replay on re-enabling, and a later fresh completion. The foreground test also uses the default completion choice. The refreshed focused and workspace runs passed on their first attempt; coder launched no development app or real agents.

## Earlier background-only checks (completion off)

Run on macOS on 2026-10-13 against the superseded completion-off candidate. Logs live under `target/validation/701/` and were preserved with the `completion-off-` prefix; each gate's exit code was captured from the command itself in its matching `.exit` file.

| Check | Exit | Result / log |
| --- | --- | --- |
| Focused notification/controller checks | 0 | 16 passed; `completion-off-quiet-notification-tests.log` |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,269 passed, 6 skipped; `completion-off-workspace-nextest.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `completion-off-workspace-clippy.log` |
| Same Clippy command with `--features updater` | 0 | `completion-off-workspace-clippy-updater.log` |
| `cargo fmt --all --check` | 0 | `completion-off-fmt-check.log` |
| `git diff --check` | 0 | `completion-off-diff-check.log` |

New controller coverage checks every event kind while a secondary Runner window is active on Chats, Settings and another mission, consumes those edges without replay after backgrounding, then verifies fresh background edges post. It also verifies completion under the earlier default of off without replay on enabling, mission master gating and withdrawal of slot/question/idle notifications, preserved event choices, whole-mission idle baseline/repeats/None/Busy transitions, and idle clicks selecting Feed even after the posted tag has been withdrawn.

The first revised focused run exited 100 because the asynchronous removal fixture stayed in the background while expecting visibility withdrawal; it now brings the window forward on its slot before removal exposes Feed (`quiet-notification-tests-initial.log`). A later compilation exited 101 for two fixture references missing `settings`; both were corrected (`quiet-notification-tests-compile.log`). The final focused and workspace runs above pass. No development app or real agents were launched by coder.

## Earlier automated checks

These results belong to the superseded candidate, run on macOS on 2026-10-13. Their logs are preserved with the `pre-quiet-` prefix.

| Check | Exit | Result / log |
| --- | --- | --- |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,266 passed, 6 skipped; `pre-quiet-workspace-nextest.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `pre-quiet-workspace-clippy.log` |
| Same Clippy command with `--features updater` | 0 | `pre-quiet-workspace-clippy-updater.log` |
| `cargo fmt --all --check` | 0 | `pre-quiet-fmt-check.log` |
| `git diff --check` | 0 | `pre-quiet-diff-check.log` |

The first full workspace run exited 100 because the existing navigation test expected 12 panes; Notifications adds the thirteenth. Its expected pane count and App ordering were updated, and the failure log is preserved as `workspace-nextest-initial.log`. Earlier focused controller runs exposed test fixture omissions (the test platform requires app identity and persisted valid node IDs); those fixtures were corrected before the passing workspace run. The latest workspace run includes the mission-load view hook, direct-chat routing fallback and both review corrections below.

The new pure and GPUI controller checks cover empty and nonempty startup baselines/new-session edges, repeated refreshes, trigger priority, content, defaults and switches, all withdrawal kinds, shell exclusion, replacement, direct-chat click routing in two windows, visible mission slots, asynchronous slot removal exposing Feed, question/slot routing, live event subscription, delayed mission metadata and cancellation of pending question posts.

## Review corrections

The reviewer reported two must-fix gaps before QA: the loaded empty startup snapshot was not establishing a baseline, and an asynchronous refresh could expose Feed after slot removal without withdrawing its visible question. Two new controller/workspace regressions reproduced both failures (exit 100; `review-regressions-before.log`). Initial installation now always records the loaded snapshot, including an empty map, and the refresh fallback defers a view check after selecting Feed. Both regressions and the other notification checks pass (13 passed, exit 0; `review-regressions-after.log`).

## Live QA — partial run, stopped by Jason

**Verdict: Incomplete.** Jason stopped live QA at 07:36:38 UTC on 2026-10-13 and will test the PR build himself. No further live checks or builds were run after that instruction was read. The final four independent mission switches are **Not run: Jason tests the PR build**. The earlier Settings screenshots show the superseded mission-master design and do not validate the final design. Native notification delivery, content, replacement, suppression, withdrawal, banner clicks, permission recovery and notification withdrawal on quit are not proven by this run.

### Candidate and environment

The tested repository/worktree was `/Users/jason/repos/yicheng47/runner/.worktrees/feat-701-desktop-notifications`, branch `feat/701-desktop-notifications`, base HEAD `1417d8d5219a9afa7d8e11a8c063ff6639f4082a`. The reviewed uncommitted full diff SHA-256 was `3586c2b55a31f2ab82b2192a57499dc9e281296af97c3c1d63beea113cdc2770`; the product-source diff (`git diff HEAD -- crates`) was `91c808a5c24de49f9ebb6ff473b77d7fe528cae1b79de95a05bd64dd554b6273`. These were reverified before cleanup. Coder reported starting the final mission-switch correction at 07:47:24 UTC, after live QA had stopped; no result below describes that new revision. The full diff also changes when this QA record is written.

Environment: macOS 26.6.2, build 25G83, arm64; Rust/Cargo 1.97.1; coordination Runner 0.15.3; tested development app/CLI 0.15.3 (dev); Codex 0.162.1. The bounded fixtures used Codex `gpt-6.1-sol`, low effort. Claude Code 2.1.296 was detected in Settings but was not launched by QA. Native Windows execution was unavailable.

QA used sanitized `make run`, then stopped its bare debug app and launched the procedure's distinct wrapper `/private/tmp/runner-701-qa-live-ha4wynj0/Runner Smoke Dev.app`, bundle ID `com.wycstudios.runner.smoke-dev`. The wrapper app and CLI were byte-identical to this worktree's binaries: app SHA-256 `bdef6138cb7f235b923a942ad578e61c4302726bb611392a76e272060523b1b8`; CLI/installed development sidecar/daemon SHA-256 `61a2677edecd3beee0c82844432b80598899fa9f0d2e531a9069576f550e2234`. The running wrapper executable was observed at that absolute path. Public `status --json` reported the development endpoint `com.wycstudios.runner-dev/runnerd.sock` and mode outside a mission. See `evidence/build-identity.json`, `installed-sidecar-identity.json`, `development-status.json` and `make-run.log`. Compilation succeeded; the make supervisor later exited 2 because QA deliberately stopped its bare app for wrapper launch. No ad-hoc signing was performed.

### Authorization and method

The plan was written before checks in scratch `plan.md`. Scope was NOTIF-1 through NOTIF-9 and the four regression selections in the brief, following the full-smoke procedure's identity, isolation, native-control and cleanup rules. WS-PANE-02 selected only focus switching; its unrelated rename/reopen cases were outside scope. An inline rename was opened, then canceled with Escape without entering or saving a name after Jason corrected the scope.

The 07:07:14 UTC human message released the development namespace for this run. The brief authorized bounded real-agent chats and one disposable mission in a canonical scratch project. The 07:16:18 UTC human message separately authorized stopping and archiving the leftover `aa` mission; it was stopped before notification checks. No other pre-existing mission/session was operated, and the production app was untouched.

Native input and screenshots came from the host computer-use tool. Opening and dismissing New chat verified harmless control before test input. Public development CLI JSON and event feeds supplied structured evidence; no database access or private PTY input was used. Wrapper and CLI children omitted `NO_COLOR` and implementation-crew environment variables. Test data, binary copies, screenshots and configuration snapshots remain outside the repository at `/private/tmp/runner-701-qa-live-ha4wynj0`; paths below are relative to that root.

Computer use could not reach macOS notification UI: the notification-center binding timed out, and the UserNotificationCenter app was rejected by the tool. At 07:23:47 UTC Jason instructed QA not to retry or find another route, reserving NOTIF-1, 2, 5, 8 and 9 for himself. Their retained access limit is **Needs human: macOS notification UI is not reachable by computer use**. At 07:36:38 UTC he canceled all remaining live QA, superseding the proposed Settings/mission-switch recheck. All unfinished portions below are **Not run: Jason tests the PR build**; Skipped denotes that explicit stop, not a pass.

### Result matrix

| Case / macOS variant | Result | Evidence already collected; remaining work |
| --- | --- | --- |
| NOTIF-1: background approval banner | Skipped | Owned chat launched and accepted native input. The harmless approval request completed without a visible approval wait, so even the wait precondition was not established (`evidence/approval-auto-completed.png`, `approval-split-state.json`). Needs human access limit above; banner/focus/permission-race check Not run: Jason tests the PR build. |
| NOTIF-2: native click routing, including second window | Skipped | Own split-pane focus and mission Feed/slot routes were exercised inside Runner; no banner was clicked. Needs human access limit above; native window/tab/pane routing Not run: Jason tests the PR build. |
| NOTIF-3: foreground suppression, no replay, withdrawal | Skipped | Foreground app routes and completed chat states were visible. Background activation and absence/withdrawal of native entries could not be proven; no approval wait was obtained. Not run: Jason tests the PR build. |
| NOTIF-4: unfocused visible pane remains silent | Skipped | The owned chat/shell split and focus changes rendered (`evidence/split-chat-focused.png`, `split-shell-focused.png`). The chat finished without an approval wait; native silence is unverified. Not run: Jason tests the PR build. |
| NOTIF-5: completion/replacement/switch withdrawal | Skipped | Chat completion reached public ready/completed/idle state (`evidence/chat-first-completion.json`); Settings showed chat completion ON. No native entry count, replacement or withdrawal was observed. Needs human access limit above; Not run: Jason tests the PR build. |
| NOTIF-6: switches and whole-mission idle | Skipped | Superseded master UI/defaults and remembered idle OFF choice were observed (`evidence/settings-notifications-defaults.png`, `mission-choice-preserved.png`). Public events show lead returning idle while worker remained busy, then both idle (`mission-activity-events.json`). `mission-one-slot-busy.json` was captured too late and shows both idle; it is not evidence of the intermediate state. No banner/withdrawal/replay conclusion follows. Final four independent mission switches and all native portions Not run: Jason tests the PR build. |
| NOTIF-7: mission question/slot notification | Skipped | One real worker ask → lead escalation → human question → native card answer → lead receipt completed. See MIS-ASK-01. Native question/slot banners, click routes and withdrawal remain unverified. Not run: Jason tests the PR build. |
| NOTIF-8: OS permission and settings link | Skipped | Open System Settings landed on Notifications (`evidence/system-notifications-link.png`). Wrapper notification permission was not toggled; recovery without restart is unverified. Needs human access limit above; remaining permission checks Not run: Jason tests the PR build. |
| NOTIF-9: notification withdrawal on quit | Skipped | Wrapper exited during cleanup, but no native notification inventory could be inspected. Needs human access limit above; native withdrawal Not run: Jason tests the PR build. |
| SET-AGENT-01: selected Settings navigation, tested revision | Passed | All thirteen panes opened; Agents refreshed/scrolled and showed detected versions/defaults/paths; Antigravity's manual Update was absent. `evidence/settings-*.png` and `settings-agents-scrolled.png`. The final revised Notifications pane is Not run: Jason tests the PR build. |
| WS-PANE-02: selected focus switching only | Passed | Own shell focus dimmed chat body and hid inspector; own chat focus restored inspector and dimmed shell, with readable chrome (`evidence/split-shell-panel-hidden.png`, `split-chat-panel-restored.png`). Rename and reopen were excluded; this is not a pass of the full regression row. |
| MIS-LIFE-02: selected Feed/slot switching only | Passed | Feed → worker → lead → Feed showed each slot's own output and question state (`evidence/mission-feed-start.png`, `mission-worker-start.png`, `mission-lead-start.png`, `mission-question-pending.png`). Full reopen/reattach coverage was not selected/completed. |
| MIS-ASK-01: worker/lead/human loop | Passed | Exactly one ask_lead, ask_human, human_question and human_response (`evidence/ask-flow-events.json`); pending count 1 → 0 (`mission-question-state.json`, `mission-after-answer.json`). Blue resolved the native card; lead terminal received `[human_response] blue` and `ACK blue` (`mission-question-answered.png`, `lead-answer-receipt.png`). |
| Windows native toasts/click/taskbar grouping | Skipped | Previously Blocked because no Windows host was available. No Windows native result exists. Not run: Jason tests the PR build. |

No notification regression was established. The approval fixture limitation and native access failures remain in the evidence; they were not converted to passes. A command-palette attempt from the mission composer submitted the harmless text `codex-approval` to the owned mission instead; that extra bounded turn is preserved in its feed and does not count as routing verification.

### Cleanup and retained evidence

Two owned Codex chats (`01M4T1QMQEM52AM53GNH8R3X73`, `01M4T2G1WDF42XNFE2C2AHTSZF`) were stopped and archived. The owned shell `01M4T2J7RD77RSDX19T632FPK2` was stopped; archive rejected it with “terminals close rather than archive,” and a generic close-tool attempt was unavailable. Its stopped metadata is retained; its process is gone. The disposable mission `01M4T27PCF3MFVSPJ6GABV23QN` and explicitly authorized leftover `aa` (`01M4T13HR8B0GNHQ3NPGR830P6`) were stopped and archived; final public state shows zero live sessions and non-empty archive timestamps. See `evidence/cleanup-after-*.json` and `ledger.json`.

The development daemon had restarted by cleanup: original PID 69333 was gone and replacement PID 50686 belonged to the QA wrapper. The UI subsequently displayed “Background service restarted · 5 sessions stopped” (`evidence/cleanup-service-restarted.png`). This unexpected observation was not reproduced or attributed to the notification diff, because Jason stopped testing. Cleanup still explicitly stopped all ledger sessions and the owned replacement daemon.

QA restored the tested candidate's notification choices (chats all ON, superseded mission master OFF, remembered event choices ON) and restored inspector OFF, preserving Jason's intervening preference change (`evidence/cleanup-restored-settings.json`, `cleanup-notifications-restored.png`). Only the three scratch-folder Codex trust entries were removed. The cleanup diff changes no other configuration values, and the parsed configuration equals the prelaunch baseline (`evidence/cleanup-codex-trust.diff`, `cleanup-codex-config-summary.json`). Authentication and permission settings were unchanged.

The wrapper quit, its supervisor completed with exit 0, its LaunchServices registration was removed, and the owned development daemon stopped. `evidence/cleanup-final-processes.json` records the 07:48:10 UTC `ps` inventory with no surviving owned app, daemon, agent, shell or CLI helper processes; System Settings was also no longer running. Production Runner and its daemon were not stopped. No background watcher/load loop was started.

The temporary project, two roles, crew, stopped shell metadata, archived histories and scratch evidence remain. Roles/crews have no archive verb and were retained under the procedure to preserve mission/session records; none was deleted. `ledger.json` contains every created ID and cleanup action. Raw screenshots/config snapshots may contain unrelated local UI details and remain private scratch evidence; they are not committed. The binary wrapper files remain for audit. QA made no product/test edits, commits, pushes or PR.
