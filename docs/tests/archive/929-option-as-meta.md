# #929 / #1003 — macOS Option as Meta validation

Run date: 2026-10-23. Reviewer ran the mission's selected live checks after reviewing the complete frozen working-tree diff, touched implementations, their callers, and the changed tests. Jason stopped QA before the Claude composer check completed. This is a partial feature run, not a full regression pass.

## Candidate and environment

- Branch: `fix/929-option-as-meta`.
- Frozen HEAD: `121f1921f0fa296978e2a979be617eb59b229bf3`; SHA-256 of `git diff HEAD`: `9984559d3f471ca57933e4a3c23afe93bfa4d69fd5843c90f4e380f51313afff`. Rechecked before and during the live run.
- Jason subsequently authorized opening the PR before QA finished. Coder committed the same content as `4c8702b28951683bbea10c582314eb27af4fedcd`, [PR #1004](https://github.com/yicheng47/runner/pull/1004). This did not change the tested binaries.
- macOS 26.6.2 (25G83), Apple Silicon; development Runner 0.17.1; app zoom 120%, terminal font 16 px; zsh, Codex 0.163.0 (`gpt-6-luna`, low), Claude Code 2.1.309 (`haiku`, low).
- Evidence root: `/private/tmp/runner-929-qa.9224jtwy/evidence`. Scratch root and retained wrapper: `/private/tmp/runner-929-qa.9224jtwy`, `Runner 929 Dev.app`, bundle ID `com.wycstudios.runner.qa929`.
- Built with `make run` in this worktree. Wrapper executable SHA-256: `81961ab0043878463eedb474053956b6718056cfd2f646684f4c621cb5155396`; its agent CLI SHA-256: `43468f4f5c97237b15c0039d54c7a7d5fa8e41b045c2fb8862fc0ed5fa833527`. Copies matched the build outputs; see `build-hashes.txt` and `target/929-dev-build.log`.
- Verified development endpoint `com.wycstudios.runner-dev/runnerd.sock` in `status.json`. Installed Runner and its mission sessions were not operated. The saved development session was allowed to resume under the standing authorization and was otherwise untouched.

## Selected checks and automated evidence

Scope was selected before execution: layout-sensitive printable input and special-key routing; legacy Meta bytes; left/right Option discrimination; persistence; the new Settings row/menu; and directly affected word motion from `TERM-INPUT-05`. Unrelated IME composition, runtime lifecycle, authentication, and the full regression suite were outside scope.

Coder's completed gates were inspected before the frozen handoff: app and terminal nextest, 934 passed and 4 skipped (`target/929-app-terminal-tests.log`); daemon terminal tests, 38 passed (`target/929-daemon-terminal-tests.log`); workspace Clippy with the CI profile, updater feature and `-D warnings` (`target/929-clippy.log`); formatting and diff checks. Reviewer also built the full development app for this run. No competing Cargo job ran during the live build.

The diff review found no must-fix input-routing, persistence, encoding, or caller-integration defect. Tests cover the layout allowlist, settings default/round-trip, printable and special-key cases, side discrimination, Meta punctuation, kitty preservation, and the existing word-motion/copy-guard paths. The live visual mismatch below remains a design finding.

## Live results

| Case | Platform/runtime/variant | Result | Evidence and observation |
| --- | --- | --- | --- |
| 929-1 | macOS / zsh / German, Automatic | Passed | `german-auto-chars.png` shows `@[]|\`. `german-auto-bytes.ndjson` records `40`, `5b`, `5d`, `7c`, `5c` for Option+L/5/6/7 and Shift+Option+7. |
| TERM-INPUT-05 | macOS / zsh / German, Automatic | Passed | Option+Left/Right moved over words without inserting characters; `german-auto-word-left.png`, `german-auto-word-right.png`. Receiver bytes were `1b62`, `1b66`. |
| 929-2a | macOS / Claude Code / German, Automatic | Blocked, then stopped by Jason | `claude-before.png` shows the scratch-folder trust prompt. No trust decision was accepted, no composer was reached, and nothing was submitted. Approval was requested because the procedure pre-approves only Codex trust entries; Jason then stopped QA. No Claude composer pass is claimed. |
| 929-2b | macOS / Codex / German, Automatic | Passed | Native Option+L inserted `@` and opened Mentions; `codex-german-at.png`. Input was cleared without submission. |
| 929-3 | macOS / zsh / ABC, Automatic | Passed | Option+B/F moved backward/forward over words; `abc-auto-word-back.png`, `abc-auto-word-forward.png`. Layout changes took effect without relaunching. |
| 929-4a | macOS / zsh / ABC, Off | Passed | Option+B typed `∫`; `abc-off-char.png`, `abc-off-bytes.ndjson` (`e288ab`). The same receiver recorded Option+Enter `1b0d`, Backspace `1b7f`, Left `1b62`, Right `1b66`. |
| 929-4b | macOS / zsh / German, Both | Passed | Option+L emitted `1b6c`, and Option+5 emitted `1b35`; `german-both-bytes.ndjson`. |
| 929-4c | macOS / zsh / German, Left only | Passed | Side-specific CGEvents used keycodes 58/61 and device flags 0x20/0x40. Left Option+L emitted `1b6c`; right Option+L emitted `40`. See `german-left-synth-bytes.ndjson`, `left-option-post.txt`, `right-option-post.txt`, `german-left-right-option.png`. |
| 929-4d | macOS / Settings / persistence | Passed | Selected Left Option only, quit only the wrapper while keeping sessions, relaunched the same binary, and observed Left Option only. `ui-settings.after-quit.json`, `settings-after-relaunch.png`. |
| 929-5 | macOS / Settings / dark and light | Failed exact design comparison | Row placement after Cursor style and all copy match. Both themes render legibly. The open menu differs from `target/design-ref/option-menu-open.png`: the shared selector grows wider, puts the check on the right, and caps height so Off initially clips below the viewport instead of displaying all five choices. See `settings-dark-menu.png`, `settings-light-menu.png`; Off remains reachable by scrolling/keyboard. |

### Finding

`crates/runner-app/src/surfaces/settings_page.rs:451` constructs the new selector using the shared menu, whose height cap is in `crates/runner-app/src/ui/select.rs:970` and `:991`. At the existing 120% app zoom, opening it does not show all five choices as required by the supplied PNG. Align the new selector's presentation with the supplied design, or obtain explicit acceptance of the shared-selector rendering before calling the visual check Passed. This is a presentation finding; the mode behavior checks above passed. Avoid a broad redesign of unrelated selectors.

### Fixture attempts retained

The initial side-specific native-tool chord `alt_L+l` was unsupported. The first receiver attempt was also on ABC rather than German and produced the ABC Option character; `german-left-bytes.ndjson` and `left-receiver-attempt.png` retain that invalid attempt and are not used as evidence of a product failure or pass. The corrected test verified German and used the authorized CGEvent method. The first receiver's German Ctrl+] exit chord was unsuitable; its 60-second timeout ended it, and subsequent receivers used Ctrl+D. All receiver processes ended.

## Stop and cleanup

Jason stopped QA at the Claude approval gate. No further feature checks were run. The owned Codex and Claude chats were stopped and archived, and the owned shell was stopped. Shells cannot be archived through the chat archive command; its stopped terminal record is retained. IDs and results are in `ledger.json` and `session-cleanup.json`.

The wrapper was terminated and unregistered from LaunchServices. Process inventory in `processes.after.txt` shows no surviving wrapper or receiver. The development daemon remains available to its pre-existing resumed session; that session and the installed Runner were preserved. Scratch files, wrapper binaries, screenshots, and logs are retained.

The original enabled input-source list and selected ABC layout were restored and compared equal (`input-sources.before.plist`, `input-sources.after.plist`). The Globe-key action returned to Show Emoji & Symbols. App theme, Option-as-Meta default, and last quit choice were restored (`ui-settings.after-cleanup.json`). Only the scratch Codex trust stanza was removed; the parsed config equals its pre-run snapshot, with diffs retained in `codex-config.cleanup.diff` and `codex-config.run.diff`.

Adding German also added German to the disabled Dictation language list. Removal of the keyboard layout did not remove that incidental language. Cleanup reached the Dictation language dialog, but native control then lost its window (`cgWindowNotFound`) during the stop; restoration of that incidental Dictation-language entry is unverified. Dictation remained off. No global Claude trust was granted.

## Menu correction follow-up — code review and automated coverage

After QA stopped, coder handed over a correction against HEAD `4c8702b28951683bbea10c582314eb27af4fedcd`, with SHA-256 of `git diff HEAD` `ed85a30742197d18599ed596e3539bc242953cc11b6af3d0fe0ad95c7756cee7` (`target/929-menu-fix.diff`). Reviewer verified that identity and reviewed the complete two-file diff, the selector and popup implementations, their callers, and the regression tests. No app or live test was restarted.

Only Option as Meta opts into the wrapped menu: fixed 350 px width, right alignment, a reserved leading check slot, inset highlights, and a 400 px height cap, scaled with app zoom. Existing selectors retain their defaults. The correction addresses the original presentation finding in code; reviewer found no remaining must-fix issue.

Coder's automated evidence passed: the new `wrapped_option_menu_shows_every_mode_with_a_leading_check` GPUI geometry test (`target/929-menu-test.log`); app and terminal nextest, 935 passed and 4 skipped (`target/929-app-terminal-menu-fix-tests.log`); workspace Clippy with the CI profile, updater feature and `-D warnings` (`target/929-menu-fix-clippy.log`); formatting and diff checks. Reviewer inspected these logs and the test assertions. The geometry test uses the five mode labels/descriptions and checks every selected mode in dark/light at 100%/120% zoom: all rows inside the viewport, no scrolling, width/right alignment, leading check aligned with the label, and menu panel fill.

This automated result does not replace the original native result: `929-5` remains **Failed on the original candidate; corrected in code with automated geometry coverage; native retest not run because Jason stopped QA**. Claude `929-2a` remains Blocked. No complete live feature pass is claimed, and the cleanup limitation above is unchanged.


## 2026-10-23 — Retest on f8f4ab11

QA mission `01M5KEPR7B179R5CPD9MWMYHAR` ran the human's five scoped checks against clean `fix/929-option-as-meta` HEAD `f8f4ab11222ccb4e67c2d280b01d76dd6bc2a67c`. No implementation, frozen expectations, agent settings, commits, pushes or PR comments were changed. This section is the only intended repository edit. The `/Users/jason/repos/...` checkout in the brief resolves to `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-929-option-as-meta`; both names identify this worktree.

### Findings first

1. **Must fix: Option+C cannot type `ç` in character modes (929-7).** On German / Automatic and ABC / Off, focus an empty zsh prompt or the byte receiver, then press Option+C. Expected from the character-mode setting, especially Off's “Option always types the layout's characters,” is the layout character `ç` (`c3a7`). Actual: no character appears and no bytes arrive. In zsh, `echo QA929-C:` followed by Option+C still executes as `echo QA929-C:`. In each receiver, ordinary keys immediately before and after Option+C arrive, while Option+C contributes no bytes. Evidence: `german-automatic-bytes.ndjson`, `german-receiver-actions.json`, `german-option-c-zsh-{draft,result}.jpg`, `abc-off-bytes.ndjson`, `abc-off-receiver-actions.json`, `abc-off-option-c-zsh-{draft,result}.jpg`. Proposed cause, from source inspection: `swallows_option_copy` at `crates/runner-app/src/terminal_ime.rs:145` ignores the mode, and the chat caller at `crates/runner-app/src/surfaces/chat.rs:1421` invokes it before character routing. The mission and agent-update callers have the same guard. This is an observed limitation of this head; no live old-build comparison establishes when it began.
2. **Characterized behavior requiring a product decision: Option+Space inserts NBSP and can break a pipeline (929-6).** German / Automatic and ABC / Off each send `c2a0` (U+00A0). On German / Automatic, type `echo a `, hold Option while pressing 7 then Space, release Option, type `cat`, then Return. Actual history bytes are `65 63 68 6f 20 61 20 7c c2 a0 63 61 74 0a`; zsh reports `command not found: <NBSP>cat`. Evidence: `pipeline-history-targeted.txt`, `german-held-option-pipeline-targeted.jpg`, receiver logs above. This check had no frozen expected byte value, so its characterization is complete; the shell failure remains disclosed. Proposed cause: `terminal_key_route` at `crates/runner-app/src/terminal_ime.rs:126` treats NBSP as printable and uses the character path. The old-build `ESC space` prediction was not verified against an old artifact in this run.

### Scope, candidate and method

Checks were selected before execution: 929-5 covers the wrapped menu correction; 929-2a closes the Claude trust/composer gap; 929-6 characterizes whitespace bytes and the held-Option shell workflow; 929-7 checks the Option-copy/character boundary; and the German printable / German arrow / ABC B/F cases cover the directly affected zsh subset of `TERM-INPUT-05`. Left/right discrimination, persistence, Both-mode bytes, Codex, unrelated IME and runtime lifecycle, Windows and a complete regression were skipped as explicitly outside this retest. No extra agents or missions were launched.

Canonical scratch root: `/private/tmp/runner-929-retest.pp57l057`; all evidence names in this section are relative to its `evidence/` directory. The retained ledger is `ledger.json` at the scratch root. Baseline source identity is in `candidate.txt`, `status-before.txt` and the empty `dirty-before.diff` (empty diff SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`). The previous run's temporary evidence/build-log paths were unavailable in this checkout, so this run generated fresh evidence.

