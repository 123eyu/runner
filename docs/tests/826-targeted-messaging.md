# 826 targeted messaging — QA

Selected feature QA completed on 2026-10-17 with one Failed check: spontaneous human-broadcast delivery stayed held at a visibly empty sender prompt. A later native baseline comparison reproduced the same symptom before #826; it is a pre-existing behavior, with no feature-specific regression established by this run. Native recovery succeeded on both versions. The original check remains Failed, and its disposition belongs to coder and human rather than being silently converted to a pass. Scope is the reviewed working tree handed off by coder through Runner message `01M554BEB212D4JNAN92Q54SJQ`, after reviewer verdict `NO REMAINING MUST-FIX ISSUES`. This is not a full regression run. The mission brief and AGENTS.md Test Scope authorize bounded development-only Codex fixtures and native testing.

Candidate: branch `feat/826-targeted-messaging`, HEAD `70be1aca698f56a9a7f202696537e0d53a236c71`, base `045ead8440ddf16134ea0d902b040248f16c2ed4`, cumulative tracked patch plus untracked `addressing.rs` SHA256 `5c2725bf87c8448b8dd74f1ed5706819bda36302b5a05d570f5192cbc46013a9`. Evidence root: `/private/tmp/runner-826-qa.x95_v41r`. Source patch and metadata are in `evidence/candidate.patch` and `evidence/candidate.json`. The QA record itself is outside that source identity.

Environment: macOS 26.6.2 (25G83), arm64; Codex CLI 0.163.0, `gpt-6-luna`, low effort. The development endpoint is `/Users/jason/Library/Application Support/com.wycstudios.runner-dev/runnerd.sock`; the installed Runner is not a test target. Native control uses `cua_repl` bound to a dedicated development wrapper. Codex configuration was captured before fixture launches; only the run's pre-authorized scratch trust entries may be added and removed. Existing runtime settings/extensions remain in effect. No SQLite inspection or global configuration changes are authorized.

## Selected behavior and checks

| Check | Why selected |
| --- | --- |
| ADDR-1 | Proves single, repeated `--to`, leading-mention and broadcast routing, unread-only reads, digest, history and non-acking filters through real members. |
| COMPOSER-1 | Proves multi-chip, single-chip and no-chip sends and feed recipient rendering. |
| MIS-MSG-01 | Targeting changes must preserve submitted delivery during Working and Idle. |
| MIS-MSG-02 | Human conversion must retain selected recipients and broadcast semantics. |
| MIS-MSG-05 | Changed chip state and resolver must preserve the complete picker, caret, Escape, email, undo and Pinyin-preedit matrix, plus explicit-recipient precedence and mid-text semantics. |
| MIS-MSG-06 | Subscription changes must preserve first-turn/startup delivery, without altering the dispatcher. |
| MIS-ADD-01 | Newly joined members need correct membership boundaries in unread counts and every read mode across restart. |
| MIS-ASK-02 | Derived human answers must remain durable, directed and readable after stopped-slot resume and daemon restart. |

Required local runtime is Codex. OpenCode/Cursor are outside the explicitly permitted local runtime set and their where-available variant is skipped. Windows needs a native Windows host and is recorded separately as a platform gap; CI does not establish a native Windows pass. Pinyin preedit requires a genuine native composition path and cannot be represented by plain text insertion. Unrelated IME/runtime/lifecycle/product cases are outside this feature's scope except for the explicitly retained MIS-MSG-05 preedit check and the selected restart/resume checks.

## Automated evidence supplied by coder

The reviewed manifest `/tmp/826-review-manifest.json` reports exit 0 for workspace nextest (2413 passed, six skipped), both workspace Clippy variants (with and without updater), formatting, generated CLI reference and diff checks. QA inspected the corresponding logs under `/tmp/826-validation-*.log`; these remain separate from native results.

## Results

Required selected QA is not clean. Seven selected local rows passed and MIS-MSG-02 failed. Native Windows coverage remains Blocked, and the unauthorized OpenCode/Cursor variant is Skipped. Both owned missions are stopped and archived with zero live sessions; evidence and conversations are retained.

| Check | Result | Evidence / limit |
| --- | --- | --- |
| ADDR-1 | Passed | Single, repeated `--to`, leading mentions, peer broadcast, explicit precedence, unread/digest/history and non-acking filters validated in the four-member fixture. |
| COMPOSER-1 | Passed | Two chips, one chip and no chips with mid-text mention reached the expected members once; feed shows the selected recipients. |
| MIS-MSG-01 | Passed | Directed notices submitted during Working and Idle and covered by reads. |
| MIS-MSG-02 | Failed | Initial composer single/multi/broadcast cases passed; later human broadcast reached five of six members, with sender held as a draft at a visibly empty prompt. |
| MIS-MSG-05 | Passed | Full keyboard/click/caret/Escape/email/undo matrix and genuine underlined Pinyin preedit executed. Enter committed preedit without a chip or event; see `pinyin-boundary-contact.png` and identical pre/final event logs. |
| MIS-MSG-06 | Passed | First-turn planner delivery, later draft hold/clear and Codex added-role startup passed; unauthorized OpenCode/Cursor variant is Skipped. |
| MIS-ADD-01 | Passed | Native and CLI joins, tasks, join boundaries in every read mode, person-only refusals and persistence across daemon restart validated. |
| MIS-ASK-02 | Passed | Answer while stopped produced one warning, no ghost session and a durable directed answer read after restart/resume. |
| Native Windows variants | Blocked | No native Windows host is available; CI is not a native pass. |

### Finding: broadcast held at an empty sender prompt

On the unchanged candidate, human broadcast `QA826-JOIN-BROADCAST` (`01M55542S2N34A4TGE1P5WNWD6`, 2026-10-17 14:44:13.218951 UTC) appears once in the default reads of alpha, beta, spare, lateui and latecli, but not sender. The sender terminal shows “Inbox waiting — a draft in the composer is holding delivery” while the input visibly shows the empty “Ask Codex to do anything” placeholder. Its last helper run is turn 11 from the diagnostic human prompt completed at 14:42:57. No submitted notice for this later broadcast appears in its owned Codex transcript. Expected: all six members receive and read this human broadcast. Actual: five read it; sender remains held.

