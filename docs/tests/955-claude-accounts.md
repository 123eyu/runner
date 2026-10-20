# #955 Claude Code accounts validation

Branch: `feat/955-claude-accounts`. Worktree: `/Users/jason/repos/yicheng47/runner/.worktrees/feat-955-claude-accounts`. Platform: Apple Silicon macOS. The feature ships in #970 as one commit per phase. Each phase is reviewed as a working-tree diff before independent QA.

## Phase 1 — agent settings pages

Changed behavior: installed and missing agents appear in compact grouped rows, each opening its agent page. The breadcrumb and Agents sidebar selection return to the list. Executable overrides, defaults, enable/default actions and agent updates retain their existing handlers.

Selected checks: `runner-app` tests cover navigation for every runtime in both detection groups, row action isolation, compact layout at two zoom levels, page defaults and captions, invalid overrides, default preference persistence, update placement and live-session captions, and locale changes without replacing input drafts. `SET-AGENT-01` and `SET-AGENT-02` plus the brief’s row, breadcrumb, Update and executable checks are assigned to QA in the development app. Workspace Clippy with and without updater and formatting are required gates. Daemon tests guard the existing core used by the settings controls; no core or daemon implementation changes are part of this phase.

Local gates (2026-10-20), all exit 0:

| Command | Exit | Log |
| --- | --- | --- |
| `cargo test --locked -p runner-app --profile ci --no-fail-fast` | 0 | `app-tests.log` |
| `cargo test --locked -p runner-daemon --profile ci --no-fail-fast` | 0 | `daemon-tests.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `clippy.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings` | 0 | `clippy-updater.log` |
| `cargo fmt --all --check` | 0 | `fmt.log` |
| `git diff --check` | 0 | `diff-check.log` |

`runner-core` has no changes in phase 1, so its separate test gate is outside this phase’s scope. Logs live under the ignored `target/validation/955-phase1/`. Initial implementation runs returned exit 101: first the action wrapper needed a GPUI element ID; next dynamic debug selectors needed static test selectors; next scroll reset needed a shell method. The first complete UI run had four failures: zoom geometry needed a one-pixel rounding tolerance, invalid overrides routed to the installed page (affecting two checks), and the locale test needed to open the page containing its field. These were corrected without dropping the affected checks; the final run determines the gate result. The first Clippy runs (with and without updater) also returned 101 for a test-only `px` import and a redundant `16. / 16.` constant; both were corrected and their logs retained.

Review: reviewer posted **NO REMAINING MUST-FIX ISSUES** through Runner on 2026-10-20 at HEAD `a24a1a69`. It inspected the whole diff, committed spec/brief and `YjEPm`/`QqpJd`, and traced the existing handlers. Its independent `cargo test --locked -p runner-app --profile ci surfaces::settings::agents::tests --no-fail-fast` passed (29 tests, exit 0); `git diff --check` passed (exit 0). The reviewer requested live root-shell coverage for scroll reset, keyboard activation and leaving a focused executable editor in addition to the brief’s checks.

Initial native QA (2026-10-20) used development app/CLI 0.16.7 with the development daemon, with an owned macOS wrapper and disposable Copilot executable. Candidate was HEAD `a24a1a69c7a9828bbf01f9757ce46480d07572b1` plus reviewed diff SHA-256 `34054259274556ebda5fa8dc2f60756a6feff5d73b69a100b637083c395ec02c`, verified unchanged before and after. Full report and matrix: `/private/tmp/runner-955-phase1.cee_c7tk/{qa-report.md,matrix.json}`. Artifact hashes and environment are recorded beside them. This is selected phase 1 QA, not a full regression pass.

| Live check | Initial result | Evidence |
| --- | --- | --- |
| `SET-AGENT-01` | Passed | All 13 Settings panes; Refresh, detection, versions, defaults and paths; `settings-*-scrolled.png`, `installed-pages-contact.png` |
| `SET-AGENT-02` selected relocated-control variants | Passed | Copilot enable/default, valid/invalid Browse override, bounded fixture launch, preference persistence across restart, Reset and restoration; `override-prefs-contact.png`, `scroll-restart-contact.png`, `runtime-status-*.json`, `fixtures/launched` |
| Every installed and missing row opens its page; breadcrumb returns | Passed | 8 installed agents and missing TRAE; `installed-pages-contact.png`, `trae-detail.png` |
| Phase 1 design hierarchy | Passed | Grouped compact rows and breadcrumb/title/executable/default sections at 130% zoom |
| Update on agent page | Passed | Disposable Copilot override: waiting-input footer, native terminal OK, success/version refresh and Done; `updater-contact.png`, `fixtures/invocations.log` |
| Scroll resets via breadcrumb and Agents sidebar | Passed | `scroll-restart-contact.png` |
| Leave focused executable editor via breadcrumb and sidebar | Passed | `focused-editor-*.png` |
| Keyboard traversal and activation | Failed — fix awaiting recheck | Copilot Executable → Tab → typing `QA955_KEYBOARD_FOCUS` left the marker in the field; Space also stayed in the editor. No baseline native attribution claimed. `keyboard-tab-retains-editor.png`, `keyboard-browse-contact.png`, `tabs-contact.png` |
| Missing-page description and install link | Blocked | Only missing runtime was TRAE, whose unchanged metadata has empty description/install URL (`runtime_metadata.rs:629–630`). Missing-page navigation and Browse passed; populated metadata is covered by automated tests. |
| Native Windows | Blocked | No native Windows host; pending Jason’s checks |
| Accounts, real-account chats, unrelated lifecycle and full regression | Skipped | Outside phase 1 scope; no installed agent was actually updated |

Evidence filenames above are under the native QA root. Cleanup: owned fixture chat `01M5CHBYNC7K6GV2YKPB697JXN` stopped/archived, fixture role `qa955phase1fixture` retained under its private `COPILOT_HOME`, wrapper stopped/unregistered, original executable override cleared, and UI settings restored to JSON equality (the initial run did not prove byte equality). Existing development sessions and installed Runner were untouched; the development daemon stayed alive for existing sessions.

### Keyboard correction

The new root-shell test `agent_page_keyboard_tabs_from_executable_to_browse_and_back` reproduced the initial field-focus failure (exit 101, `keyboard-before.log`). The Agents settings root now routes Tab and Shift-Tab through GPUI’s focus traversal, as the role and crew pages already do, while leaving composition alone. The regression exercises the real settings shell, focus moving from the executable field to Browse, Space opening the path picker, Shift-Tab returning to the field, and keyboard list-row activation. No daemon/core change is involved. The extended root-shell test and all relevant corrected gates passed (exit 0):

| Check | Exit | Log under `target/validation/955-phase1/` |
| --- | --- | --- |
| Root-shell keyboard regression | 0 | `keyboard-after.log` |
| Full `runner-app` test command above (780 passed, 3 existing ignored) | 0 | `app-tests-after-keyboard.log` |
| Workspace all-targets Clippy | 0 | `clippy-after-keyboard.log` |
| Workspace all-targets Clippy with updater | 0 | `clippy-updater-after-keyboard.log` |
| Formatting | 0 | `fmt-after-keyboard.log` |
| Whitespace | 0 | `diff-check-after-keyboard.log` |

The previous daemon gate remains valid because this correction changes only app focus routing and its regression test. Follow-up working-tree review is clean: reviewer posted **NO REMAINING MUST-FIX ISSUES** on 2026-10-20, independently ran the exact root-shell keyboard regression (1 passed, exit 0) and `git diff --check` (exit 0), and inspected corrected full-app and Clippy/updater logs. The affected native recheck is requested next. Its scope adds Tab/Shift-Tab and Enter/Space on rows, toggles and Browse, and a brief check that active text composition prevents Tab traversal because the corrected routing touches that behavior. Populated missing-page description/install-link coverage will use the existing isolated invalid Copilot override; no TRAE metadata change is required. The initial native QA failures and blocked observations above remain part of the record.

### First native keyboard recheck

QA rebuilt and tested frozen candidate `a24a1a69` plus aggregate SHA-256 `bdd04ca4d1cb24194ed920917cc4acd6bbae26da96a92a9eb6e0d25362d2bb22`; the candidate was unchanged before and after. App SHA-256 was `6263d655a9685d31d864cdad2ede3045f270cef9b28689d8fa1db16b26969b6c`; worktree, wrapper and installed development CLI hashes matched. Full report, matrix and native evidence: `/private/tmp/runner-955-phase1-recheck.y47viojq/`.

| Affected native check | Recheck result | Evidence |
| --- | --- | --- |
| Field Tab/Shift-Tab, Browse Enter/Space | Passed | Native picker opened and was canceled for both keys; reverse focus confirmed in editor; `keyboard-contact.png`, `browse-space-picker.png`, `browse-enter-picker.png`, `browse-shift-tab-editor-corrected.png` |
| Installed and missing row Enter/Space | Passed | OpenCode and TRAE; `installed-row-enter.png`, `installed-row-space.png`, `missing-row-space.png`, `list-row-enter-corrected.png` |
| Enable-toggle Enter/Space | Failed — correction below | OpenCode toggle visibly focused; Enter/Space left the saved state unchanged from both on and off, while mouse activation changed it. `focus-ring-detail.png`, `toggle-failure-comparison.png`, `toggle-*-readback*.json` |
| Breadcrumb/sidebar and focused-editor exit | Passed | `restoration-navigation.png`, `focused-editor-*.png` |
| Populated missing-page description/install link | Passed | Owned valid Copilot override renamed to make its persisted path invalid, then Refresh; description and Install instructions rendered. Reset restored null override. `missing-page-detail.png`, `runtime-restored.json` |
| Active text composition | Blocked | Existing Pinyin could be selected, but available key injection committed ASCII without marked text/candidates. The native composition guard could not be graded; no input source was added or removed. |
| Native Windows | Blocked | Host unavailable as declared |
| Prior passed phase 1 cases | Passed, retained | Settings/layout/update/persistence checks unaffected by the focused correction |
| Accounts, unrelated lifecycle and full regression | Skipped | Outside affected scope |

The recheck created no chats, roles or missions. Its disposable executable was retained with evidence, override cleared, OpenCode restored, UI settings restored byte-for-byte to the recheck baseline, and the owned wrapper stopped/unregistered. U.S. was observed restored in the QA app before closure; afterward the current input-source plist showed Pinyin in another active context, so no global/per-app restoration claim is made. Enabled input sources were unchanged. The initial run’s restoration above was JSON-equal; exact bytes were established only for this recheck.

### Toggle correction

The new deterministic shared-control regression sends both key down and key up for Enter and Space, from both on and off, and checks exactly one state change, no containing-row activation, a second toggle back, and mouse activation. It reproduced the failure before the fix (exit 101, `toggle-before.log`): Enter from off ended off. GPUI synthesizes a click on key release; `Toggle` also invoked its handler on key down, causing two flips. `Toggle` now uses its single click handler for activation and consumes key down only to keep a containing row from opening. The existing root-shell regression now includes key release for its toggle interaction. This shared-control change warrants the full app suite and workspace Clippy; daemon/core remain unchanged.

The targeted toggle regression passes (exit 0, `toggle-after.log`). Corrected gates all pass (exit 0):

| Check | Exit | Log under `target/validation/955-phase1/` |
| --- | --- | --- |
| Full `runner-app` test command (781 passed, 3 existing ignored) | 0 | `app-tests-after-toggle.log` |
| Workspace all-targets Clippy | 0 | `clippy-after-toggle.log` |
| Workspace all-targets Clippy with updater | 0 | `clippy-updater-after-toggle.log` |
| Formatting | 0 | `fmt-after-toggle.log` |
| Whitespace | 0 | `diff-check-after-toggle.log` |

The existing daemon gate remains 0 with no daemon/core changes. The toggle correction passed working-tree review: reviewer posted **NO REMAINING MUST-FIX ISSUES** on 2026-10-20 after checking the shared control, Agents/MCP/General/Notifications callers, and pinned GPUI key-release click behavior. Its exact toggle regression, exact root-shell regression and `git diff --check` each exited 0. The affected native toggle recheck is requested next. Active composition and native Windows remain explicitly Blocked unless separately verified.

### Final affected macOS QA result

The native toggle recheck passed every completed affected case on 2026-10-20, with no remaining observed must-fix behavior issue in the completed phase 1 macOS matrix. QA tested HEAD `a24a1a69c7a9828bbf01f9757ce46480d07572b1` plus frozen aggregate `e248cd35f653bd6b43688e36711f23aad438daca1c0b73ab55f79b9726738b22`, unchanged before and after; app SHA-256 `083a788592e2220862ac6efe444604ff4a1dd73bf63ca0528e49180f2dbea0f1`. Worktree/wrapper/installed development CLI hashes matched, and the development app/CLI 0.16.7 and endpoint were verified. Evidence: `/private/tmp/runner-955-phase1-toggle.tokm_eu6/{qa-report.md,matrix.json,state-matrix.json}`.

| Final affected case | Result | Evidence |
| --- | --- | --- |
| List Enter, enabled → disabled and disabled → enabled | Passed | `list-enter-on-to-off`, `list-enter-off-to-on` PNGs and independent settings JSON |
| List Space, enabled → disabled and disabled → enabled | Passed | `list-space-on-to-off`, `list-space-off-to-on` |
| Page Enter, enabled → disabled and disabled → enabled | Passed | `page-enter-on-to-off`, `page-enter-off-to-on` |
| Page Space, enabled → disabled and disabled → enabled | Passed | `page-space-on-to-off`, `page-space-off-to-on` |
| Mouse on/off and restoration on list and page | Passed | `list-mouse-*`, `page-mouse-*` |
| Saved preferences and route isolation | Passed | 14 passing independent saved-state assertions; `toggle-variants-contact.png`, `full-route-contact.png`; default runtime stayed Codex |
| Earlier passed selected phase 1 cases | Passed, retained | Settings/layout/update/persistence, field/Browse, installed/missing rows, navigation and populated missing-page metadata |
| Active text composition | Blocked | Available native key injection did not establish a gradeable marked-composition state |
| Native Windows | Blocked | Host unavailable as declared |
| Accounts, real-agent chats and full regression | Skipped | Outside affected recheck |

QA restored UI settings byte-for-byte, stopped/unregistered its wrapper and verified process absence. It created no chats, roles or missions and made no source/test-record, account, input-configuration, SQLite or production changes. Existing sessions were not individually operated. The working tree was unfrozen after the recheck. Both failed native attempts remain recorded above. Phase 1 review is clean, all required local gates pass, and completed native cases pass with the two named Blocked checks retained. The spec and brief are folded into the phase 1 commit before pushing #970; this is not a full regression or native Windows pass.

## Later phases

Phase 1 is committed and pushed as `cfaa9f0e84523639750ccd492b0d51912ddb70f2`, `feat(ui): give each agent its own settings page`, on `origin/main` `8e523b5e`. Draft #970 remains open. CI run `39419730277` passed on macOS and Windows at the exact phase 1 head SHA. `gh pr checks 970 --watch` exited 0; its log and exit are retained at `target/validation/955-phase1/{ci-watch.log,ci-watch-exit.txt}`. macOS completed formatting, Clippy with/without updater, tests and generated CLI-reference checks. Windows completed installer validation, Clippy, tests, resource/manifest/icon verification and generated CLI-reference checks. This is automated Windows validation; the native Windows QA gap remains.

## Phase 2 — opening probes

The fork-placement probe exited 0 on 2026-10-20 with installed Claude Code 2.1.304 and `--model haiku`. It used `/private/tmp/runner-955-fork-fr9tkcm3`, the existing work sign-in, two tiny prompts and no tools. It created its own source conversation, copied only that owned transcript into an isolated `/tmp` origin configuration, then forked using the path form and an assigned UUID. The fork recalled the marker, wrote under the active work account's `projects/` directory, and preserved both source files. Probe script and sanitized output are under ignored `target/probes/955-phase2/{fork_probe.py,fork-result.json}`. No credentials were copied or logged.

The read-only per-account usage probe is retained at `target/probes/955-phase2/usage_probe.py`. It keeps credentials in memory and emits only service names, exit/status codes and numeric usage windows. Jason authorized both reads through Runner on 2026-10-20. The probe exited 0: both Keychain commands exited 0, both HTTP requests returned 200, and each returned valid five-hour and weekly windows. The native weekly value was 82%, work 24%; no credential or identity was retained. Jason also clarified that Make main changes only the shared configuration owner, preserving fixed sign-in targets, and that stored chat origins remain usable after removing an account. The native row stays represented. Phase 2 implementation and working-tree review are in progress; native QA has not started.

Phase 3 sharing probes, implementation, review and QA have not started. No mission code has written into `~/.claude` or `~/.claude.json`.

Native Windows is unavailable to this crew. A folder with spaces, the credentials-file route and the configuration-junction route require Jason’s native Windows checks after implementation; they are not claimed as validated by macOS or CI.

## Phase 2 — implementation and automated verification

Phase 2 implements account persistence and operations through the client protocol, per-folder authentication, active-account launches for direct and mission sessions, recorded account/origin metadata, cross-account resume and fork, per-account usage, and the Accounts, usage and chat UI. Configuration sharing is reserved for phase 3. No database migration was added. Authentication identity and sign-in URLs remain in memory; launch metadata stores no email or credential. Headless test cores disable authentication so UI tests cannot invoke an installed Claude sign-in command.

The changed behaviors and selected checks are:

| Behavior | Checks and reason |
| --- | --- |
| Account state, operations and authentication | Daemon tests cover native bootstrap without migration, JSON without authentication data, folder derivation, add/rename/remove/make-main/use rules, null subscription plans and a disposable fake login process with scoped cancellation |
| Launch, resume, fork and recovery inputs | Existing one-account spawn and usage goldens stay unchanged; new session-manager tests switch targets, retain origin and key on resume-on-launch, restore the transcript’s permission mode, place fork history under the active account, honor explicit pins and preserve history after forgetting Work |
| Transcript lookup and credentials | Tests cover canonical working-directory fallback, a folder with spaces, exact per-folder SHA-256 Keychain services, explicit native-folder pins, credential-file isolation and denial without a credential read |
| Usage and cache transitions | Tests cover selecting only the chosen account’s cached value, separate denial state and renaming the sole account without clearing its existing meter |
| Account UI and protocol | App tests cover removal guards, rows and add form, the single-pane header chip, Account beside Runtime and the config row; core tests round-trip the new protocol and preserve the distinction between native identity and an explicit folder pin |

Sign-in subprocesses inherit Runner’s composed login-shell PATH and use its existing process-tree isolation. Cancellation, account removal and daemon shutdown stop only the owned sign-in job and its helpers; Windows uses the existing job-object route and hidden-console flags. Account mutations serialize the persisted selection and cache update across windows, and retain the selected native ID when a prior multi-account usage fetch is still finishing. Account changes queue a follow-up refresh when another fetch is in flight.

The global Claude launch gate remains unchanged. It preserves the existing OAuth refresh-race protection and one-account timing; separate accounts may briefly wait behind each other during concurrent fresh launches. Resumes retain the existing gate policy. Cross-account path resumes strip competing role/mission permission flags before restoring the last valid `permissionMode` from the original transcript; the owned fork probe’s transcript was also checked to contain that field.

Jason’s Runner clarifications are reflected in the spec: Make main changes the configuration owner while sign-in targets remain fixed; the original native row stays represented; existing recorded chat origins remain usable after removing an account, including when only the native row remains. A setup that has never added a second account keeps the original launch/resume/fork/usage path. Explicit environment pins remain pins, including an explicit `~/.claude`, and pinned paused chats do not promise a switch to the global active account.

An added single-account rename regression initially failed with a pool timeout: account mutation retained its database connection while requesting a new snapshot. The connection is now released before that read, and the regression passes. The failing full daemon run remains in `target/validation/955-phase2/daemon-test-final.log` (exit 101); `rename-recheck.log` records the targeted passing recheck (exit 0). A new panel-layout test’s first run used the wrong case for an existing debug selector; the corrected test passes, with the failed `panel-layout.log` and passing `account-ui-recheck.log` (exit 0) retained. The final candidate app, daemon and core runs are the `*-candidate.log` files. Clippy then identified one redundant borrow in the extracted status-refresh helper; its cleanup changes no behavior.

Automated results before review: app tests exited 0 (785 passed across targets, three existing ignored live/manual tests); daemon tests exited 0 (1,284 passed); core tests exited 0 (64 passed). Final lint, updater lint, formatting and diff results are recorded in `target/validation/955-phase2/gate-exits-candidate.json`, with each command, its actual exit code and its log. App, daemon and core commands use `--locked --profile ci --no-fail-fast`; both Clippy commands cover `--locked --workspace --all-targets --profile ci -- -D warnings`, one with `--features updater`; the remaining commands are `cargo fmt --all --check` and `git diff --check`.

Phase 2 follow-up working-tree review and native QA are pending. The required native checks remain adding the existing work folder, active usage and new-chat identity, paused-chat cross-account resume and permission mode, cross-account fork, removal and one-account resume, `SET-USAGE-02`, `DAE-STOP-01` and `DAE-CRASH-01`. Browser sign-in for a fresh `/tmp` folder needs Jason’s participation or a named Blocked result. Native Windows remains Blocked as declared; automated file/path coverage is not a native Windows pass. Phase 3 has not started.

### Phase 2 review corrections

The first working-tree review found three must-fix issues on candidate diff `293480be91b99911509b1da4ed35c49fadc6575185870cd3aaeaa04aa9e29e27`: the app inferred a newly added account from two snapshots and could select another window’s addition for login; a login status preflight was not registered as a job, so Remove or Cancel could miss it and authentication could start afterward; a legacy chat pinned through the inherited shell environment could fork from the native folder instead of its actual transcript origin. The reviewer independently passed the selected daemon, cache, app and core checks; the findings remain recorded here.

“Add and log in” is now one daemon action that keeps the created account ID through admission. The app sends that action directly. Login admission, Remove and Cancel share the account-mutation lock; the pending request is registered before its status preflight starts, and both the preflight and login process use the same cancelable process owner. The worker verifies its request identity before starting login or publishing status. Remove also prevents an older general status refresh from publishing a forgotten account. Launches and legacy fork origins now share role, shell and process environment-pin resolution.

The new deterministic regressions use a disposable fake Claude executable and filesystem barriers. Two windows start from the same account snapshot; B is admitted before A, both status preflights remain blocked, and each login must later start with its own folder. Separate Remove and Cancel cases interrupt a blocked preflight, then verify the worker exits, the subprocess is reaped, no login starts and no stale job or status returns. A session-manager regression starts a shell-pinned chat with one registered account, adds a second, then checks the fork preserves its UUID resume, pinned folder and transcript permission mode. These changes do not launch a real agent or read an account’s credentials. The targeted daemon command `cargo test --locked -p runner-daemon --profile ci claude_ -- --nocapture` exited 0 with 93 passed; evidence is `target/validation/955-phase2/review-fixes-targeted.log` and its exit-code file.

All seven required gates pass after these review corrections (exit 0): app tests (785 passed, three existing ignored), daemon tests (1,287 passed), core tests (64 passed), workspace all-targets Clippy, the same Clippy command with updater, formatting and whitespace. Commands, actual exit codes and logs are recorded in `target/validation/955-phase2/gate-exits-review-fixes.json`; logs use the `*-review-fixes.log` suffix. The core protocol round-trip fixture now exercises the add-and-login request. Follow-up working-tree review is requested next; phase 2 remains uncommitted and native QA has not started.

### Shutdown admission correction

The follow-up reviewer confirmed that the three original findings were resolved, then found one remaining lifecycle race: daemon shutdown drained existing login jobs without closing admission, so an already dispatched Login or AddAndLogin request could register a job after cleanup. The new deterministic regression holds the mutation lock, dispatches each request type on another thread, runs shutdown cleanup, then releases the request. Before the fix it failed with exit 101 because the delayed Login was accepted; `target/validation/955-phase2/shutdown-admission-before.log` retains the failure. No real agent or account was involved.

The job registry now contains a permanent closed state protected by its existing registration mutex. Shutdown sets that state before draining jobs and clearing their signing-in statuses; the shared admission helper rejects later registrations with a shutdown error. The regression checks that both request types are rejected, neither fake preflight nor login starts, and no job or status is registered. The account test group, including the existing pending/running cancellation and interleaved additions, passed afterward (9 passed, exit 0, `shutdown-admission-after.log`). The follow-up review and native QA remain pending.

The full required gates after the shutdown fix all exited 0: app tests (785 passed, three existing ignored), daemon tests (1,288 passed), core tests (64 passed), Clippy with and without updater, formatting and whitespace. Commands, exit codes and individual `*-shutdown-fix.log` files are listed in `target/validation/955-phase2/gate-exits-shutdown-fix.json`. The source-only delta from the prior reviewed candidate is retained at `shutdown-admission-delta.patch`. Phase 2 remains uncommitted for follow-up working-tree review before native QA.

### Initial Phase 2 native QA and scope changes

The reviewer posted **NO REMAINING MUST-FIX ISSUES** for HEAD `cfaa9f0e84523639750ccd492b0d51912ddb70f2` plus full diff SHA-256 `14380f516931061fd48c5ac1e0108ed2a7f165af98d7adcfc95e40f4ec33d1cd`, after resolving all four findings. Its account test group (9 passed) and whitespace check exited 0 independently. QA then built that frozen candidate in the development environment. Partial report, matrix, sanitized readbacks, binary/sidecar hashes and screenshots: `/private/tmp/runner-955-phase2.z_gmijsj/{partial-report.md,partial-matrix.json}`. The candidate was unchanged at pause.

| Native Phase 2 check | Initial result | Evidence / limit |
| --- | --- | --- |
| Browse/add existing Work without browser; signed-in null-plan acceptance | Passed | Existing Work status signed in; exact rendered null-plan line remained to recheck after privacy masking |
| Rename, Make main, Use, native removal guard | Passed | Work main/active transitions and fixed native identity; native remained non-removable when neither main nor active |
| Usage dropdown, active footer, separate weekly values | Passed, partial matrix | Personal 86%, Work 24%; full sizing/zoom/collapse coverage remained unfinished |
| Running native identity; single-pane chip; Account/config placement | Passed | Native chat retained its account after switching; Account beside Runtime and config between cmd/cwd |
| Native initial Haiku-low marker turn | Passed | Owned source recorded marker and ACK |
| Native-origin resume on Work | Failed | Two attempts exited 1 in Claude’s startup screen with `api.anthropic.com ECONNRESET`; ID and origin metadata stayed correct |
| Resumed recall, source append and permission UI | Blocked | Runtime stopped before the prompt |
| Fork new key and source preservation | Passed | Work origin/new ID; original source hash unchanged |
| Fork transcript/recall; fresh Work turn | Failed / dependent checks Blocked | Fork stopped at the theme picker; Jason confirmed missing onboarding state as a phase 2 must-fix |
| SET-USAGE-02 complete size/zoom/collapse matrix | Interrupted | QA paused safely for Jason’s requested selector revision; no full pass claimed |
| DAE-STOP-01 and DAE-CRASH-01 native | Blocked | Shared development daemon contains pre-existing sessions; no supported isolated native endpoint fixture was found, and QA did not stop/crash it |
| Removal and one-account resume | Not run | Paused before this step |
| Browser sign-in, Log in, Open sign-in page, Cancel while signing in | Skipped — owned by Jason | Human scope amendment `01M5CR7B2YS66QN0DYSWZ4RE95`; these are Jason’s manual checks, not QA blockers |
| Native Windows | Blocked | Host unavailable |
| Phase 3 linking/sharing | Skipped | Not implemented yet |

QA stopped safely and released the source for edits. It retained three owned chats, its wrapper and the Work registry selection for the later revised-candidate recheck; the partial report lists their exact IDs and cleanup obligations. No source/test-record edit or further UI action by QA occurred at pause. The shared development daemon and pre-existing sessions must stay intact. Privacy masking initially missed a truncated identity in transient tool output; retained screenshots were remasked, and the capture pipeline was strengthened. Transient output is not shareable evidence.

### Compact selector and onboarding corrections

Jason requested a smaller account selector and simpler menu from the live screenshot. The usage header now uses a 24px borderless text trigger, a small chevron and a menu aligned to its right edge. Account names and weekly percentages share one line, the active account has a leading check, and Manage accounts has a settings icon below a separator. Shared select defaults stay unchanged. The targeted compact-menu test covers 100%/150% zoom, bounded trigger size, inline values, check placement, menu alignment and one keyboard selection. Its first run failed on inherited row line height (exit 101, `compact-selector-targeted.log`); explicit compact line height fixes it, and `compact-selector-recheck.log` exits 0.

Jason authorized an onboarding preseed for added signed-in accounts (`01M5CS5RZZ92KTZY73ADJPK16G`), including the narrow write to `~/.claude-work/.claude.json` during QA. The opening probes use throwaway HOME/config/cwd folders, a dummy API key and no submitted prompt. The first two runs observed native global hash changes during the interval and do not establish isolation; the writer was not identified, and no original content was restored or overwritten. The decisive repeat denied all home writes with the OS sandbox and retained unchanged native hashes. In that run, setting only `hasCompletedOnboarding: true` skipped initial connectivity/theme setup and reached project trust; `lastOnboardingVersion` alone did not bypass setup. Probe scripts/results and sanitized screens are under `target/probes/955-phase2/onboarding/`; `result-sandbox.json` is the isolation-verified key-selection evidence.

The same isolated comparison with a copied owned transcript and path-resume arguments reproduced `ECONNRESET` without completion, while completion alone reached trust. A tiny real Work headless path-resume/fork exited 0, returned OK with the assigned new ID, and preserved the owned source. `resume-diagnosis.json` records only booleans, IDs/paths and exit results, without credentials or identity. Together these show the observed runtime failure occurs in first-run connectivity setup; the Work sign-in and authenticated path-resume/fork inference work in print mode. This does not substitute for native resume, append, permission or fork recall QA.

The implementation marks only `hasCompletedOnboarding` after successful sign-in and when status confirms an added folder is signed in, skips completed files, and preserves every unrelated JSON byte using the existing member-span parser and atomic config writer. It refuses native folders and links to native configuration. Invalid files remain unchanged and the account reports setup/status unavailable. The targeted onboarding group passes (5 tests, exit 0, `onboarding-targeted.log`), including existing signed-in add and completed fake login, byte preservation, no rewrite when complete, malformed-file preservation, native guards and symlink rejection. No implementation code has written the real Work preseed yet; the app is still paused on the prior candidate.

Jason changed publication order after the QA pause: finish these corrections, run checks/review, then commit and push phase 2 to #970 before any QA rerun. QA remains paused; the initial failures and blocked checks above are retained and the feature is not claimed fully validated. Phase 3 has not started.

All seven required gates for the selector/onboarding correction exit 0: app tests (786 passed, three existing ignored), daemon tests (1,293 passed), core tests (64 passed), workspace/all-targets Clippy with and without updater, formatting and whitespace. Exact commands and actual exits are in `target/validation/955-phase2/gate-exits-selector-onboarding.json`, with individual `*-selector-onboarding.log` files. The corrected working-tree diff is sent back to the reviewer before the authorized phase 2 commit/push. QA remains paused until publication and a later explicit recheck handoff.

The selector/onboarding correction passed follow-up working-tree review on 2026-10-20: reviewer posted **NO REMAINING MUST-FIX ISSUES** for HEAD `cfaa9f0e84523639750ccd492b0d51912ddb70f2` plus full diff `f3ee88c7ce5e608a08b882d053bf879c03857d70d439263f897149dd414734e4`. Independent onboarding (5), account-service (10), select (10), menu (8) and whitespace checks exited 0; all seven reported gates were inspected. Review confirmed byte preservation, native guards, cancellation/removal/shutdown validity and unchanged default menu behavior. Per Jason’s revised order, the reviewed phase 2 change is committed and pushed before QA reruns; native QA is still incomplete with its historical matrix retained above. Current main advanced through #973 and #974, so publication rebases the branch onto that mainline and rechecks it before pushing.