Environment: macOS 26.6.2 (25G83), arm64, Runner 0.17.1 development namespace, zsh, Claude Code 2.1.309 / Haiku 5.5 / low effort. The existing active development Claude account uses `/Users/jason/.claude-work`, with trust stored in `/Users/jason/.claude-work/.claude.json`; account identity is omitted from this record. Its config, settings and resolved paths were snapshotted before launch (`claude-config-paths.json`). Existing permissions/extensions were preserved; the native composer displayed auto mode. The direct-chat status reports the explicit model/effort and account folder (`claude-status-start.json`). No prompt was submitted.

Built with `env -u NO_COLOR -u RUNNER_CREW_ID -u RUNNER_MISSION_ID -u RUNNER_HANDLE -u RUNNER_EVENT_LOG make run`, using this worktree's own `target/`; see `make-run.log`. Stopped only that newly launched bare app, then copied the app and CLI into `Runner 929 Retest Dev.app`, bundle ID `com.wycstudios.runner.qa929retest`, under the scratch root. Native control was bound to that bundle ID. App SHA-256 `c8d8e4de451cb0c81e23b909a59589db2fba491985bec96851432882328737d4`; CLI SHA-256 `02ade1310bbca55b7017d21e03c9219c08bd58f0cd805ea221336ae6c08c8bfb`. App/CLI copies matched their sources, and the installed development CLI and daemon matched the CLI hash (`build-hashes.json`, `installed-sidecar-hashes.json`). `dev-status.json` confirms `com.wycstudios.runner-dev/runnerd.sock` and outside-mission invocation. Installed Runner and pre-existing development sessions were preserved; automatic development-session resume occurred under standing authorization.

