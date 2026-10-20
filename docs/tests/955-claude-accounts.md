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

Phase 2 fork-placement and per-account usage probes, account behavior, review and QA have not started. Phase 3 sharing probes, implementation, review and QA have not started. No mission code has read account credentials or written into `~/.claude` or `~/.claude.json`.

Native Windows is unavailable to this crew. A folder with spaces, the credentials-file route and the configuration-junction route require Jason’s native Windows checks after implementation; they are not claimed as validated by macOS or CI.