Evidence: `evidence/sender-broadcast-native.png`, `evidence/status-pinyin-final.json`, `evidence/events-pinyin-final.ndjson`, `evidence/sender/turn-011.json`, plus the failing all-six assertion in `assert-evidence.py`. The sender conversation is `/Users/jason/.codex/sessions/2026/10/17/rollout-2026-10-17T22-33-48-01a14a48-826d-7f63-84c0-776b3dc64960.jsonl`. At the initial report, no baseline comparison established whether this behavior predated the candidate. The later native baseline result below establishes the same symptom on the pre-feature build. Reported to coder through Runner; this failure remains failed even if clearing the owned prompt later releases it.

### Earlier suspected resume failure was withdrawn

An earlier empty/Idle display after resume was initially reported as stalled delivery. Exact timestamps show natural release before any recovery input: alpha ACKed at 14:42:43.626 UTC, beta at 14:42:43.753 and sender at 14:42:44.794. The diagnostic prompt was submitted later, at 14:42:55, and its helper read was empty. The approximately 64-second Starting hold remains an observation, with no baseline attribution or specified timeout violation. It is not recovery-dependent delivery. Preserve `daemon-owned-delivery.log`, `events-natural-resume-delivery.ndjson`, sender turns 9–11 and `sender-explicit-recovery-submitted.png`. The later broadcast failure above is a separate observation.

### Setup attempts retained

Original member turn 1 used a bare CLI resolved from inherited PATH and failed unsupported `--all`; the fixture was corrected to the absolute development sidecar before candidate assertions. Those startup attempts are excluded, with `fixture-setup-correction.json` retained. An initial unfocused single/no-chip composer attempt concatenated two markers; distinct `-R2` repetitions were verified. IME attempts before rebinding the restarted app or before source switching settled are not counted; the final underlined preedit/Enter pair is the accepted check. A plain-text `@ni` Enter attempt produced a recipient-validation banner and no message, separate from genuine preedit.

The source patch was rechecked unchanged at completion of live input checks and after cleanup. The original U.S. input source was restored (`input-sources-final.json`), and the owned unsent IME draft was cleared. Owned-object cleanup is complete, as recorded below.

## Failed delivery and native recovery

The fresh daemon excerpt `evidence/sender-drafting-before-clear.log` records `reason=Drafting` at 14:44:13.231103 UTC for sender, directly after the missing broadcast. It remains held until the native recovery control is used. `sender-before-clear-fresh.png` preserves the empty prompt and waiting banner before any clearing action. Existing CLI inspection exposes terminal metadata but no frame dump with cell styles; no new snapshot harness or configuration change was introduced. The native image and original Codex transcript are retained instead.

At 14:50:33.288 UTC, QA focused only the owned sender terminal and pressed Control-U. The immediate screenshot and `status-after-clear-broadcast.json` still show the hold, with no new helper read. At 14:50:54.181, QA clicked the visible native “Submit / clear” button. The daemon released Drafting at 14:50:55.945324 and confirmed delivery at 14:50:57.533079. Sender helper turn 12 began at 14:50:57.770256 and printed the broadcast once; its repeated default read was empty. The native screenshot shows one submitted `[inbox] 1 new messages` followed by the helper invocation and ACK. `sender-clear-action.json`, `sender-clear-button-action.json`, `sender-drafting-recovery.log`, `sender-after-recovery-complete.png`, `sender/turn-012.json` and `broadcast-before-after.json` retain that sequence. This is clear-control recovery, not a spontaneous-delivery pass. The initial failure lasted approximately 6 minutes 43 seconds before release.

The shortest reproduction supported by this run is:

1. Start the bounded Codex fixture, stop sender with a pending human ask, answer while stopped, restart the development daemon and resume the original conversation. Let its queued notices naturally finish.
2. At the owned sender prompt, use native CUA `typeText` to enter `Run python3 /private/tmp/runner-826-qa.x95_v41r/qa_worker.py read once. Then ACK QA826.` and press Return. This diagnostic prompt is a native terminal input, not a CLI session-write call. It completes at 14:42:57 and the screen returns to the empty placeholder. The earlier diagnostic click coordinates were not separately logged.
3. Send a human broadcast through the version-matched development CLI. Five other members read it; sender remains Idle with an empty placeholder and a Drafting hold. Preserve the log and native screenshot before clearing.
4. Control-U alone did not release this occurrence. Clicking Runner’s “Submit / clear” control released it and produced the covering read.

This is the observed sequence, not a claim that every preceding step is necessary. The initial run did not establish the minimal trigger or candidate attribution. `crates/runner-daemon/src/session/manager/mod.rs:626` is the readiness path mapping a model draft hold to Drafting; this is a behavior pointer, not a proven cause. Coder reported those tracker/gate/dispatcher sources unchanged from the base. The later native comparison supplies the previously missing behavioral baseline, while the exact event-order cause remains unproven. The Failed result is retained and required selected QA is not represented as an unconditional clean pass.

## Routing and read evidence

The four-member fixture began with sender, alpha, beta and spare. Each real agent ran `qa_worker.py` once per submitted notification, recording command arguments, exit status, stdout/stderr, timestamps and acknowledgments. The helper performs a default read, another default read and `--all`; special bounded phases additionally exercise filters and history acknowledgment. `assert-evidence.py` checks the event IDs against those recorded reads. The initial all-six assertion failed before recovery; `broadcast-before-after.json` preserves that original Failed result separately from the later successful read. The post-recovery assertion run exited 0, but does not change MIS-MSG-02’s result.

| Exchange | Event ID | Observed default readers, exactly once |
| --- | --- | --- |
| Peer single `--to alpha` | `01M554J2V5KYDFBH4CDSP4JT8V` | alpha |
| Peer repeated `--to alpha --to beta` | `01M554JRTJA4W8V7559K7FYX9M` | alpha, beta |
| Peer leading `@alpha @beta` | `01M554KF762EBFCC0XHGC1MZP2` | alpha, beta |
| Peer broadcast | `01M554MCJ01GBKCH95JC4DSBE8` | alpha, beta, spare |
| Explicit alpha overrides leading beta | `01M554M0AY4G5RW5WGP2DP0TB1` | alpha |
| Two-chip human composer | `01M554KKCKFGZD73604KQQGDJK` | alpha, beta |
| One-chip human composer, R2 | `01M554N22PPS2E3VB3SBHS34SB` | alpha |
| No-chip human composer with mid-text beta, R2 | `01M554N4Q60VGVFDP6Z6XVPME0` | sender, alpha, beta, spare |
| Directed to both added members after restart | `01M5553MN8HXD0YMPQF63SKQGA` | lateui, latecli |