Native input and screenshots used computer use. The held-Option reproduction used PID-bound CGEvents, keycodes 58 (left Option), 26 (7), 49 (Space), with Option/device flags `0x80020`. Input-source selection used synthesized Control+Space after the native shortcut could only toggle prior sources; the successful driver sent modifier `flagsChanged` events. Receivers used raw mode, a 120-second safety deadline, and exited normally on Ctrl+D. The German receiver's native `y` key produced `z` under German; its sentinel bytes `78` and `7a` are expected driver/layout behavior, not a defect.

### Results

| Case | Variant | Result | Evidence and observation |
| --- | --- | --- | --- |
| Automated | app + terminal | Passed | `cargo test --locked -p runner-app -p runner-terminal`: 935 passed, 4 ignored, no failures; `app-terminal-tests.log`. Includes `wrapped_option_menu_shows_every_mode_with_a_leading_check`. |
| Automated | workspace Clippy / updater | Passed | `cargo clippy --locked --workspace --all-targets --features updater -- -D warnings`, exit 0; `clippy.log`. Formatting and `git diff --check` also passed. |
| 929-5 | dark / 100% / Automatic + Off | Passed | `menu-dark-100-{automatic,off}.jpg`. All five choices fit, about 350 logical px wide, right edge aligned with trigger, leading selected check, wrapped copy and solid panel. |
| 929-5 | light / 100% / Automatic + Off | Passed | `menu-light-100-{automatic,off}.jpg`; same observations. |
| 929-5 | dark / 120% / Automatic + Off | Passed | `menu-dark-120-{automatic,off}.jpg`; about 420 logical px wide, all five choices fit without scrolling. |
| 929-5 | light / 120% / Automatic + Off | Passed | `menu-light-120-{automatic,off}.jpg`; same observations. |
| 929-2a | Claude / German / Automatic | Passed | `claude-german-automatic-at.jpg` shows `@` and file suggestions, including the owned `QA929-mention.txt` fixture. `claude-cleared-unsubmitted.jpg` shows cleared input. |
| 929-6 | German / Automatic / receiver | Passed | Characterized `c2a0` for Option+Space in `german-automatic-bytes.ndjson`; not a claim that NBSP is the desired policy. |
| 929-6 | ABC / Off / receiver | Passed | Characterized `c2a0` in `abc-off-bytes.ndjson`; source and setting evidence in `abc-input-source.txt`, `abc-off-setting.jpg`. |
| 929-6 | German / Automatic / held-Option pipeline | Passed | Requested characterization completed: shell workflow reports `command not found`; exact NBSP history and screenshot cited in finding 2. This is not a successful-pipeline verdict. |
| 929-7 | German / Automatic / receiver + zsh | Failed | No `ç` or bytes; finding 1. |
| 929-7 | ABC / Off / receiver + zsh | Failed | No `ç` or bytes; finding 1. |
| 929-1 subset | German / Automatic / Option+L/5/7 | Passed | `german-automatic-zsh-characters.jpg` shows `@ [ |`; receiver records `40`, `5b`, `7c`. |
| TERM-INPUT-05 subset | German / Automatic / Option+Left/Right | Passed | `german-automatic-word-{left,right}.jpg` show cursor at the start/end of `delta` in the intact unsent draft. Receiver records `1b62`, `1b66`. |
| Meta regression | ABC / Automatic / Option+B/F | Passed | `abc-automatic-word-{back,forward}.jpg` show corresponding word motion with intact unsent text. |
| Cleanup | keyboard input sources + Dictation | Passed | Initially Blocked by native controls, then completed after Jason approved APIs. `tis-cleanup.txt`, `dictation-api-cleanup.txt`, `keyboard-after-cleanup.jpg` and `keyboard-dictation-cleanup-summary.json`: enabled sources exactly match U.S./Pinyin baseline, U.S. selected, Dictation off/English only. Preference snapshots/diffs and residual metadata are disclosed below. |
| Excluded variants | left/right, persistence, Both bytes, Codex, other runtimes/platforms | Skipped | Explicitly outside this retest; no earlier passes are attributed to this head. |

