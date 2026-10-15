# 562 mission 4 — start empty

## Scope and candidate

Phase 3 of [the implementation plan](../impls/562-mission-spawn/plan.md#phase-3--start-empty), including [#849's Start mission dialog](../features/849-start-mission-modal.md). Worktree `/Users/jason/repos/yicheng47/runner/.worktrees/feat-562-start-empty`, branch `feat/562-start-empty`. Working-tree review, frozen-candidate QA, five commits, then open PR and macOS/Windows CI; no merge or release is authorized. Native Windows is unverified.

Current selected macOS verdict: **Incomplete — 11 Passed, 0 Failed, 1 Blocked** after the [affected rerun](#second-native-run--affected-checks). UPG-1 pre-existing-session resume remains Blocked at Jason’s instruction; native Windows is unverified. The first run’s failures and every retry remain below as history. Both working-tree reviews reported **NO REMAINING MUST-FIX ISSUES**.

The changed behaviors are the version 27→28 database upgrade, optional crew provenance and mission paths/envelopes/environment, empty start and lead-less coordination, crew deletion retaining its missions, removal of `ask_lead` sends, and Start mission / empty workspace / composer / mission info. Existing crew launches and their prompts are regression gates; their only coordination copy changes are Phase 3 decision 16's two lines and the removed signal in the allowed-signal list.

Use [full-smoke setup and isolation](full-smoke-test.md) for candidate identity, development data, account/configuration handling, evidence and cleanup. Select only the cases below, not the entire suite. `qa` owns live execution and its verdict; coder and reviewer do not launch the app. The development app is authorized, including its automatic saved-chat resume, but pre-existing sessions must never be operated by QA. Block if another person's development app or daemon is running. Use the Phase 2 plan's two #881 lessons. Archive only owned fixtures and stop every owned app, daemon, agent and helper when finished.

## Selected checks and reasons

| Check | Required procedure and evidence | Reason |
| --- | --- | --- |
| UPG-1 | Before launching, copy the development database and count crews, slots, roles, missions, sessions, sessions with a mission, and attention rows using read-only queries on the copy. Launch the candidate; repeat on a fresh copy. Counts match and saved chats/missions resume. | The migration rebuilds four tables, including both parents of existing session links. |
| EMPTY-1 | Native No crew start: goal visible without a target, empty state and Add role in feed/rail, no paused overlay, composer disabled. Add one role with task and verify goal/task/first post; add a second and prove untargeted composer delivery to both and direct messages in both directions through covering reads and ACKs. | Main new empty-start and lead-less workflow. |
| EMPTY-2 | Keep an empty mission and a mission with one added role, quit development app, stop development daemon, relaunch identical candidate. Empty mission reopens; member resumes in the populated mission. | Empty and lead-less mounting and persistence. |
| DELEG-1 | In an owned direct chat, start with `runner mission start --title … -q` and add with `runner mission add-role … --task-file -`. Record sidebar update without focus change. | The embedded help teaches chat-driven delegation. |
| CREW-1 | Delete an owned crew while its mission runs; verify posting, adding roles, `runner mission show`, retained directory and absent Crew info row. | Crew deletion now preserves missions and their original directory provenance. |
| ASK-1 | Refuse `runner ask "question"` and `runner signal ask_lead` with the prescribed hint, preserve log length, send `runner ask --human`, answer it, and open an owned historical `ask_lead` event. | Removed send path must preserve human asks and old feed rendering. |
| MIS-CREW-01 | Existing regression through the redesigned dialog, including crew preselection, changing crew/roster, five slots with overrides, empty crew refusal, failed-start error, keyboard confirm and 640 × 480 layout. | Shared crew start and #849 redesign. |
| MIS-LIFE-01 | Selected owned crew mission Stop/Resume/Archive, correct PIDs and history. | Optional crew changes touch lifecycle paths. |
| MIS-MSG-02 | Existing targeted/broadcast sender/recipient regression with reads and ACKs. | Routing now handles an optional lead. |
| MIS-ASK-01 | Rewritten [regression](regression/missions.md): worker direct-message to lead, refused legacy send, human ask/card/answer and historical ask rendering. | Every crew loses legacy ask sending. |
| MIS-ROSTER-01 | Owned crew edits, remount and frozen original roster/prompt/overrides. | Optional provenance must preserve crew snapshots. |
| MIS-FEED-01 | Feed/native ask and event rendering for the owned mission. | Goal targeting, empty state, historical asks and info surface changed. |

Compare changed surfaces to `target/design-ref/562-{start-from-crew,start-no-crew,crew-picker-open,no-roles-yet,first-role-joined,info-no-crew-row}.png` and retain screenshot paths. Shipped shared controls win over the frame where they differ: the picker, crew picture, seeded role avatars, LEAD badge and runtime/setup rows use the existing components and dimensions.

## Automated checks

Logs are in ignored `target/562-validation/`. Exit codes below are collected from each command, not inferred from log text.

| Command / run | Result | Log |
| --- | --- | --- |
| Migration runner focused tests | 33 passed; exit 0 | `01-migration-runner-retry.log` |
| Migration 0028 / delete focused tests and schema regeneration | 35 passed; exit 0 | `02-migration.log` |
| Workspace nextest after backend and initial dialog | 2,339 passed, 6 skipped; exit 0 | `04-tests.log` |
| Final workspace nextest | 2,343 passed, 6 skipped; exit 0 | `09-tests.log/.exit` |
| Workspace clippy, ci profile, all targets, `-D warnings` | Passed; exit 0 | `09-clippy.log/.exit` |
| Same clippy with `--features updater` | Passed; exit 0 | `09-clippy-updater.log/.exit` |
| `cargo fmt --all --check` | Passed; exit 0 | `09-fmt-check.log/.exit` |
| `git diff --check` | Passed; exit 0 | `09-diff-check.log/.exit` |

The final nextest command is `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`. Clippy uses `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, then the same command with `--features updater` before `--`. Golden updates are intentional: schema 0028, removal of the ask-lead prompt/signal text, and CLI empty-start/help/refusal cases. The CLI harness's update flag is `UPDATE_RUNNER_GOLDENS=1`; daemon goldens use `RUNNER_UPDATE_GOLDEN=1`.

New automated coverage includes the FK-off transaction rollback and restored enforcement; a populated version-27 all-column snapshot with preserved session/mission links; RESTRICT guards and application-owned deletes; empty start/path/envelope/roster/remount; first-turn goal/preamble and absent crew environment/shim; two-worker direct/broadcast routing, human answers and restart notifications; surviving crew deletion; roster load failures without an existing crew; CLI title defaults and send refusal; five-slot picker/roster rendering, keyboard behavior, empty start at minimum window size and disabled→enabled composer input.

Initial failures remain recorded: `01-migration-runner.log` (compile exit 101, statement lifetime fixed); `03-check*.json/.stderr` (type propagation compile failures, exit 101); `03-tests.log` (exit 100, 19 failures from changed contracts/fixtures plus an unchanged OpenCode timeout); `03-focused.log` (compile exit 101, test used a private function); `03-focused2.log` (exit 100, CLI golden update used the daemon flag); `03-cli-golden-update.log` (exit 0 with the CLI flag); `05-tests.log` (compile exit 101, temporary debug-selector lifetime); `06-tests.log` (exit 100, two new UI fixture mistakes and unchanged `model_query_waits_for_settlement_and_reaps_its_private_server` timeout). `07-focused.log` reran all 14 selected cases, including the OpenCode settlement test, successfully (exit 0). `08-clippy.log` then found a needless borrow in a test (exit 101); that borrow is fixed, and both `09` clippy variants pass. The UI fixtures now use flattened `SlotWithRole` serialization and remove only their seeded temporary crews before testing an empty picker. Production OpenCode code and timeout are unchanged; the final full workspace run passed that test and all other tests (exit 0).

## Working-tree review

On 2026-10-15 the reviewer reported **NO REMAINING MUST-FIX ISSUES** for the full working-tree diff at HEAD `dd5fa7b34efaad630cfd08bfd6270cea534f1682`, full diff SHA-256 `98fefab9964c2e72c37c9cc0d99a9f5ee902b8e8f47bccf3403cb76ed5f54289`. It reviewed all six PNGs, the migration and rollback behavior, directory provenance and optional crew readers, no-lead prompts/routing, historical asks, changed tests and all 48 spawn-golden value changes. It ran no builds or tests and changed no files. That candidate was frozen for the first QA run; only this review/status documentation changed after review. Source diff SHA-256 is `d0bb669e55b1e8a5688a58ac134545a3b4870b4a4c7bad10e0b5c4977e8faebb`. No commit, push or PR before QA's verdict.

Non-blocking follow-ups to carry in the PR handoff, unchanged in this candidate:

1. `crates/runner-cli/src/output.rs:446` (callers at 309 and 350) displays a new empty mission as stopped in list/show because it has zero live sessions; the stored mission and start response are running and add-role/lifecycle work. A future rendering case should distinguish an empty roster from a populated paused mission.
2. `crates/runner-cli/tests/golden.rs:674` caps generated help cases at `take(50)`. The three new empty-start cases shift ask/call help out of that list, removing their golden coverage. Make those help cases explicit or remove the positional cap in a follow-up.

## Native QA matrix

### 2026-10-15 preflight and execution plan

QA verified branch `feat/562-start-empty`, HEAD `dd5fa7b34efaad630cfd08bfd6270cea534f1682`, handoff full diff SHA-256 `6454809fce0f99af8a0d0b0592b278cb9af25399c2cc61d18697352234788213`, and source diff SHA-256 `d0bb669e55b1e8a5688a58ac134545a3b4870b4a4c7bad10e0b5c4977e8faebb`. All matched before QA record edits. Environment: macOS 26.6.2 (25G83), arm64; Rust/Cargo 1.97.1; Codex CLI 0.162.1; installed coordination CLI 0.16.1. Candidate app/sidecar are not yet built or launched by QA, so binary identity is pending. No native result is claimed.

The selected plan is macOS with bounded Codex fixtures using an available inexpensive model and explicit low effort, existing sign-in, canonical fresh scratch directories and no global configuration changes beyond the run's approved trust entries. UPG-1 checks preserved row/link counts on read-only copies and saved-work resume; EMPTY-1/2 and DELEG-1 prove native empty/add/delivery/persistence/focus behavior; CREW-1 proves retained mission provenance and operation after deletion; ASK-1 proves legacy refusals, human answer routing and historical rendering. The six selected MIS cases cover redesigned crew form (including five-slot overrides, keyboard/error/640 × 480 states), lifecycle, targeted/broadcast delivery, frozen roster across remount, and feed/ask persistence. All six design PNGs require native screenshot comparisons. Delivery requires event IDs, covering reads and recipient ACKs; goal/task receipt uses saved runtime transcripts. Windows requires Jason's native follow-up and stays unverified. No unrelated regression suite is selected.

**Setup Blocked:** before any build, launch, DB copy or native interaction, `ps` showed a pre-existing development daemon PID `8829`, started `2026-10-15 11:29:44` local, executable under `com.wycstudios.runner-dev/bin/runnerd`; no development app process was visible. The feature plan explicitly requires blocking when another person's development app/daemon is running. QA asked Jason through Runner for clearance to stop this daemon/use the namespace and notified coder; no process/session/data was touched. This prerequisite initially blocked all selected native checks. This is an environment blocker, not an observed product failure. Source remains frozen. The initial blocker will be retained if a later cleared run passes.

Evidence root: `/private/tmp/runner-562-qa-1sitq8cf`. Preflight identity: `evidence/candidate-preflight.json`; process inventory: `evidence/preflight-processes.txt`; fixture/process ledger: `ledger.json` (all lists empty). No screenshots exist yet. No QA-owned app, daemon, agent, watcher or helper was started; there is nothing to stop/archive at this checkpoint.

The initial preflight blocked every selected native row. Clearance allowed the run below; this final matrix supersedes those initial statuses while preserving the blocker history above. **Overall: Failed, with a remaining Blocked upgrade subcheck.** This is selected macOS feature QA, not a full regression pass.

| Check | Verdict | Result and primary evidence under the evidence root |
| --- | --- | --- |
| UPG-1 | Blocked | Counts and links preserved, migration 27→28 and paused route restored. Pre-existing conversation resume requires its owner; see U1. `db-before/counts.json`, `db-after/counts.json`. |
| EMPTY-1 | Failed | Empty start, goal/task receipt and directed/broadcast delivery work. Empty rail has wrong copy/no Add role, first worker is labeled LEAD, and new icon is blank; F1–F3. `empty-workspace-settled.png`, `first-role-joined.png`, `runtime-prompt-proof.json`, `empty-delivery-proof.json`. |
| EMPTY-2 | Passed | Same empty mission reopened without a paused overlay; one-member mission resumed its original session/key and accepted a human answer. `empty-persist-after-restart.png`, `single-role-after-restart.png`, `single-resumed-terminal.png`, `post-restart-01M4YV2AF9WB7YMSN52QZRRN7T.json`. |
| DELEG-1 | Passed | Owned direct chat executed empty start and stdin add-role; Recents updated while that chat retained focus. `delegation-focus-before.png`, `delegation-complete-no-focus-steal.png`, direct-chat saved transcript. Initial unsupported flag/sandbox retry preserved below. |
| CREW-1 | Passed | Deleted owned running crew; original mission retained origin directory, accepted posting/add-role, survived restart and had no Crew info row. `delete-before-show.json`, `delete-after-show.json`, `delivery-final-deleted.json`, `deleted-crew-retained-info.png`. |
| ASK-1 | Failed | Both legacy refusals, exact `ask --human`, native answer routing and historical ask rendering pass. Answered crew ask reopens on remount; F5. `legacy-refusal-log-proof.json`, `exact-alias-native-answered.png`, `historical-ask-native-top.png`, `crew-asks-replay-complete.json`. |
| MIS-CREW-01 | Failed | Crew preselection/switch, five override rows, empty-crew refusal, real failed-start error, keyboard start and 640 × 480 work. Crew goal recipient label is missing; F4. `crew-preselected-three.png`, `five-slot-overrides.png`, `minimum-five-slot-scroll.png`, `failed-start-deleted-crew.png`, `crew-valid-native-start.png`. |
| MIS-LIFE-01 | Passed | Native slot stop/resume, blind recall, all-stop/resume with original keys/new PIDs and final native Archive; timestamp/zero live and absent Recents verified. `worker-blind-recall-result.png`, `lifecycle-all-resumed.json`, `mission-native-archive-result.png`, `recents-after-archive.png`, `cleanup-final-01M4YV82RXZ43N8S4J3J680WE4.json`. |
| MIS-MSG-02 | Passed | Native no-recipient broadcast reached all three; @worker chip reached worker and attributed human correctly. Phase 3 decision 8 explicitly preserves composer broadcast, overriding the suite's stale default-lead wording. `crew-broadcast-submitted.png`, `explicit-target-submitted.png`, `delivery-final-crew.json`. |
| MIS-ASK-01 | Failed | Worker→lead message/read/ACK, legacy refusals, initial human card/answer and historical rendering pass; remount resurrects the answered question, F5. Crew lead used `signal ask_human`; exact alias was separately exercised with supported external `--mission/--as` attribution. |
| MIS-ROSTER-01 | Passed | Original handles/lead/overrides/addendum/keys survived crew edits and remount; new names refused; original directed/broadcast recipients read and ACKed. Current crew page reflects edits. `crew-roster-before.json`, `crew-roster-after.json`, `after-session-*.json`, `remount-directed-proof.json`, `edited-crew-current-roster.png`. |
| MIS-FEED-01 | Failed | Bounded content/authors and historical event render, default feed hides routine events available in CLI `--all`. Answered/pending state does not persist across remount, F5; crew goal label also missing, F4. |
| Native Windows | Blocked | No Windows host in this run; native behavior remains unverified. CI is a separate gate. |

## Regression cases for the release sweep

- **Empty start and add (EMPTY-1):** In a fresh owned folder start No crew from the dialog. Assert no sessions, correct no-role feed/rail, disabled composer and no paused overlay. Add two bounded roles with distinct tasks; each reads the shared goal and its own task, posts an ACK, exchanges directed messages and receives an untargeted composer broadcast. Record native screenshots, events, covering reads and ACKs.
- **Empty persistence and delegation (EMPTY-2 / DELEG-1):** Start an empty mission from an owned direct chat; confirm sidebar arrival without focus change, then add one role by stdin task file. With this mission and a second empty mission saved, restart only the owned development app/daemon. Both reopen, the first member resumes, and paths stay fixed. Record mission/session IDs, keys, files and native evidence.
- **Crew provenance after deletion (CREW-1):** Start from an owned crew, then delete that crew. The running mission retains its stored origin ID, directory, roster and prompts, accepts messages and a new role, and shows status without a crew. Every mission's info panel lacks the Crew row. Archive the owned mission and stop its processes.
- **Legacy asks (ASK-1 / MIS-ASK-01):** Refuse both former ask-lead send forms without appending an event; the hint directs the agent to its lead by message or to the person with `runner ask --human`. Resolve one human card to its actual asker. An owned historical `ask_lead` event remains readable/renderable and does not inject a legacy escalation on replay.

## PR and CI

The second working-tree review is clean and QA has released the source with **Incomplete: 11 Passed, 0 Failed, 1 Blocked**. [PR #906](https://github.com/yicheng47/runner/pull/906) has five focused commits against current `origin/main`, references `Part of #562` and `Closes #849`, and reports the QA verdict without upgrading failed or blocked checks. macOS and Windows CI are tracked in the [PR checks](https://github.com/yicheng47/runner/pull/906/checks) and must be green; no merge is authorized.

Coder process cleanup before review: final gates finished, and a `ps` check found no remaining worktree build/test/fixture process. The coder did not launch the development app.

### Preflight clearance

Claude Code, acting for Jason under AGENTS.md’s standing development-app authorization (clarified at 03:45 UTC), answered “QA may stop dev daemon and proceed” through Runner at 03:42 UTC and confirmed PID 8829 had no child processes and no development app was running. QA stopped that daemon through the absolute development CLI (`{"stopped":true}`), copied the development database before candidate launch, and began sanitized `make run`. Read-only copy `_migrations` ends at 27 (`PRAGMA user_version` is 0 and is not Runner's schema version). Before counts: crews 27, slots 22, roles 43, missions 28, sessions 168, mission-linked sessions 63, attention 118. Evidence stays under the preflight root. No pre-existing session has been operated.

## Frozen-candidate native run — 2026-10-15

### Identity, authorization and method

The preflight identity above remained fixed throughout this run. `evidence/candidate-final.json` confirms the same HEAD and source diff hash at cleanup. Only this QA record changed during execution. Initial full diff hash includes the handed-off documentation; later full hashes necessarily differ as this record is filled in. No product source/test/expectation, commit, push or PR was made by QA.

QA built with sanitized `make run` (all four coordinating `RUNNER_*` variables and `NO_COLOR` removed), then used a temporary native wrapper `Runner Smoke Dev.app`, bundle ID `com.wycstudios.runner.smoke-dev`, under the canonical scratch root. The wrapper executable and sidecar were byte-identical to this worktree's development build. No Cargo ran concurrently with another slot. `make-run.log` records the successful build; the coder's completed automated gates above were not rerun.

| Binary identity | SHA-256 |
| --- | --- |
| `target/debug/Runner` and scratch wrapper `Contents/MacOS/Runner` | `fde226425bd1d4c3bec684857b8bee072dcf7e61700aa65524475d737d3fa54b` |
| `target/debug/runner-agent-cli`, wrapper sidecar, development `bin/runner` and `bin/runnerd` | `4e054a1870692a87d92afb7c684bd9308e5b3e74ac4acb0fa6cda47f73ec9764` |

App/CLI version was `0.16.1 (dev)` / `0.16.1`; endpoint was `/Users/jason/Library/Application Support/com.wycstudios.runner-dev/runnerd.sock`. OS/tools are the preflight versions above. Real fixtures used Codex `gpt-6-luna`, low effort, existing sign-in and fresh scratch folders. Native computer use was verified by harmlessly opening/dismissing New chat before relying on input. UI actions, composer sends and human answers used the host's native computer-use API; CLI JSON, owned event logs and saved runtime conversations supplied structured evidence. No private PTY input or database mutation was used. Only UPG-1's expressly required read-only copies of the database were queried; no other SQLite access occurred. Production app/daemon and pre-existing sessions were not operated.

All six design references were visually compared. Shipped shared picker size, crew picture, seeded avatars, lead badge styling and runtime/setup rows were accepted as instructed; F1–F4 concern required content/design behavior, not those accepted shared-control differences. Screenshots are full native captures, including 1280 × 960 pixels at the 640 × 480 logical minimum and the larger normal window.

### Findings and blocked work

**F1 — Empty Sessions rail omits Add role and uses the wrong empty copy (EMPTY-1).** Start a No crew mission and open Sessions with zero members. Expected “No roles yet.” followed by **+ Add role**; actual “No sessions yet.” with no rail action. The feed's Add role does work, the composer is disabled, goal has no recipient, and no Mission paused overlay appears. Evidence: `empty-workspace-settled.png`, `empty-initial-show.json`, `empty-roster.json`. Recheck both empty feed and rail after a fix.

**F2 — First member is visually labeled LEAD in a lead-less mission (EMPTY-1).** Add alpha to the empty mission. Sessions displays LEAD although CLI/session/roster and the role_joined event all have `lead:false`; the first-turn prompt explicitly says the mission has no lead. Reproduced with the later one-member exact-alias fixture as well. Evidence: `first-role-joined.png`, `empty-after-first-add.json`, `runtime-prompt-proof.json`, `exact-alias-native-pending.png`. This is a display failure; no evidence established that routing elected a lead.

**F3 — New user-plus icon is blank (EMPTY-1 / dialog visual comparison).** The No crew card/menu and new empty-feed icon area leave the new glyph blank. Expected the specified user-plus tile. Evidence: `start-no-crew.png`, `crew-picker-open.png`, `empty-workspace-settled.png` against the corresponding design PNGs. Existing shipped avatars/icons otherwise render.

**F4 — Existing crew goal loses its recipient label (MIS-CREW-01 / MIS-FEED-01).** Start the valid three-member crew from the native form. Expected its goal row to retain `→ @lead`; actual goal has no recipient label, just as the intentionally untargeted empty goal does. The crew has a real lead and correct original roster; its mission_goal event has `to:null` and only `payload.text`, without a target. Evidence: `crew-valid-native-start.png`, `valid-crew-show.json`, `crew-goal-envelope.json`. This is an unmet selected assertion; regression attribution is not established. Coder acknowledged triage while source remained frozen.

**F5 — Answered human ask reopens after daemon remount (ASK-1 / MIS-ASK-01 / MIS-FEED-01).** In the owned three-member mission, worker messages lead; lead asks the person on worker's behalf. Answer YES natively: the first card resolves, pending clears and lead's actual terminal receives `[human_response] YES`. Stop the owned app/daemon and reopen the identical candidate. Expected the same question to stay answered and pending to remain zero. Actual router replays the same ask into a new question, pending becomes one, and a warning says the original response references an unknown question. The reopened card also changes asker from lead to worker. An owned historical ask_lead fixture was present in this log; this run did not isolate its causal role. Answering it again clears the test state but does not erase this first failure. `crew-pending-human-ask.json`, `crew-after-restart-pending.json`, `crew-asks-replay-complete.json`, `crew-after-reanswer.json`, `crew-human-ask-answered.png`, `historical-ask-rendered.png` and `legacy-ask-native-feed.png` retain the sequence. Coder acknowledged the separate failure; no comparison established when it was introduced.

| F5 event | ID |
| --- | --- |
| Lead ask_human (`on_behalf_of:worker`) | `01M4YV9WVWHMCFJ7K2AS0F4CH6` |
| Original question, asker lead | `01M4YV9WVXFM2Z3ZXYHDBNBS5J` |
| First native YES response | `01M4YVB4JR6P8E1BQ03HTZH32F` |
| Replay-created question, same triggering ask | `01M4YVDVAWWJG8Q6PM7Y24J5QQ` |
| Original-answer unknown-question warning | `01M4YVDVAWWJG8Q6PM7Y24J5QR` |
| Second native YES response | `01M4YVJJ1BX6SCSTSQ3FNA22Q2` |

**U1 — Pre-existing saved conversation resume remains Blocked (UPG-1).** Fresh before/after copies retain crews 27, slots 22, roles 43, missions 28, sessions 168, mission-linked sessions 63 and attention 118. `_migrations` advances 27→28 and `foreign_key_check` returns no rows. The saved paused route restores, but no pre-existing session was live before launch; QA cannot operate sessions it did not create to prove resumption. Claude Code acting for Jason explicitly answered through Runner at 04:05 UTC to keep this subcheck Blocked and put it in the PR's **waits for Jason** list. Jason can resume one of his own development chats. Counts/route evidence is not an upgrade-resume pass; resuming QA's newly created fixtures below is a separate persistence check.

### Passing behavior and delivery evidence

Empty start `01M4YTTE9559QXYTXYHT3HD1B9` created no sessions, a version-2 empty roster, `crew_id:null`, and the fixed `runner-dev/missions/<id>/` directory. Its events omit crew_id; the generated shim explicitly unsets RUNNER_CREW_ID. Saved alpha/beta first-turn conversations contain the lead-less preamble and shared GOAL-562-EMPTY; their addressed task events are covered by inbox reads. The first role's initial test instructions were faulty (see retries below), so only observed prompt/task receipt and subsequent posting/delivery are claimed, not an untouched automatic task ACK. CLI's zero-live display says stopped, as the review follow-up already records; stored mission/start and native no-paused state remain running.

| Delivery | Probe → covering read → ACK |
| --- | --- |
| Empty native composer broadcast to alpha | `01M4YTZW5S2SA6BX2MHVTNKR18` → `01M4YTZWC1AWE0Y5WVPA73J97T` → `01M4YV00XGBZG9W77N7MTYA8SR` |
| Same broadcast to beta | same probe → `01M4YTZZG6819BTSX07X6C2TNR` → `01M4YV05KJQNYVA0EVKD9CHFHX` |
| Alpha→beta directed probe | `01M4YV02Y6QFMEHVJZE668TXMQ` → `01M4YV088DBHAN8M0CCBRGDCFQ` → beta→alpha `01M4YV0AHA8B5WTF03SFMHQ0XV` |
| Reverse directed ACK read by alpha | `01M4YV0AHA8B5WTF03SFMHQ0XV` → `01M4YV0CZFGAAK8WT0NAYXES4A` |
| Crew native explicit @worker | `01M4YVBEJ2PWTYC5Z005DRHEJD` → `01M4YVBHH341KYWEERVXBTV8H4` → `01M4YVBM606VP8AMP50ZK04VK8` |
| Frozen original peer→worker after remount/resume | `01M4YW3KCRHANK3VXMJ3G41JBW` → `01M4YW3W10TPYJHV3E3TNTZPT2` → `01M4YW3Z85H11J43GPCQY6CD7T`; peer read `01M4YW429Z5E8FSTFHB2XG87NE` |

Crew broadcast probe `01M4YV9M79T7WXG6XRMAE7H299` was covered by each original recipient and produced lead/worker/peer ACKs in `delivery-final-crew.json`. Post-remount broadcast `01M4YVKPAVY6ZKV7A3SDGH31GP` likewise reached all original handles; renamed/replacement targets refused with the original team listed. Some cheap-model ACK bodies misnamed their own handle; authenticated `event.from`, recipient reads and saved terminal evidence identify actual recipients. They are not inferred from model prose.

The three-member roster was byte-identical before crew edits, after edits and after remount: original lead/worker/peer, lead=lead, all model gpt-6-luna/effort low, original crew name and null addendum. Session JSON and native footer preserve effective model/effort. Saved lead prompt names original crew and peers; no edited addendum appears. Current crew page correctly shows QA562 Three Edited, lead @renamed, replacement and medium overrides. All original keys survive the restart and slot/all resumes. Worker PID changed from 43303 initially to 94177 after its native slot resume; final all-resume PIDs were lead 33164, worker 33165, peer 33166 with original IDs/keys. Blind native worker recall returned exactly MEMORY-562-ORIGINAL without tools. Final native Archive left zero live sessions and a non-empty archive timestamp, removed the row from Recents and preserved conversations.

Both removed ask forms returned exit 2 with the specified direct-message/person hint on empty and crew missions. The zero-member fixture's log stayed exactly 503 bytes through both attempts (`legacy-refusal-log-proof.json`). An explicitly authorized owned historical ask_lead event `01M4YV00000000000000000ABC` was appended only while the daemon was stopped; it renders as HISTORICAL-ASK-562 at its timestamp-sorted position, and no legacy inline question was injected on replay. This offline fixture is evidence of historical rendering, not evidence of live legacy send routing.

The original real-agent human asks used the supported `signal ask_human` verb even when prompted to use the alias. To test the exact alias without claiming that different spelling was executed, QA created one final one-member fixture and invoked the documented external CLI `ask --human "QA562 exact ask alias" --choices A,B --mission <id> --as solo`. The resulting signal/card was answered A through native UI; the solo terminal visibly received `[human_response] A`, and pending count cleared. Exact argv/exit/IDs and native screenshots are `alias-send-exact.json`, `alias-pending.json`, `alias-after-answer.json`, `exact-alias-native-pending.png`, `exact-alias-native-answered.png` and `exact-alias-recipient-terminal.png`. This does not convert F5 into a pass.

### Preserved retries and limitations

The first alpha fixture used an invalid `--to human` destination and a bare runner invocation in a Codex login shell resolved the older installed CLI, which refused the no-crew envelope. Alpha briefly tried an invented crew environment value; no persistent agent setting was changed. QA corrected only its own fixture instructions to the absolute development CLI and broadcast syntax, then sent a native bounded correction. Those first failed attempts remain in the owned saved conversation and are not candidate-path successes. Reported delivery passes use the verified absolute development CLI. Beta read its supplied task but followed its generic BOOT instruction rather than emitting the exact requested task ACK; saved prompt/task/read evidence is retained rather than treating model recall as proof.

The first direct-chat start used unsupported `--name` (exit 2, no session created); the corrected command succeeded. Its agent's first delegation attempt hit its normal sandbox boundary (exit 5), then its normal approved retry succeeded without changing global settings. The automatic-watch capability was unavailable in this fixture host; no follower was started and no watcher-delivery result is claimed. The agent's printed foreground feed command was one-shot; a real continuous foreground observer would add `--follow`.

An attempted native failed-start check used a nonexistent cwd, which unexpectedly launched three bounded sessions with a shell fallback outside that requested folder. QA promptly stopped/archived that mission and did not count it as a valid-cwd start or an error-state pass. No repository work was requested. A later real backend failure used an owned crew deleted while the Start form was open; “crew not found” preserved the dialog/input (`failed-start-deleted-crew.png`). The cwd observation is retained without attributing a new regression. Five-slot/override and minimum-size screenshots were captured without starting the five-slot crew.

The first restart attempt's native quit chord left the old wrapper alive; stopping its daemon let it reconnect before the new launch, briefly producing two owned wrappers. QA terminated both owned wrappers (69280 and 34358), stopped the development daemon, verified no dev process remained and relaunched one identical wrapper (42944, daemon 43385). These failed quit/restart attempts are preserved in the logs. The clean restart provided EMPTY-2/roster persistence evidence; F5's first replay was already observed during remount and remains recorded. A directed probe was interrupted by the intentional all-stop; it was resubmitted through the owned peer's real native terminal after resume, and the later probe/read/ACK chain is listed above.

### Fixture ledger and cleanup

Scratch root `/private/tmp/runner-562-qa-1sitq8cf` is canonical and outside the repository. `ledger.json` records all created roles, crews, missions, sessions, folders and processes; no pre-existing similarly named QA objects were used. Roles qa562-alpha/beta/gamma are retained. Crews QA562 Three Edited, QA562 Five and QA562 Empty are retained; QA562 Delete alone was deleted during the explicitly required CREW-1 check. Nothing else was deleted. Scratch evidence, wrapper, database copies and conversation history are retained locally; private database/config/transcript material must not be committed.

| Owned fixture | Mission/session ID | Final state |
| --- | --- | --- |
| Empty UI + alpha/beta | `01M4YTTE9559QXYTXYHT3HD1B9` | Archived, zero live |
| Empty Persist | `01M4YV0BYPXV449252YDT673RB` | Archived, zero live |
| Delegated + solo | `01M4YV2AF9WB7YMSN52QZRRN7T` | Archived, zero live |
| Invalid-cwd start attempt | `01M4YV57N9ENH7ZXJ9D3KB1KEV` | Archived promptly, zero live |
| Delete Running Crew + lead/after | `01M4YV6RP61682P6MX9M4JE6YF` | Archived, zero live; original directory retained |
| Crew Valid + lead/worker/peer | `01M4YV82RXZ43N8S4J3J680WE4` | Native Archive, zero live, absent Recents |
| Exact ask alias + solo | `01M4YWKASHF5Z7Z9X0PGTDHZQ9` | Archived, zero live; alias-cleanup files |
| Delegating direct chat | `01M4YV0PM407ZYA4711427KJJB` | Stopped/archived |

The original six missions and direct chat were verified archived through CLI JSON; all corresponding agent/MCP child processes stopped. The final alias fixture, its app/daemon and its one approved trust entry received the same cleanup after the exact-alias check. Final process/config evidence below is authoritative for the end of the whole run. No watcher/load/helper survives. Production Runner remained running and untouched. Only owned trust entries were removed; final Codex config matches its captured pre-run bytes and semantics. Temporary native wrappers were unregistered from LaunchServices without deleting files.

Evidence index: `candidate-preflight.json`, `candidate-final.json`, `binary-hashes.json`, `db-{before,after}/counts.json`, native PNGs named in the matrix/findings, `final-events-<mission>.ndjson`, minimized `final-transcript-<key>.jsonl`, `runtime-prompt-proof.json`, `remount-directed-proof.json`, `crew-asks-replay-complete.json`, `cleanup-fixtures.json`, `config-cleanup.json` `cleanup-final-verification.json`, `alias-config-cleanup.diff` and `final-config-baseline.diff`. Full paths are `<scratch>/evidence/<name>`. All source changes remain frozen and uncommitted; coder must route fixes through working-tree review and hand back a newly identified candidate for affected checks. U1 and native Windows remain explicit human/platform gaps, and every Failed/Blocked row must be carried into the PR as recorded.

Final cleanup at 2026-10-15T04:23:55.462394+00:00: all seven owned missions archived, one direct chat archived, 13 owned sessions stopped, no owned/dev-path process or recorded live PID survives. `cleanup-final-verification.json` and `cleanup-processes-final.txt` record the final ps checks; `final-config-baseline.diff` is empty. Exact-alias question `01M4YWKP96R9ZF493SRBQ48CW0` received native response `01M4YWMX8Y0VMV93H44E0DTY1B`; requester ACK `01M4YWN343BY3CMN22GPRWEM1T` confirms A.

## Coder follow-up after the first QA verdict

QA released the candidate after its 2026-10-15 run with 6 Passed, 5 Failed and 1 Blocked selected macOS checks. The original matrix, evidence and cleanup above remain unchanged. UPG-1's pre-existing-session resume remains Blocked at Jason's direction; native Windows remains unverified. The UI fixes below require another working-tree review and affected live QA before any commit or PR.

F1 removes the empty-session early return from the Sessions rail, so the empty roster shows “No roles yet.” and its Add role button. F2 renders the LEAD badge only from the session's recorded lead flag. F3 registers the missing `user-plus.svg` asset used by the No crew tile/menu and empty feed. F4 derives a goal row's recipient from the mission roster's actual lead, leaving a lead-less goal without a recipient. The missing crew goal label existed at the base revision too; this UI correction aligns the changed surface with the approved design without changing the goal envelope or routing.

F5 is attributable to the offline legacy-log fixture. Its appended ID `01M4YV00000000000000000ABC` precedes the existing ask, question and answer IDs despite appearing after them in the log. `EventLog::append` guarantees strictly increasing ULIDs in append order, and the unchanged `Router::reconstruct_from_log` uses the final ID as its replay watermark. The older manually appended tail therefore made the answered ask look new during replay: it created a new card, then warned when the historical answer referenced the original resolved card. `target/562-validation/10-ask-replay-attribution.json` records the ID comparison and confirms that reconstruction is byte-for-byte unchanged from HEAD. No production replay change is made for this invalid fixture ordering.

The new reconstruction test answers a human ask, appends a legacy ask through the ordered event log, reopens the router and replays history for both crew-lead and lead-less rosters. It requires zero pending asks, no new log rows and no stdin injection. The GPUI regression checks the empty rail's copy/action, an actual worker's absent LEAD badge, a real lead's badge, and goal recipient presence only with a lead; an asset test covers registration. All three focused tests passed, exit 0 (`10-focused.log/.exit`). QA must recheck F1–F4 visually and F5 with a monotonically increasing offline legacy event ID, preserving the first failed attempt. Earlier passing behavioral checks remain attached to their original frozen source; the new verdict belongs to QA.

The first post-QA full suite (`11-tests.log/.exit`) returned exit 100: 2,344 passed, one unchanged OpenCode settlement test timed out at its existing three-second deadline, and six were skipped. Its isolated retry passed with exit 0 (`12-opencode-retry.log/.exit`). No OpenCode implementation or timeout was changed. Both post-QA workspace clippy variants passed with exit 0 (`12-clippy.log/.exit`, `12-clippy-updater.log/.exit`); the final full-suite rerun passed 2,345 tests with six skipped, exit 0 (`12-tests.log/.exit`). Formatting and diff checks passed, exit 0 (`12-fmt-check.log/.exit`, `12-diff-check.log/.exit`).

## Second review and affected QA handoff

On 2026-10-15 the reviewer reported **NO REMAINING MUST-FIX ISSUES** for the fixes at HEAD `dd5fa7b34efaad630cfd08bfd6270cea534f1682`, full diff SHA-256 `59aa14489e9556a1708a85c08cc8820cd97ff20a032e68b029a0d39477bc4787`, source diff SHA-256 `a40b79f9c5b456561004c396eb1acb295a89d106ea925f28cce3978ce76c5364`. It checked the five changed source/test files, their callers and validation evidence, without editing, building or testing. Source is now frozen for the affected QA rerun; only this review/status documentation changed after review.

The two earlier non-blocking CLI follow-ups remain. The reviewer added a non-blocking restart-confirmation copy issue at `mission_workspace/mod.rs:198` / `actions.rs:140–154`: a lead-less worker's confirmation says “and @ gets a note” because the absent lead becomes an empty string. Execution already notifies every other member correctly. Carry this copy fix as a follow-up under the brief's severity gate.

The affected rerun covers EMPTY-1 (empty rail/action, absent worker LEAD badge, icon in dialog/picker/feed and no goal recipient), MIS-CREW-01 (real crew LEAD badge and goal recipient), ASK-1 and MIS-ASK-01 (human ask/card/answer remains resolved through remount alongside a correctly ordered historical ask), and MIS-FEED-01 (goal labels and historical/answered ask rendering). Retain each original failed attempt and let QA issue the new verdict. Earlier passing cases remain valid for their unchanged production behavior. UPG-1's pre-existing-session resume remains Blocked at Jason's direction; do not operate another owner's sessions. Native Windows is still unverified. No source edit or coder build while QA holds this candidate.

## Second native run — affected checks

Preflight at 2026-10-15T04:40:10.005404+00:00 verified the handed-off branch, HEAD, full diff `f9a285a3d1d4fe75a330bde5d96a2666f1354a7260e3fddb40f4dc3a359c3d99` and source diff `a40b79f9c5b456561004c396eb1acb295a89d106ea925f28cce3978ce76c5364` before this record edit. No development app/daemon is running. New canonical evidence root: `/private/tmp/runner-562-qa2-kid_9hnh`. Same macOS/Codex development setup and authorization apply; exact tool readbacks are in `evidence/preflight.json`.

Plan: rebuild/identify this frozen candidate, then recheck EMPTY-1's No crew icon/menu, empty feed/rail and rail Add role, disabled composer/no overlay/untargeted goal and first worker's absent LEAD badge. Start a bounded owned crew natively to check its recorded lead badge and actual goal recipient. Exercise worker→lead message, legacy refusals and human ask/native answer, append a strictly increasing owned offline historical ask with the daemon stopped, restart the identical app/daemon and require the answer to remain resolved with no new question/warning/injection. Capture native and structured evidence for ASK-1, MIS-ASK-01 and MIS-FEED-01. Carry forward unchanged passing rows explicitly; UPG-1 owner-resume stays Blocked and native Windows unverified. Stop/archive all new fixtures and owned processes and remove only this run's trust entries. The first run and its invalid legacy fixture remain preserved.

### Candidate, execution and evidence

QA rebuilt with sanitized `make run`; the native build completed successfully in 6.09 seconds. The executable copied to `/private/tmp/runner-562-qa2-kid_9hnh/Runner Smoke Dev.app/Contents/MacOS/Runner` is byte-identical to this worktree's `target/debug/Runner`, SHA-256 `f8dbfa4b819fbc32f1ff48457dd5e13360bac32056efcaa02de41393919d5e78`. The CLI sidecar, wrapper sidecar and development `bin/runner` and `bin/runnerd` all retain SHA-256 `4e054a1870692a87d92afb7c684bd9308e5b3e74ac4acb0fa6cda47f73ec9764`. Bundle ID is `com.wycstudios.runner.smoke-dev-qa2`; CLI status reports `0.16.1 (dev)` and the development socket. Exact paths, hashes, process identity and tool versions are in `evidence/{preflight,build-hashes,candidate-status,candidate-final}.json` and `evidence/{running-identity,remount-process-identity}.txt`. Final source hash remains `a40b79f9c5b456561004c396eb1acb295a89d106ea925f28cce3978ce76c5364`; QA changed only this record.

The same macOS 26.6.2 arm64, Rust/Cargo 1.97.1 and Codex CLI 0.162.1 environment applies. Fixtures used Codex `gpt-6-luna`, low effort, existing sign-in and the new scratch root's `empty-ui` and `crew` folders. Native control was verified by opening/dismissing New chat. All UI starts, Add role, terminal instructions and answers used native computer use; absolute development CLI JSON, owned append-only logs and saved runtime conversations provide structured evidence. The four coordinating `RUNNER_*` variables and `NO_COLOR` were removed from the app and external test CLI environment. No SQL, private PTY input, authentication changes or pre-existing-session operation occurred. No new database inspection was necessary: UPG-1 evidence is carried from the first run. Automatic watching is unavailable in this host; no follower was started and no active watch is claimed. QA used the runner and runner-dev skills; neither was used to operate production product data.

F1–F4 now pass visually. The No crew menu/card and empty feed show the user-plus icon. With zero members, Sessions shows “No roles yet.” and a working Add role button; the composer rejects input and no paused overlay appears. Adding the first worker through that rail produces `lead:false` with no native LEAD badge. Its goal stays untargeted. Starting the owned two-slot crew natively displays LEAD only on `@captain` and the goal recipient `→ @captain`. Evidence: `no-crew-{menu,card}-icon.png`, `empty-start-fixed-rail.png`, `empty-composer-rejects-input.png`, `rail-add-role-open.png`, `first-worker-task-ready.png`, `first-worker-no-lead.png`, `owned-native-start-verified.png` and `crew-goal-captain-and-real-lead.png`, all under the second root's `evidence/`. These rechecks use the same six design references and accepted shared-control exceptions as the first run.

### Ordered historical-event and human-answer recheck

The solo and captain each executed the exact `ask --human` alias through their real native terminal. Solo offered A/B; QA answered A. Captain offered the actual single opaque label YESNO; QA answered YESNO. Each native card resolved, pending became zero, the actual terminal received its `[human_response]`, and its asker broadcast the corresponding ACK. Saved exact tool calls and input messages are in `final-runtime-proof.json`; native pending/answered cards and captain terminal are captured in `solo-human-card-{pending,answered}.png`, `captain-human-card-{pending,answered}.png` and `captain-human-answer-terminal.png`.

| Fixture | Ask → question → native response → asker ACK |
| --- | --- |
| Lead-less solo | `01M4YYCZ4P2P58QSFKPA1RDNMJ` → `01M4YYCZ52TVES8GDENG9NXXS4` → `01M4YYF4B818C4PHFJ2GRB8QJ9` → `01M4YYF7D77DR9BFFDNY5BD1JP` |
| Crew captain | `01M4YYHP3R3A0RDGG1DR2KYSZ9` → `01M4YYHP4335TFDHSSDDJRFJCJ` → `01M4YYMTZX8K1QR3Q98444X16Q` → `01M4YYMYD82X7XSED5F7Z11P54` |

The worker's directed request `01M4YYANW2WNEXN943Y07EZGNA` is covered by captain read `01M4YYAS2MV3R0RNJSVAVYYAX0`. Captain's directed ACK `01M4YYHKWX1FEG8QSB4Q6TY0F3` is covered by worker read `01M4YYHPQ20HQEY83ZEB3T454F`. This confirms the message path and covering reads. The captain's initial cheap-model turn read the request but stayed Working; QA interrupted only that owned turn natively and supplied a bounded two-command instruction before the observed ACK/ask. This is not claimed as an untouched automatic response. The first run's successful `on_behalf_of:worker` question/answer substep remains carried forward; this rerun's captain question merely names worker and does not assert that payload field.

After both answers, QA stopped the owned wrapper and development daemon and verified no development process remained. QA then appended one explicitly authorized historical `ask_lead` rendering fixture to each owned log. Each new ID is exactly one greater than that log's prior maximum/tail: solo `01M4YYF8VDRV39V924DJE5DJWD` → `01M4YYF8VDRV39V924DJE5DJWE`; crew `01M4YYN3K3K5KGVCD2XEHZ3SVR` → `01M4YYN3K3K5KGVCD2XEHZ3SVS`. `ordered-legacy-fixtures.json` records the before/after order assertions, original question/answer IDs, log hashes and envelopes. The lead-less event omits crew_id and recipient; the crew event retains its actual crew and captain recipient. Original first-run evidence and its invalid event were not modified.

The identical wrapper/sidecar restarted with the same three session IDs and runtime keys. Both missions retained exactly their one original question and one original answer, zero pending asks, one original ask_human, one historical ask_lead and zero warnings. No question/answer row was added by replay, and saved user messages contain no historical inline escalation. Native `empty-post-remount-answered-and-legacy.png` and `crew-post-remount-answered-and-legacy.png` show the resolved cards beside the historical events, with correct lead badges. Structured proof: `pre-remount-proof.json`, `remount-result.json`, `{empty,crew}-{pre,post}-remount.json`, ordered event snapshots and `final-runtime-proof.json`. The invalid first-run F5 attempt remains a failed fixture execution; it is not evidence of a production replay regression. Correctly ordered lead and lead-less rechecks pass.

### Updated selected matrix and verdict

**Overall: Incomplete — 11 Passed, 0 Failed and 1 Blocked selected macOS checks; native Windows remains Blocked/unverified separately.** Five affected rows were rerun. Six unchanged passing rows and UPG-1's existing partial evidence are explicitly carried forward from the first frozen candidate because the intervening production changes only affect the rail, badge, icon and goal rendering; production daemon/routing behavior is unchanged. This is selected feature QA, not a full regression pass. Earlier failures, the setup blocker, retries and their evidence remain above.

| Check | Verdict | Current basis |
| --- | --- | --- |
| UPG-1 | Blocked | Carried forward: migration 27→28, counts/links and paused route pass. Pre-existing owner conversation resume remains U1, explicitly waits for Jason. |
| EMPTY-1 | Passed | Rerun: icons, zero-role feed/rail, rail Add role, disabled composer/no overlay, untargeted goal, and first worker without LEAD. Saved solo conversation contains goal/task and its task ACK. First run's unchanged two-member directed/broadcast paths remain carried forward. |
| EMPTY-2 | Passed | Carried forward: zero-member and one-member persistence. Rerun additionally preserves solo's original session/key and resolved answer. |
| DELEG-1 | Passed | Carried forward: direct-chat start/add, Recents arrival and retained focus. Original unsupported-flag/sandbox retries remain recorded. |
| CREW-1 | Passed | Carried forward: crew deletion retains origin/directory/history, post/add operation and no Crew info row across restart. |
| ASK-1 | Passed | Rerun: exact alias, native answers, resolved cards and correctly ordered historical rendering survive remount for lead and lead-less fixtures. Unchanged legacy-send refusal/no-append evidence is carried forward. |
| MIS-CREW-01 | Passed | Rerun: actual captain LEAD badge/goal recipient through valid native start. First run's unchanged preselection, five overrides, empty refusal, error retention, keyboard and minimum-size form checks are carried forward. |
| MIS-LIFE-01 | Passed | Carried forward: native stop/resume, blind recall, all-stop/resume with original keys, Archive and zero live. |
| MIS-MSG-02 | Passed | Carried forward: native targeted/broadcast delivery/read/ACK. Phase 3 decision 8 wins over stale suite default-lead wording. |
| MIS-ASK-01 | Passed | Rerun: worker→captain request/read, directed ACK/read, exact human alias/native answer and resolved state after remount. Initial `on_behalf_of` success and legacy refusals carried forward; invalid first replay fixture retained. |
| MIS-ROSTER-01 | Passed | Carried forward: frozen original handles/lead/overrides/addendum/keys despite crew edits and remount. New fixture additionally preserves session IDs/keys and recorded badge flags. |
| MIS-FEED-01 | Passed | Rerun: actual lead goal recipient, untargeted empty goal and resolved/historical ask rendering after remount. Original authors/content/archive checks carried forward. Routine-event hiding is a native feed assertion; CLI `--all` supplies structured routine events. |
| Native Windows | Blocked | No native Windows host was available; CI cannot establish native behavior. |

UPG-1 owner-resume belongs in the PR's **waits for Jason** list under the explicit 04:05 UTC Runner instruction from Claude Code acting for Jason. Jason can resume one of his own development conversations. No full pass is claimed. The three non-blocking review follow-ups remain as recorded above.

### Preserved setup attempts and cleanup

The bare app's `ps comm` was relative (`target/debug/Runner`), so an initial absolute-path assertion failed before termination; its cwd was then verified with `lsof`, and only that owned build app was stopped. One native crew navigation opened a pre-existing crew's draft form after a search reset; QA canceled before submit, refiltered the unique owned name and verified the actual captain hint before starting. No pre-existing mission was started or crew changed. Longer native terminal text sometimes needed a second Return to submit. The captain instruction visibly reached the terminal as `--choices YESNO`, so this record uses that actual value and does not claim a two-choice captain test or a choice-parser defect. The solo A/B check covers multiple choices. These setup attempts and the bounded captain interruption remain in screenshots/conversations.

`ledger.json` lists two new roles (qa562v2-alpha/beta), the QA562v2 Native crew, two missions, three sessions, two folders, every owned app/daemon launch and the ordered offline fixtures. Roles, crew, scratch files, wrapper, logs and conversations are retained. Only owned trust entries were removed; `config-cleanup.diff` records those two removals and `final-config-baseline.diff` is empty. Final Codex configuration is byte-identical to its captured baseline. The wrapper was unregistered from LaunchServices without deleting it. No watch/load/helper was launched.

| Owned fixture | Mission ID | Cleanup |
| --- | --- | --- |
| QA562v2 Empty + solo | `01M4YY047S1FMNX76TK3D5Q506` | Archived at 05:02:26 UTC, zero live sessions |
| QA562v2 Crew + captain/worker | `01M4YY7AMAP05CN0AMXP227A7D` | Archived at 05:02:26 UTC, zero live sessions |

`cleanup-fixtures.json` and `cleanup-final-<id>.json` confirm archive timestamps and zero live sessions. `cleanup-owned-missions-absent.png` shows neither mission in active search. `cleanup-before-processes.txt` records the owned process tree; `cleanup-final-verification.json` and `cleanup-processes-final.txt` confirm no recorded PID, owned-path or development-path process survives. Final source/binary identity is in `candidate-final.json` at 05:03:18 UTC. Evidence paths in this second-run section are relative to `/private/tmp/runner-562-qa2-kid_9hnh/evidence/`; no private config or raw conversation is added to the repository. QA releases the frozen candidate and Cargo slot with this Incomplete verdict; no commit, push or PR was made by QA.

## Five-commit validation

After QA released the candidate, the initial brief commit was folded into the migration-runner commit and the implementation was shaped into the five authorized commits on current `origin/main` `cf839e169228b9081c5969aa344e3c2039e2b5f8`. Every cumulative stage independently built and passed the full workspace nextest command, normal and updater workspace Clippy, formatting and staged-diff checks. These checks ran sequentially in this worktree’s own target directory. No application was launched by coder.

| Commit stage | Workspace tests | Exit codes | Logs under `target/562-validation/` |
| --- | --- | --- | --- |
| 1. Foreign-keys-off migration runner | 2,330 passed, 6 skipped | 0 for every gate | `commit-1-*.log/.exit` |
| 2. Migration 0028 and schema | 2,331 passed, 6 skipped | 0 for every gate | `commit-2-*.log/.exit` |
| 3. Core, daemon and CLI | 2,342 passed, 6 skipped | 0 for every gate | `commit-3-*.log/.exit` |
| 4. Crew-only Start mission redesign | 2,343 passed, 6 skipped | 0 for every gate | `commit-4-*.log/.exit` |
| 5. No crew and empty workspace | 2,345 passed, 6 skipped | 0 for every gate | `commit-5-*.log/.exit` |

After history shaping, `git diff dd5fa7b34efaad630cfd08bfd6270cea534f1682 -- crates` still has SHA-256 `a40b79f9c5b456561004c396eb1acb295a89d106ea925f28cce3978ce76c5364`, exactly matching the second reviewed and native-QA-tested source. Only status/validation documentation changed after QA. The original failed attempts remain above; the final selected native verdict remains Incomplete.

## Windows CI follow-up

The first complete-source CI run, [38685888953](https://github.com/yicheng47/runner/actions/runs/38685888953), failed Windows Clippy before tests: four `#[cfg(windows)]` tests in `cli_install.rs` still passed `&str` to the newly optional crew argument. The job exited 1; its log is retained locally as `target/562-validation/ci-windows-38685888953.log`. The correction wraps those four existing test values in `Some(...)`, without changing test expectations or any production code. It returns to working-tree review before being folded into the backend commit. macOS CI passed (6m41s); Windows tests and resource checks were not reached in this attempt.

The initial five-commit source identity above predates this Windows-only test-call correction. Native macOS results remain applicable to unchanged production code, including the Incomplete verdict and UPG-1/Windows gaps. No additional native test is selected for an argument-type correction in Windows-only tests; Windows CI must compile and execute them before the PR is ready.

The first local full-suite check for this test-only correction again hit the unchanged OpenCode model-settlement timeout (2,344 passed, one failed, six skipped; exit 100, `commit-ci-fix-tests.log/.exit`). Its isolated retry passed, then the full suite (2,345 passed, six skipped), both Clippy commands, formatting and diff checks passed with exit 0 (`ci-fix-opencode-retry.log/.exit` and `commit-ci-fix-retry-*.log/.exit`). These retries did not fix the flake; Jason then explicitly requested that it be fixed as well.

## OpenCode test stability follow-up

Diagnostic full-suite runs reproduced the failure: the three-second success deadline expired after 3.018 seconds, with the fake server PID recorded but no API request recorded (`flake-diagnostic-2.log/.exit`, exit 100, 2,344 passed/one failed/six skipped). The other three diagnostic suites passed. This is consistent with subprocess startup contention in the full suite; the diagnostic did not establish whether expiry occurred while awaiting the handshake or starting the first API process. Temporary diagnostic instrumentation was removed.

The test now exercises successful settlement through the production `models::query` entry point and its normal fifteen-second budget, retaining the exact settled model, exactly two API calls and server-reaped assertions. The always-empty failure case has a three-second budget instead of 500 ms and accepts the two deadline outcomes the implementation can produce: `EmptyCatalog` between polls or `Timeout` inside a subprocess query. It still rejects success, invalid output and query failure. Clearing both marker files between scenarios and asserting a fresh API count greater than zero ensures that the failure scenario actually queried the new server; its new PID must also be reaped. Production query code and deadlines are unchanged.

The fix is test-only, alongside the Windows test-call correction. The selected native QA verdict and its gaps remain unchanged. Validation of the revised test uses three consecutive full workspace runs at default parallelism, followed by both workspace Clippy variants, formatting and diff checks; no retries or concurrency overrides are configured.

All post-fix checks passed with exit 0. Each of the three full workspace runs passed 2,345 tests with six skipped; the OpenCode process test passed in 5.592 s, 5.086 s and 4.217 s respectively. Normal and updater workspace Clippy, formatting and the working-tree diff check also passed. Logs and command exit codes are retained under `target/562-validation/flake-fixed-{tests-1,tests-2,tests-3,clippy,clippy-updater,fmt-check,diff-check}.log/.exit`. No load helper, watcher or app was launched; every validation command completed, and a final process check found no remaining owned test/build process. The fix returns to working-tree review before folding and pushing; Windows CI remains to be rerun.

Reviewer checked the complete three-file working-tree diff at `d0b27bda83b07642208f1c97f31bcf54777d148f` (SHA-256 `ffac8c46c3173ae80959b4a805bf609834654dc3d95cd6e112e8f47ca4458a64`) and reported **NO REMAINING MUST-FIX ISSUES** through Runner at 05:40:42 UTC. Both timeout outcomes, independent marker checks, Windows signatures and unchanged production code were reviewed. The fixes are folded into the backend commit; this evidence is folded into the fifth commit. The three existing non-blocking follow-ups and native QA gaps remain. CI results are tracked on [PR #906](https://github.com/yicheng47/runner/pull/906).

## Human-requested rendering follow-ups

At 05:57:28 UTC, Jason requested two small fixes before merging PR #906: lead-less restart confirmation must name the other members, and a running mission with no sessions must remain `running` in CLI list/show. The message explicitly waives live QA for these copy/rendering changes unless their scope expands. The existing selected native verdict remains Incomplete, with UPG-1 owner resume and native Windows still outstanding; no new native execution is claimed.

The restart dialog now passes an optional lead handle and has a separate no-lead sentence. Two tests pin the complete no-lead body and both unchanged crew bodies. CLI status rendering now distinguishes zero total sessions from a populated mission with zero live sessions. A unit matrix exercises both list and show for empty running missions, stopped/crashed members, mixed live/stopped members, and empty completed/aborted missions. The existing stopped-show fixture now contains a stopped member, and the existing list golden changes only the empty fixture's `stopped` label to `running`. No daemon, routing, schema, lifecycle or machine-output behavior changes. The help-golden cap remains the sole outstanding review follow-up once these fixes land.

Selected pre-review checks cover all runner-app and runner-cli tests because the changes affect their renderers and the CLI golden output. The first run had 916 passed, one failed and five skipped (exit 100): only the old list golden disagreed with the requested behavior. Its expected line was updated, and `built_cli_goldens::mission_list` then passed (one passed, 206 filtered/skipped, exit 0). Both workspace Clippy variants, formatting and the working-tree diff check passed with exit 0. Logs and exit files: `target/562-validation/human-fixes-{crate-tests,mission-list-golden,clippy,clippy-updater,fmt-check,diff-check}.log/.exit`. The three new unit tests passed in the affected-crate run. The full validation gates will run again after clean working-tree review and folding into commits three and five, as Jason requested.

Reviewer checked the complete six-file working-tree diff at `d4ad44ef5bbc1887a85dee80fff1894b325b8d7c` (SHA-256 `8d87fa3208ef3e7865c3cdc38b577862a391f07ee4b0c7ba50f15f1c5cf3a65c`) and reported **NO REMAINING MUST-FIX ISSUES** through Runner at 06:05:02 UTC. The two prior rendering findings are resolved; the help-golden cap is the sole remaining review follow-up. This verdict leaves the recorded native QA gaps unchanged. The CLI fix is folded into commit three and the restart-copy fix plus this record into commit five.

Post-fold validation passed on the reviewed source: 2,348 workspace tests passed, six skipped; every required gate exited 0. Only this validation evidence was appended afterward; the tested source is unchanged.

| Gate | Result | Log under `target/562-validation/` |
| --- | --- | --- |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 2,348 passed, six skipped; exit 0 | `human-fixes-postfold-tests.log/.exit` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | Exit 0 | `human-fixes-postfold-clippy.log/.exit` |
| Same Clippy command with `--features updater` | Exit 0 | `human-fixes-postfold-clippy-updater.log/.exit` |
| `cargo fmt --all --check` | Exit 0 | `human-fixes-postfold-fmt-check.log/.exit` |
| `git diff origin/main --check` (entire five-commit diff) | Exit 0 | `human-fixes-postfold-diff-check.log/.exit` |

The PR body removes the resolved restart-copy and empty-status follow-ups, retains the help-golden cap, and records CI for the pushed head. No live QA rerun was performed under Jason's explicit waiver for these two rendering fixes. Native QA remains Incomplete; no owned test/build/helper process remains after validation.