Spare had no directed-message work and stayed at its startup turn until the peer broadcast. Native `alpha-native-delivery.png`, `composer-*-before.png` and `composer-*-feed.png` show actual submitted notices, recipient chips and feed rendering, rather than only a running process or visible ACK. The two-chip composer send occurred while alpha/beta were handling preceding work; their subsequent notices were submitted and read once. Default read output and acknowledgment comparisons are in `assertions-after-recovery.json` and per-member `turn-*.json` files.

For beta’s unread A/B case, A is `01M554RD4G6CHQ49ARMW6YMS3Q` and B is `01M554RD5XWVHNMCTSXNFA6PZB`. `beta/turn-007.json` proves `--since A` prints B, all four `--since`/`--from` variants with and without `--all` leave the acknowledgment unchanged, and the following default read still prints A and B. Its repeated default read is empty. `alpha/turn-013.json` proves `--all` includes later other-member traffic but acknowledges only `QA826-HISTORY-OWN` (`01M554WEC1RWY9Y4D89YHZGG05`), not `QA826-HISTORY-OTHER-LATER`. Sender’s `--all` includes its sent traffic as well as messages between other members. The bounded digest appears separately from inbox messages; it never substitutes for an inbox read. All 29 recorded default reads containing a digest have at most ten entries, with the exact ten-entry cap observed (`digest-cap-check.json`).

## Complete composer selection matrix

All four pick methods were exercised: Tab, Enter, Space on the exact handle and native click. The mid-text cases ran with and without an existing chip. Leading cases ran at the start with and without an existing chip, including all four methods before existing trailing text. Mid-text picks completed text with one space and added no recipient; leading picks added a distinct chip and retained trailing text without a leading space. Existing chips survived another pick; a duplicate handle added no second chip. The before-text cases also cover the leading-with-existing-text boundary.

Arrow and click caret movement closed the picker away from the token and reopened it on return. Escape kept the same token dismissed, `me@host` opened no picker and undo restored the typed token. These observations are retained in the individually named `picker-*.png` files and inspected contact sheets `picker-contact-00.png`, `picker-contact-10.png` and `picker-contact-20.png`.

For genuine preedit, QA used the already configured macOS Pinyin source, physical `n`/`i` key events after `@`, and confirmed the marked underline. `pinyin-boundary-preedit.png` shows the picker hidden during marked text. Return removes the underline while retaining `@ni`, with no recipient chip or send. `pinyin-boundary-contact.png` shows the before/after pair, and the mission event log is byte-identical between `events-pre-pinyin-enter.ndjson` and `events-pinyin-final.ndjson`; member turn counts also remain unchanged. A separate plain-text attempt without a settled input-source switch is retained as an invalid attempt, not an IME pass. The final accepted check uses the underlined pair. U.S. was restored and verified afterward.

## Startup, joining, answer and persistence evidence

The separate startup fixture had planner and startupworker. Worker message `01M554WDTXXVHP3VE1V8NAAQF2` was created at 14:40:02.397 UTC during planner’s first turn; planner first became Idle at 14:40:03.369 and began handling the queued notice at 14:40:03.533. Its covering read is at 14:40:04.936, after the first turn, shown in `planner-native-delivery.png` and `planner/turn-001.json`. A later owned single-character draft held `QA826-LATER-DRAFT` (`01M554YEZGJ33SNQHQCEB1C8H2`) until it was cleared; `planner-later-draft-held.png`, `planner-later-draft-cleared.png` and planner turn 2 retain the release. Planner also emitted an extra fixture ACK message despite its role instruction; that fixture deviation is preserved and is not the proof of initial delivery.

Lateui was added through the native role form with `QA826-UI-TASK`; latecli was added through the development CLI with `QA826-CLI-TASK`. Their join boundaries are `01M554VEK5BJR1JBH8X51MK1WX` and `01M554VNHE3AY8VYBSAMK95PFY`. Their recorded default, `--all`, `--since`, `--from` and combined modes print only post-join message IDs, even with a cursor before joining; filtered reads leave acknowledgments unchanged. Task messages and introductions were visible, and direct messages after restart reached both once. In-mission role-add and `--as` bypass attempts exited 2 without adding a member. The roster, task overrides and original conversation keys survived daemon restart; `status-joined.json`, `status-pre-restart.json`, `status-restarted-before-resume.json`, `late-member-after-restart.png` and both late-member record directories retain the evidence.

The pending ask was `01M554P7CRFN8AXZR99WRMG2DA`; the answer message was `01M554RTW4QTD5QTJ4MTBV29B6`, from human to sender with its question ID. Answering through the native Proceed control while sender was stopped produced one queued-until-resume warning, cleared the ask and created no ghost live session. The feed displayed the response once and hid its derived inbox copy. Sender turn 9 read the durable answer after restart/resume; the history/ack assertion confirms it was default-read once. See `stopped-ask-answer-warning-r2.png`, `events-answer-stopped.ndjson`, `status-answer-stopped.json`, `cursor-resume-sender.json` and the earlier natural-release timeline above. Original-conversation resume was required and checked; new-conversation rekey is outside this targeting change’s selected scope.

## Build, commands and test boundaries

QA launched `make run` with inherited mission identity unset, then controlled a dedicated temporary development wrapper using the freshly built app and sidecar. The old owned launch was stopped; the installed app and daemon were never test targets. The app hash is `f8bdb3b96a62f6d7b964ae1b77fc2053fcdf5953a8d860482c6512d32faffe59`; development agent CLI/daemon sidecars are `1b9b81619a0a7e4075105f4428cd1c7dddf00835484464a1c26ade4c18e8ab51`. `build-hashes.json`, `installed-hashes.json` and `make-run.log` tie the launched artifacts to the reviewed worktree. Runner reports 0.16.4. Codex retained the existing account’s fast speed setting shown in the terminal; QA did not change it.