The original candidate's 929-5 failure remains in the earlier history; the corrected head passes the native menu retest. No additional visual defect was observed against the mission's criteria. Design PNGs were unavailable, so no exact-design-image comparison is claimed. Option+C remains a must-fix issue; this run does not support “NO REMAINING MUST-FIX QA ISSUES.”

### Attempts and cleanup

The first held-Option helper posted globally while the browser was frontmost, so the candidate received neither Option character and executed `echo a cat`. That driver attempt is retained in `pipeline-history-invalid-global-event.txt` and `german-held-option-pipeline.jpg`; it is not a candidate pass or failure. The corrected PID-bound attempt produced the recorded `|<NBSP>cat` shell failure. Several source-switch attempts selected U.S./Pinyin/German before the corrected modifier-event driver reached ABC. Native System Settings/source-menu access failures are setup limits, not Runner failures. No unsupported chord result was counted as a product result.

Owned Claude chat `01M5KF6MNTF5NHHKJ93R3XXX5J` was stopped and archived; `claude-chat-final.json` records its archive timestamp. Owned shell `01M5KFCH3DHNZEVA12DNSZJDTY` and both receivers are stopped; `shell-final.json` records stopped state. Shell archive is unsupported, so its stopped record/history is retained. No test missions or roles were created. The wrapper process was terminated and its LaunchServices registration removed (`unregister-wrapper.txt`); its files and evidence remain. The development daemon remains available to pre-existing sessions. No installed Runner process, session or mission was operated.