Every external fixture command used `/Users/jason/Library/Application Support/com.wycstudios.runner-dev/bin/runner`; `dev_cli.py` unsets inherited crew/mission/handle/event-log identity and records each exact command/result in `cli-*.json`. Agent helper commands use that same absolute sidecar with each agent’s actual mission identity. Operations included version-matched `mission start/show/add-role/resume/stop/archive`, session stop, daemon stop, message post/read, native human answers and the selected native composer actions. Agent read records retain exact flags and exits. `snapshot.py` independently captured structured mission status and copied owned event logs by phase; native images establish actual UI behavior.

Relevant existing tests inspected include the addressing resolver tests, `runner-cli/src/msg.rs` read-mode/history/digest/stale-cursor tests, mission composer pick/caret tests, and daemon role-join/remount tests. Coder supplied the full automated gates; QA inspected their logs rather than rerunning the unchanged workspace suite. Logs are `/tmp/826-validation-nextest-final.log`, `/tmp/826-validation-clippy-reviewed.log`, `/tmp/826-validation-clippy-updater.log`, `/tmp/826-validation-fmt-final.log`, `/tmp/826-validation-guide.log`, `/tmp/826-validation-diff-final.log` and `/tmp/826-review-fix-diff-check.log`. The manifest records each exit as 0, with 2413 passed and six skipped tests. Those automated results do not resolve the native failure or native Windows gap. No implementation code, frozen expectations, runtime configuration or SQLite data was altered to obtain a pass.

## Cleanup ledger

| Owned object | Final state |
| --- | --- |
| ADDR/composer mission `01M554GXD33QD1H3S4XWCSAJNX` | Stopped, archived at 14:51:34.781072 UTC, completed, zero live sessions. |
| Startup mission `01M554W47Y5CRVTG2N2Y43FK7M` | Stopped, archived at 14:41:21.770069 UTC, completed, zero live sessions. |
| Six ADDR agents and two startup agents | Stopped through their owned missions; conversations and event history retained. Prior live ADDR PIDs are absent. |
| Dedicated wrapper PID 82199 and original owned app/build launches | Wrapper stopped and unregistered from Launch Services; all three recorded launcher/app PIDs absent. Files retained under the scratch root. |
| Two test crews and roles | Retained as metadata/evidence; no destructive deletion. IDs are in `ledger.json`. |
| Codex trust entries for scratch `addr` and `join` | Only those two entries removed; final config is byte-for-byte equal to captured pre-run config. |
| Native input source and draft | U.S. restored; only the owned unsent composer draft cleared. |
| Existing development sessions and installed Runner | Never typed into or archived; development daemon left running to preserve unrelated sessions. |

`cleanup.json`, `cleanup-addr-show.json`, `cleanup-startup-show.json`, `cleanup-process-verification.json`, `config-cleanup.json`, `input-sources-final.json` and `native-cleanup-archived.png` verify cleanup. Candidate identity remains unchanged in `candidate-final.json`; only this QA record was authored by QA. No commit, push, PR, branch switch or baseline build was performed. Further work requires coder’s disposition of the open draft-hold finding; native Windows remains an explicit unresolved platform prerequisite.

## Separately identified baseline comparison — initial attempt Blocked

Coder requested a bounded attribution comparison through Runner message `01M555TBFMBA0PXBQ0M0T9J6XA` at 14:56:23 UTC, after the candidate run and its cleanup. The original MIS-MSG-02 result remains Failed. This baseline diagnostic did not reach the native workflow and establishes neither reproduction nor non-reproduction; attribution remains unresolved.

Coder prepared the baseline in the same worktree and branch. HEAD remained `70be1aca698f56a9a7f202696537e0d53a236c71`, while all tracked crate sources were byte-identical to `045ead8440ddf16134ea0d902b040248f16c2ed4`. The untracked `addressing.rs` and QA record were retained; baseline `lib.rs` does not reference the untracked module. QA independently verified the empty tracked crate diff and did not restore the feature patch or switch candidates. Coder’s reversible feature patch is `/tmp/826-baseline-restore.patch`, SHA256 `bc583435647032d3616fa24fc3c77e58c6ad97f7219912d7093f905af1bcf70c`. Coder’s build command `cargo build --locked -p runner-app -p runner-cli` exited 0, with `/tmp/826-baseline-build.log` and `/tmp/826-baseline-build.json` retained in the baseline evidence directory.

Baseline evidence root is `/private/tmp/runner-826-base.egxrpa45`. App SHA256 is `1faeb06b9418a20688a323de22ce953e207899688e7271755b1db02f43bfcb60`; agent CLI/daemon sidecar SHA256 is `8036e0630a603d8cf9fb1f8ff94fd2e0daeed1ef2f618d004c9e5c418c98598b`. QA stopped the previous candidate development daemon before launch, then verified both the launched wrapper app and installed development `bin/runner` and `bin/runnerd` against these baseline hashes (`evidence/launched-hashes.json`). CLI status confirmed the development namespace and socket. No additional build was run by QA.

Before creating agents, QA prepared a fresh canonical `/tmp` fixture folder, a baseline-compatible helper using one ordinary `msg read` and only a single-target trigger plus bounded human ask, and a two-member sender/control role prompt. The helper contains no `--all`, repeated `--to` or unread-only assumption. No role, crew, mission or test agent was actually created because native control failed at window binding.

The first baseline wrapper process started at 14:57:19 UTC and its app log recorded restored window geometry of 1865 × 1031 logical points. Native `cua.getApp` returned `cgWindowNotFound` for both the bundle ID and exact bundle path, despite inventory showing the wrapper running. Registering the bundle and relaunching through the native app API did not provide a bindable window. QA then reused only the previously working owned `Runner QA826 Dev.app` identity, preserving its prior candidate binaries as backups in the new evidence root and verifying the replacement wrapper binaries were the baseline. A fresh native-control session still returned `cgWindowNotFound`. The documented optional `cua.computer.launch_app` API was unavailable on this host. There was no bound native window in which QA could verify the endpoint and type; no diagnostic prompt, human broadcast, stopped-slot answer/resume sequence or recovery action was attempted on the baseline. A live CLI status or running process does not establish a native pass.

| Baseline diagnostic check | Result | Evidence / boundary |
| --- | --- | --- |
| Exact baseline source and app/development-sidecar identity | Passed | `build-identity.json`, `launched-hashes.json`, version-matched CLI status and empty tracked crate diff. |
| Native window prerequisite | Blocked | Repeated `cgWindowNotFound` after the scoped setup changes above; `native-binding-setup.json`, app/daemon startup excerpts and tool record. |
| Two-member stopped-answer/restart/diagnostic-input/broadcast comparison | Blocked | Zero workflow attempts; no fixture agents created. The two-attempt ceiling was not consumed by a valid diagnostic run. |
| Control-U and native Submit / clear recovery on baseline | Skipped | No reproduced baseline hold and no bound native window. |
| Original candidate failure attribution | Blocked | This comparison provides no behavioral baseline evidence; do not classify the original failure as pre-existing or fixed. |
| Baseline owned-object/configuration cleanup | Passed | Both owned wrappers stopped and unregistered; no baseline missions/roles/crews created; config byte-identical to before. |

Cleanup finished at 14:59:00.715135 UTC. The first wrapper PIDs 88216 and 88705 had already been stopped during setup; the final owned wrapper PID 89050 was stopped and verified absent. Both temporary wrapper registrations were removed. Prepared helper files, binary backups and logs are retained. There were no new scratch trust entries to remove, no input-source change and no native test conversation. Existing development sessions were allowed to resume with the development daemon under standing authorization but were never typed into or archived. The development daemon remains running to preserve those sessions. `evidence/blocked-cleanup.json`, `ledger.json` and final process verification retain the cleanup facts.

The baseline tracked source and target binaries were left unchanged. It is safe for coder to restore the saved feature patch and rebuild in this same worktree; QA has no running wrapper, build or fixture process using those artifacts. Before a later candidate live run, its QA operator must stop the still-running baseline development daemon and verify the rebuilt candidate app and sidecars again. The candidate finding and platform gap remain unresolved; this blocked attribution comparison does not supply a clean QA verdict.

## Coder-provided offline race diagnostic

Runner message `01M5566JZ8K4Y6RJCHXP6GN97K` supplied a separate deterministic diagnostic. `/tmp/826-submit-race-diagnostic.json` identifies source `/tmp/826-native-submit-probe.rs` and candidate/base logs `/tmp/826-native-submit-probe.log` and `/tmp/826-native-submit-probe-base.log`. Both commands exited 101: `native_submit_before_echo_releases_when_the_prompt_is_empty` expected Idle but observed Drafting when native Submit occurred before the first paste echo. Coder removed the disposable test afterward, with no production change. QA inspected the manifest and log endings, without executing or modifying this diagnostic. It demonstrates a legacy race class on both code versions and is a plausible cause of the observed native hold; it does not prove the exact native event ordering or resolve the original Failed result.

## Baseline comparison resumed after human restored native access

The human explicitly requested continued testing after the window prerequisite was restored. Native control successfully bound the owned wrapper again. During the earlier pause, coder had restored the exact reviewed feature source patch and rebuilt target artifacts; QA independently rechecked source SHA256 `5c2725bf87c8448b8dd74f1ed5706819bda36302b5a05d570f5192cbc46013a9`. The live baseline comparison uses the preserved baseline wrapper and installed development sidecars, whose hashes still match the baseline build above. It does not run the rebuilt feature artifacts and requires no source switch. `baseline-resume-hashes.json` verifies app, CLI and daemon after restart. The historical Blocked attempt above remains preserved.

The baseline fixture is mission `01M55676BZSYEMAQP8EEHK4QNE`, crew `01M556767W3MZVZZ75E71WV7DY`, role `01M556766M0CM1VGYF44W1VN37`, with sender session `01M55676C1BY8EY9B26YHTMXMD` and control session `01M55676C1Q34B6FETR5C8MP8P`, both in the previously empty fresh scratch `fixture` directory. It uses Codex `gpt-6-luna`, low effort and inherited fast speed. Native image dimensions are 3730 × 2062 pixels, matching the original candidate. A mistaken resize attempt used an out-of-window coordinate and made no change; dimensions were then verified from the image. The helper uses baseline-supported history reads, not feature unread semantics. Agents emitted a few extra directed fixture ACK messages despite the role’s tool restriction; these are retained and are not evidence of spontaneous broadcast delivery.

Sender created question `01M5568F2B134TD5VM36D8JK5R`, was stopped, and the native Proceed card was answered while stopped. One queued-until-resume warning appeared. QA stopped the owned wrapper and development daemon, relaunched the preserved baseline wrapper and resumed sender’s original key `01a14a63-a3f9-7cd0-a6a7-6224e7d6fc7b` at 15:04:46 UTC; control retained key `01a14a63-a58e-7f90-ae73-315ae4dabd81`. No diagnostic input was used to release its queued answer. Starting released at 15:05:46.783152, and sender turn 3 read the answer at 15:05:49.851549. Two bounded native diagnostic/broadcast trials then used this same resumed fixture; the second reused the stopped-answer/restart preparation rather than creating another mission.

### Native baseline result: same symptom reproduced on trial 2

| Check | Result | Evidence |
| --- | --- | --- |
| Restored native access and verified preserved baseline app/CLI/daemon | Passed | Successful native binding, matching hashes in `baseline-resume-hashes.json`. |
| Stopped human answer, daemon restart and original-key natural resume | Passed | Native answer card, queued warning, original keys, sender turn 3 and owned daemon log. |
| Trial 1 spontaneous human-broadcast delivery | Passed | Both members submitted a notice and read `01M556CRDDNHR1NM86S9K1RWYX` naturally. |
| Trial 2 spontaneous human-broadcast delivery | Failed | Sender held at empty input; control read `01M556FB7TE5WFZJG29MK66979` while sender had no covering read. |
| Control-U recovery | Failed | Native action did not release this hold. |
| Native Submit / clear recovery | Passed | Hold released and sender read trial 2 afterward. This does not replace the Failed spontaneous-delivery result. |
| Baseline cleanup | Passed | Mission archived, zero live sessions, owned PIDs absent, config restored. |
| Initial blocked native attempt | Blocked, historical | Retained above; prerequisite subsequently resolved by human. |
| Additional runtime/platform variants | Skipped | Outside this two-member macOS Codex attribution diagnostic; original Windows gap is unchanged. |