Only this scratch folder's Claude project/trust entry was removed. `claude-trust-cleanup.diff` proves that targeted removal; `claude-config-run.diff`, `claude-config-final.diff` and `claude-cleanup-summary.json` preserve remaining changes. The work-account settings file is byte-identical to its snapshot. Other startup/cache/bookkeeping fields changed during the run: `numStartups`, `promptQueueUseCount`, `additionalModelOptionsAnsweredAt`, GrowthBook cache keys/timestamp and a client-data cache slot. Those were preserved, not reverted; concurrent/runtime attribution is not established. No Codex test chat or trust entry was created. App theme (dark), zoom (130%), effective Option mode (Automatic) and last quit choice (stop) were restored (`ui-settings-after-cleanup.json`).

Keyboard baseline before the run was U.S. plus Pinyin – Simplified, with U.S. selected (`input-sources-before.plist`, `keyboard-before.png`). Dictation was off with English (United States) only; the prior run's German Dictation language was absent at this run's baseline. This run added ABC and German keyboard layouts; German also appeared in Dictation (`keyboard-during.ax.txt`). The native-control cleanup blocker was reported before the following authorized recovery; it is retained as attempt history.

On 2026-10-23 Jason explicitly approved TIS layout APIs and preference APIs for cleanup (Runner messages `01M5KG20KWJZ1SXY8CJ1JJR3M6`, `01M5KG20N12AWN7K0S7M3YHQBW`), with before/after domain snapshots, changes limited to the run's added layouts/German Dictation entry, and no retries over unexplained differences. `TISSelectInputSource` selected U.S.; `TISDisableInputSource` disabled German and ABC, all returning status 0. The enabled-source array is exactly equal to the pre-run baseline and current keyboard is U.S. `CFPreferencesSetValue` and synchronization removed only `VisibleNetworkSRLocaleIdentifiers.de_DE` in `com.apple.speech.recognition.AppleSpeechRecognition.prefs`; the before/after Dictation-domain comparison proves every other value was preserved. No plist was hand-edited.