Both native inputs used CUA `typeText` followed immediately by Return in the same call, after clicking the owned sender input at `[1300,1738]`. The exact text was `Run python3 /private/tmp/runner-826-base.egxrpa45/qa_worker.py read once. Then ACK QA826.` Native empty-prompt images were captured after completion before posting each broadcast. The recorded Return times were 15:06:10.054–10.097 and 15:06:55.590–55.645 UTC; `baseline-diagnostic-native-action.json` and `baseline-diagnostic-trial2-action.json` preserve those inputs.

Trial 1 broadcast `01M556CRDDNHR1NM86S9K1RWYX` was posted at 15:06:26.093920. Control’s covering helper read began at 15:06:27.457732 and sender’s at 15:06:27.725050, with actual submitted native notices. Trial 2 broadcast `01M556FB7TE5WFZJG29MK66979` was posted at 15:07:50.906754. Control read it at 15:07:52.347734; sender stayed at helper turn 6 with no submitted notice or covering read. The daemon recorded `reason=Drafting` at 15:07:50.969869. `baseline-trial2-held-empty-before-clear.png` shows the waiting banner and the empty “Ask Codex to do anything” placeholder. `trial2-before-clear-failed.json`, `events-trial2-held-before-clear.ndjson`, `status-trial2-held-before-clear.json`, owned before-clear conversations and `baseline-drafting-before-clear.log` preserve this Failed state independently of later recovery.

QA pressed Control-U at 15:08:46.222; the banner, hold and missing read remained, including in the later `after-control-u` snapshot and native image. QA clicked the native Submit / clear control at 15:09:36.356. Drafting released at 15:09:38.140106, and sender turn 7 read the broadcast at 15:09:39.668643. Exact actions, before/after images, `baseline-drafting-recovery-final.log` and the covering read record are retained. `baseline-final-results.json` preserves trial 1 Passed, trial 2 Failed and recovery separately. Expected baseline history reprints in later helper reads are not duplicate-delivery failures.

This comparison now establishes that the native empty-prompt Drafting symptom exists on the pre-feature baseline, including the same ineffective Control-U and successful native-control recovery seen in the candidate. It does not establish the exact native paste/echo/Submit ordering, whether the feature changes the frequency, or that every original preparatory step is necessary. The coder-provided synthetic race remains a plausible mechanism rather than the proven cause. No additional #826 implementation defect was demonstrated. Original MIS-MSG-02 stays Failed; no unconditional `NO REMAINING MUST-FIX QA ISSUES` verdict is issued for a matrix that still contains that required failure and the native Windows gap.

The human’s explicit request to test after restoring native access preceded the later coder message still assuming the native prerequisite was blocked. The single owned baseline fixture was already running when that message arrived; QA finished only the existing bounded comparison and cleanup, without another mission or additional agents. The historical Blocked result and coder-provided diagnostic remain separately identified above.

### Baseline live cleanup

Mission `01M55676BZSYEMAQP8EEHK4QNE` was stopped and archived at 15:10:45.027622 UTC, with completed status and zero live sessions. Sender PID 92656, control PID 92026 and owned wrapper PID 91893 are verified absent; the preceding reopened wrapper PID 89969 is also absent. The wrapper was unregistered. Role and crew metadata, event history, conversations, helper files and binary backups are retained. Only `/private/tmp/runner-826-base.egxrpa45/fixture` was removed from Codex project trust; the resulting config is byte-for-byte equal to the pre-run capture. No input-source change occurred in these baseline trials. Existing development sessions and installed Runner were never typed into or archived.

`baseline-live-cleanup.json`, `baseline-live-process-verification.json`, `status-archived-final.json`, `events-archived-final.ndjson` and `baseline-native-archived.png` retain cleanup evidence. The source patch remains exactly the reviewed `5c2725bf87c8448b8dd74f1ed5706819bda36302b5a05d570f5192cbc46013a9`; QA did not switch source candidates, rebuild, commit or publish. The development daemon remains baseline to preserve unrelated saved sessions, while coder’s target artifacts remain the restored feature build. Any future candidate live check must explicitly stop that baseline daemon and verify the candidate app and installed sidecars. No QA wrapper, fixture or build process remains active.

## Publication disposition

After the native baseline reproduction and reviewer’s final verdict of `NO REMAINING #826 IMPLEMENTATION MUST-FIX ISSUES`, Jason explicitly instructed coder, “Right, open the pr.” This authorizes publication with the documented pre-existing MIS-MSG-02 failure and native Windows gap retained. The failed result is not relabeled as Passed, readiness remains outside this mission, and the exact tracker mechanism and comparative frequency remain unproven.

## PR #939 review follow-up — 2026-10-18

Jason requested three corrections through Runner message `01M56AK3G0VBR4HH16S0FH79T3`: recognize leading address tokens with one optional trailing colon or comma, render multiple recipients as names in the CLI feed, and group unconverted `human_said` signals with human messages to the same recipients. The shared resolver now stops at any token outside `@[a-z0-9][a-z0-9_-]*[:,]?`, leaving possessives and bare `@` as text. Unknown handles and `@human` in valid address tokens remain errors; explicit recipients still win. The roster-sidecar requirement is unchanged.

Selected automated follow-ups cover the resolver grammar and unchanged text, actual CLI posts and daemon `mission_post` agreement, single/multiple/broadcast recipients in ordinary and streaming CLI feed output, and native feed grouping for string, array and null targets. The spec, plan and agent guide describe the grammar. Validation results and the reviewed candidate identity are recorded below.