Both touched domains were snapshotted before and after cleanup: `cleanup-hitoolbox-{before,after,final}.plist` and `cleanup-dictation-{before,after,final}.plist`; `cleanup-hitoolbox.diff` and `cleanup-dictation.diff` show the changes. Fresh native Settings verification shows U.S./Pinyin and Dictation off/English only (`keyboard-after-cleanup.jpg`, `keyboard-after-cleanup.ax.txt`); API evidence is in `tis-cleanup.txt`, `dictation-api-cleanup.txt` and `input-sources-final-tis.txt`. Reopening Settings did not change either domain. The owned Settings app was then quit. No keyboard/Dictation cleanup check remains blocked.

Full preference domains are not byte-identical to the pre-run baseline. The TIS-maintained source history retains disabled ABC/German and puts U.S. first; its last-update timestamp records this run. The run's input-source selection also dropped the non-keyboard Character Palette from `AppleSelectedInputSources`; TIS still reports it enabled, but selected=false. No palette preference was edited. Dictation now explicitly retains `en_US` in `VisibleNetworkSRLocaleIdentifiers`, whereas the baseline relied on the default; the native UI consequently also exposes the default-on Auto-punctuation control. Only `de_DE` was removed from that dictionary, as authorized. These known run/selection/default-materialization differences are preserved in `baseline-to-final-{hitoolbox,dictation}.diff` and `keyboard-dictation-cleanup-summary.json`; no unexplained difference or further mutation/retry was found.