Coder selected a focused COMPOSER-1 rerun after clean review to verify punctuation, possessives, explicit chips and recipient notifications through the development composer. The full ADDR-1 exchange is not selected again because subscriptions, read cursors, acknowledgments, digest and dispatcher behavior are unchanged; resolver and CLI/daemon integration tests cover the changed addressing grammar. The native feed grouping case uses deterministic legacy-event fixtures instead of constructing an unconverted live log. The original MIS-MSG-02 failure, baseline reproduction, #940 follow-up and native Windows gap remain as recorded.

Follow-up validation: focused nextest passed five tests (exit 0); full `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` passed 2416 tests with six skipped (exit 0). Workspace Clippy with and without `--features updater`, `cargo fmt --all --check`, `make guide-cli`, `make guide-cli-check` and `git diff --check` all exited 0. Each command’s output and exit code are retained in `/tmp/826-followup-validation-*.log` and `/tmp/826-followup-validation.json`; focused evidence is `/tmp/826-followup-focused.log`. CLI help and prompts did not change; regeneration left the CLI reference unchanged. MIS-MSG-02’s existing regression row retains its previous cases and now includes punctuation, possessives and bare `@` expectations.

Candidate for follow-up review: branch `feat/826-targeted-messaging`, HEAD `98137f2fd36fafcaf3f1c4b1fd47830b40fea2cb` plus the uncommitted follow-up. SHA256 of `git diff --binary 045ead84 -- crates` is `fe4d070d29c56fbf681cef0abf70cd2fbbfe70261df2d694a5a9bfe46f2cba3d`; this identifies code independently of later QA-record edits. At that review handoff, working-tree review and the selected focused COMPOSER-1 rerun were pending; no fixup, push or PR update had been made for this candidate.

### Follow-up QA selection and identity

QA received explicit post-review handoff `01M56B3QH27XENAE3WQ6WB1ZG0` after reviewer verdict `01M56B2TJY5CYQ0TH5C0WTKPT7`, `NO REMAINING MUST-FIX ISSUES`. Independently verified branch `feat/826-targeted-messaging`, HEAD `98137f2fd36fafcaf3f1c4b1fd47830b40fea2cb`, code diff SHA256 `fe4d070d29c56fbf681cef0abf70cd2fbbfe70261df2d694a5a9bfe46f2cba3d` against `045ead84`, and saved review-patch SHA256 `3995e5a0b7ed7afaf53e6886e333977a2bf4dd42a700aeca201d664f281e6092`. Fresh evidence root is `/private/tmp/runner-826-followup-4llvwap8`; initial config, candidate patch/metadata, review manifest and automated logs are retained there. The existing worktree changes belong to the reviewed follow-up; QA changes only this record.

Selected checks: leading colon/comma addresses route only to coder; mixed punctuation addresses both coder and qa; possessives and bare `@` broadcast; an explicit qa chip overrides a leading coder token; unknown punctuated handles and human addresses fail without append or wake. These cover the changed grammar through the native composer, rendered recipients and actual per-member reads. Multi-target CLI feed text must use recipient names while JSON retains its array. Legacy human-signal grouping is covered by the supplied deterministic null/string/array fixture, without live-log manipulation. Full ADDR-1, unchanged picker/IME/restart flows and the #940 diagnostic are not selected again. Historical failure and platform/runtime gaps remain separate.

### Follow-up native results and evidence

The selected macOS Codex follow-up passed with no new implementation finding. This scoped result does not replace the historical MIS-MSG-02 Failed result or resolve #940 or native Windows coverage. The reviewed code SHA256 remained unchanged through cleanup. `assertions.json` verifies exact text, recipient shapes, one covering read per addressed member, no covering reads by non-recipients, and an unchanged event log for each rejected send.

Fresh `make run` compiled the CLI and app and replaced the prior baseline development daemon. The owned make app was then stopped and copied with its sidecar into `/private/tmp/runner-826-followup-4llvwap8/Runner QA826 Followup.app`, bundle ID `com.wycstudios.runner.qa826-followup-dev`. Launched app SHA256 is `22be0775006b87092fde73840e4aab43c176ee2deb84ff71d21fc891bfa9ea7a`; CLI and installed development `runner`/`runnerd` SHA256 is `c949c31f5af630fa9cabc6ecc0b965b1eea3e8623e444c297a8cd00b5363b563`. Native binding and opening/dismissing New chat succeeded before fixture input. Structured status verified the development socket. macOS 26.6.2 (25G83), arm64, Codex 0.163.0, `gpt-6-luna`, low effort, inherited fast speed; colors were visible and `NO_COLOR` omitted only from child launches. Required automated gates were inspected from the supplied logs and copied to evidence, not rerun unchanged.

Only mission `01M56B7XGG1QVR432M1AVAXEPK`, crew `01M56B7XC7EJAYBT6K01DM0P76` and role `01M56B7XAXFEBWNSRNB2DS8QA0` were created. Coder session `01M56B7XGJ3A53X2CV1VVGNME6` retained key `01a14cb4-0c7a-76e0-ac9d-75b40efcfc7d`; qa session `01M56B7XGKNEV77TEBES7M4C0V` retained key `01a14cb4-03e3-77c3-88c7-c6e4df228ff7`. Both used the same fresh canonical scratch `fixture` directory. Their startup and each submitted inbox notice ran one helper invoking the absolute development CLI’s `msg read` and recording its result; test bodies were treated as data. Human test messages were entered only in the mission composer through native CUA `typeText` and Send. No message bodies were typed into agent terminals.

| Native composer text | Message event | Actual default readers | Result |
| --- | --- | --- | --- |
| @coder: QA826F-COLON | `01M56B9BPTZD9Q6RYYX41M2BPE` | coder | Passed |
| @coder, QA826F-COMMA | `01M56BA2SGQ61TEXC2HJZZ79S5` | coder | Passed |
| @coder, @qa: QA826F-BOTH | `01M56BAJX6M8VEJEPRYABEV0ZC` | coder, qa | Passed |
| @impl's fix QA826F-IMPL-POSSESSIVE | `01M56BBAFVEET6QJH3TMJN29X5` | coder, qa | Passed |
| @reviewer's audit @qa QA826F-REVIEWER-POSSESSIVE | `01M56BBVQPA7G2NVVW5541NKM0` | coder, qa | Passed |
| @coder's fix QA826F-KNOWN-POSSESSIVE | `01M56BCA90NZ0XKJQEM7XGFKN1` | coder, qa | Passed |
| @ QA826F-BARE | `01M56BCS90KVH58C4KCV453VTW` | coder, qa | Passed |
| @coder: QA826F-EXPLICIT-QA | `01M56BDWBNZXA209ZMV1VQY0Y0` | qa | Passed |