### Fix retest — 2026-10-23

**NO REMAINING MUST-FIX QA ISSUES within this requested fix-retest scope.** The earlier Option+C failure and NBSP pipeline failure remain recorded for the unfixed candidate; both now pass on the frozen uncommitted correction. This is a scoped macOS shell/input retest, not a complete feature or regression rerun.

Candidate: HEAD `f8f4ab11222ccb4e67c2d280b01d76dd6bc2a67c` plus the uncommitted implementation/guide diff with SHA-256 `d424ec3aab953c94077a9c2d46a40ff7d66a9403b1c1766566a7b5e95bc9d5e0`, computed from `git diff HEAD -- crates docs/guide`. QA verified this hash before building and again before reporting; this test-record edit is excluded. The frozen diff and both hash checks are retained in `candidate-dirty.diff`, `candidate-diff-sha256.txt` and `candidate-diff-sha256-final.txt`.

Scope was selected before testing from the human's Runner request: rerun 929-7 in German Automatic and ABC Off to verify character-mode copy-guard routing; rerun 929-6 bytes and the German held-Option pipeline to verify NBSP normalization; check ABC Automatic's preserved copy guard and Meta-space bytes as direct regressions; and smoke-test German `@` on the rebuilt app. Menus, Claude/Codex, left/right selection, persistence, word motion, other platforms and runtime lifecycle were excluded from this follow-up.

Scratch root: `/private/tmp/runner-929-fix-retest.ksibb0s9`; evidence names below are relative to its `evidence/`. Environment remained macOS 26.6.2 / arm64, Runner 0.17.1 development namespace, zsh, dark theme / 130% app zoom / 16 px terminal font. No agent chat or mission was created. The run used the same authorization, with TIS enable/select/disable APIs for this run's German/ABC layouts and restoration to U.S./Pinyin.

Rebuilt with `make run` in this worktree's own `target/`, omitting `NO_COLOR` and implementation-mission identity variables. After stopping only that launch's bare app, QA drove `Runner 929 Fix Retest Dev.app`, bundle ID `com.wycstudios.runner.qa929fixretest`, through native computer use. Wrapper app SHA-256 `2a50dca80bb70653f42dab661d6f4798e7f83dc8e13aeb2f2855d22bf84da7b4`; CLI SHA-256 `02ade1310bbca55b7017d21e03c9219c08bd58f0cd805ea221336ae6c08c8bfb`. Source/copy hashes match; installed development CLI/daemon hashes also match. See `make-run.log`, `build-hashes.json`, `installed-sidecar-hashes.json`, `status.json` (development socket, outside-mission mode). Pre-existing development sessions resumed under standing authorization and were otherwise untouched.

QA independently ran `cargo test --locked -p runner-app terminal_ime`: six tests passed, including the new normalization/copy-guard unit tests and UTF-8 forwarding through the session manager (`terminal-ime-tests.log`). The human handoff separately reported app/terminal nextest 936 passed / 4 skipped, workspace CI-profile updater Clippy with `-D warnings`, formatting and diff checks; those broader results are handoff evidence, not a newly executed QA suite. QA's final `git diff --check` passed.

| Check | Variant | Result | Evidence and actual result |
| --- | --- | --- | --- |
| 929-7 | German / Automatic / receiver + zsh | Passed | `german-automatic-bytes.ndjson` records `c3a7`; `german-option-c-zsh.jpg` shows `QA929-C:ç` after the requested echo sequence. |
| 929-7 | ABC / Off / receiver + zsh | Passed | `abc-off-bytes.ndjson` records `c3a7`; `abc-off-option-c-zsh.jpg` shows `QA929-C:ç`. |
| 929-6 | German / Automatic / Option+Space | Passed | Same German receiver records `20`, replacing the previous candidate's `c2a0`. Action order is in `german-receiver-actions.json`. |
| 929-6 | ABC / Off / Option+Space | Passed | Same ABC Off receiver records `20`; `abc-off-option-space-zsh.jpg` also shows a regular argument separator and output `QA929-SP ok`. |
| 929-6 | German / Automatic / held Option+7 then Space | Passed | `german-held-option-pipeline.jpg` shows `echo a | cat` printing `a` with no error. `pipeline-history.txt` is exactly `echo a | cat\n`, with byte `20` after `7c`. PID-bound CGEvents held left Option across keycodes 26 and 49 with flags `0x80020`; `held-option-driver.txt`. |
| Meta guard | ABC / Automatic / Option+C | Passed | `abc-automatic-meta-bytes.ndjson` and `abc-meta-receiver-actions.json`: ordinary `x`, Option+C, ordinary `k` arrive as `78`, `6b`, with no bytes for Option+C. `abc-meta-receiver-results.jpg`. |
| Meta space | ABC / Automatic / Option+Space | Passed | Same Meta receiver records `1b20`; character-mode normalization leaves the Meta path intact. |
| Rebuilt-app smoke | German / Automatic / Option+L | Passed | `german-at-zsh.jpg` shows unsent `@`, then cleared. Receiver independently records `40`. |
| Focused automated coverage | terminal IME / routing | Passed | Six passed, no failures; `terminal-ime-tests.log`. The normalization unit test also preserves NBSP within a longer string. |
| Cleanup | owned shell, wrapper, input sources, Dictation, app settings | Passed | Evidence and baseline comparison below. |
| Other cases | menus, agent composers, persistence, left/right, word motion, other platforms | Skipped | Outside the human's explicitly narrowed fix-retest scope. |

No required check is Failed or Blocked on this correction. The aggregate receiver files were independently checked as exact hex: German `c3a72040`, ABC Off `c3a720`, ABC Automatic Meta `786b1b20` (the `78`/`6b` are the guard's surrounding sentinels). Receivers exited on Ctrl+D; no invalid attempt or recovery retry was needed in this follow-up. No implementation or frozen expectation was changed by QA.

Cleanup: owned shell `01M5KH1SC52FDDA6CF7WQPVZXE` is stopped (`shell-final.json`), with its record/history retained because shell archiving is unsupported. All three receivers exited normally. The wrapper is stopped and unregistered (`wrapper-unregister.txt`, `processes-final.txt`); its bundle, scripts and screenshots are retained outside Git. The development daemon remains available to pre-existing sessions, and installed Runner was not operated. No Claude/Codex chat or scratch trust entry was needed.

TIS selected U.S. and disabled only this run's German/ABC, all with status 0 (`restore-us.txt`, `disable-run-layouts.txt`). Enabled sources, selected-source array and current U.S. keyboard exactly match this run's recorded U.S./Pinyin baseline. The Dictation domain remained exactly equal to the run baseline, off/English only, with no German entry and no Dictation preference write needed. Fresh native Settings verified U.S./Pinyin and English-only Dictation (`keyboard-dictation-final.jpg`, `keyboard-dictation-final.ax.txt`), then that owned Settings app was quit. Both domains have before/after/final snapshots: `cleanup-{hitoolbox,dictation}-{before,after,final}.plist`, plus `{hitoolbox,dictation}-baseline.plist` and `*-{cleanup,baseline}-to-final.diff`. Only HIToolbox's source-history and last-update metadata differ from the run baseline; these expected source-switch records were preserved. The entire parsed app-settings document equals its baseline (`ui-settings-{baseline,final}.json`). The ledger is `ledger.json` at the scratch root. No commit, push, PR action, merge, mission archive or worktree/branch removal was performed.

## Regression coverage

Archived with 0.17.2 on 2026-10-24. Lasting input checks are `TERM-OPTION-01` through `TERM-OPTION-03` in the [terminal regression cases](../regression/terminal.md), with the menu and saved choice in `SET-OPTION-01` in the [settings cases](../regression/settings.md). Word motion continues under `TERM-INPUT-05`. Routing, encoding and menu-geometry assertions remain in the owning automated tests. Candidate-specific failures, scoped retests, cleanup history and excluded variants remain evidence above, not new native passes.