| Selected follow-up check | Result | Evidence / limit |
| --- | --- | --- |
| Leading colon and comma addresses | Passed | Coder turns 2 and 3; qa remains at startup turn 1 through both sends. Text and native feed recipients preserved. |
| Mixed punctuation with both members | Passed | Coder turn 4 and qa turn 2 read the same array-addressed event once; native feed shows coder and qa. |
| Possessives and bare `@` without chips | Passed | Four broadcasts, including nonexistent impl/reviewer possessives, known coder possessive and bare `@`; both members read each, with later `@qa` after a possessive remaining plain text. |
| Explicit qa chip overrides leading coder | Passed | Native `@qa` picker/Tab chip, unchanged `@coder:` body; only qa turn 7 reads it, coder stays at turn 8. |
| Unknown and reserved recipients rejected | Passed | `@missing:`, `@missing,`, `@human`, `@human:` and `@human,` each show an error naming valid coder/qa handles, retain the input and append no event. Five event logs are byte-identical to the pre-error snapshot; coder/qa stay idle at turns 8/7. |
| Actual native notices and recipient-only work | Passed | Owned Codex conversations contain exactly seven submitted inbox notices for coder and six for qa, matching seven and six post-startup helper reads. Native terminal images show submitted notices, helper execution and ACK, not only processes. |
| CLI feed recipient rendering | Passed | `feed-final.txt` says `human -> coder, qa`; `feed-final.json` retains `["coder","qa"]` and unchanged message text. |
| Legacy unconverted human-signal grouping | Passed (automated) | Supplied focused fixture test covers null/string/array and passed; no live unconverted log or database manipulation was selected. |
| Native Windows | Blocked, existing gap | No native Windows host; this macOS follow-up does not supply native Windows evidence. |
| Full ADDR-1, unchanged picker/IME/restart flows, #940 diagnostic | Skipped | Explicitly outside this focused handoff. Historical results and failure remain above. |
| Other runtime variants | Skipped | Not selected; local authorization and prior gaps are unchanged. |

Evidence includes all before/send screenshots, `punctuation-contact.jpg`, `broadcast-contact.jpg`, `explicit-before-crop.jpg`, `errors-contact.jpg`, `coder-native-delivery.png`, `qa-native-delivery.png`, helper `coder/turn-001` through `turn-008.json`, qa turns 1–7, `submitted-notice-proof.json`, both owned conversation JSONL copies, phase status/event snapshots, `feed-final.*` and `assertions.json`. All evidence paths are relative to the follow-up root’s `evidence` directory. This host has no automatic feed-watch delivery facility; no undeliverable follower was left running. Structured status/events were captured independently during active native checks.

Setup/evidence corrections did not change product expectations: process discovery initially expected an absolute make-app executable path but macOS reported a relative path, so the owned parent/PID was verified before stopping it; mission creation returned its ID inside `mission`, and the ledger was corrected immediately without a second mission. Native control once required refreshing its state before an action. The first conversation parser expected the older `event_msg.user_message` shape and was corrected to actual `response_item` user messages; a pre-cleanup snapshot with a premature phase label was retained under `pre-cleanup-evidence-parser-attempt`. Cleanup verification used the documented `live_session_count` after an initial wrong-key assertion. These were harness/setup errors, with the original attempts retained, not application failures or retries of failed product checks.

### Follow-up cleanup and scoped verdict

The owned mission was stopped and archived at `2026-10-18T01:57:48.310049Z`, completed with zero live sessions and absent from native Recents. Owned agent PIDs 90561/90573, make PID 89151, make-app PID 89645 and wrapper PID 90192 are verified absent. The wrapper was unregistered; its files, role/crew metadata, conversations and evidence are retained. Only the trust table for `/private/tmp/runner-826-followup-4llvwap8/fixture` was removed, leaving Codex config byte-for-byte equal to its pre-run capture. Input source and other global configuration were unchanged. Existing development sessions were allowed to resume and were never typed into, archived or otherwise operated; installed Runner was untouched. The development daemon remains the verified follow-up build to preserve unrelated saved sessions. No QA agent, wrapper or build remains active. `cleanup.json`, `config-cleanup.json`, `cleanup-process-verification.json`, archived CLI status/events and `archived-native.png` retain cleanup proof.

NO REMAINING MUST-FIX QA ISSUES for the selected macOS COMPOSER-1/feed follow-up. No selected local follow-up check remains unresolved. Historical MIS-MSG-02 remains Failed and tracked as P2 #940; native Windows remains Blocked, and this is not an unconditional full regression pass. QA changed only this record and scratch fixture/evidence, with no implementation edits, frozen-expectation edits, commits, pushes or publication.

### Follow-up publication preparation

Coder created fixups for the owning commits and autosquashed them into the three planned commits, rebasing onto `origin/main` at `1c8d7bc2708ab793bec0a39157da08cececad5b5`. The new mainline change only preserves a symlinked cwd in `PWD`. Before this publication note, the cumulative feature patch after rebase matched the pre-rebase patch byte-for-byte. The feature code diff relative to the new base retains reviewed/QA SHA256 `fe4d070d29c56fbf681cef0abf70cd2fbbfe70261df2d694a5a9bfe46f2cba3d`, so no additional live QA rerun was selected for the rebase.

Rebased validation passed: full workspace nextest 2417 passed and six skipped (exit 0), both workspace Clippy variants (exit 0), format check (exit 0), generated CLI guide check (exit 0) and cumulative whitespace check (exit 0). The extra test is from the newer mainline. Commands, logs and exit codes are retained in `/tmp/826-followup-rebased-validation.json` and `/tmp/826-followup-rebased-*.log`. The selected COMPOSER-1/feed follow-up is clean; historical MIS-MSG-02 and native Windows limitations remain unchanged.
