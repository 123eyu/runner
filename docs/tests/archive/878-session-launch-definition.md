# #878 — session launch definitions

Archived in the 0.16.2 docs sweep on 2026-10-15. Lasting live checks are maintained as RT-LAUNCH-01 in [Runtimes](../regression/runtimes.md) and MIS-ROSTER-03 and MIS-FEED-02 in [Missions](../regression/missions.md); LD-3 was a one-time upgrade check. Historical verdicts and platform gaps below are preserved.

Candidate: `fix/878-session-launch-definition` in its authorized worktree. The [live-QA brief](https://github.com/yicheng47/runner/blob/v0.16.2/docs/impls/briefs/878-session-launch-definition-qa.md) and [implementation plan](../../impls/archive/878-session-launch-definition.md) define the changed behavior. Jason's 2026-10-13 live-QA request superseded the 2026-10-12 waiver, then narrowed the remaining mission checks to behavior affected by this PR, explicitly including MIS-CONFIG-01. QA ran the native development-app pass; coder and reviewer stayed off the frozen app/Cargo candidate.

## Changed behaviors and selected checks

| Changed behavior | Selected checks | Reason |
| --- | --- | --- |
| Spawn records the effective configured launch inputs; Resume and fork replay them after role edits | Manager launch-definition, runtime override, resume, pi and fork tests; LD-1 and LD-2 | These paths previously mixed recorded values with the current role |
| Restart rebuilds from the current role and roster slot | Manager slot restart and launch-definition tests, failed runtime-changing Restart and repository transaction rollback; LD-2 | Restart is the explicit update path, including runtime and prompt; a failed launch must keep its runtime, definition and conversation key consistent |
| Executable resolution, shell probe environment and permission policy remain live | Executable-override, Cursor/Grok probe and permission tests; native MIS-CONFIG-01 and LD environment/permission observations | These inputs describe the machine and must apply on every launch; full global-override/runtime variants are outside the final focused live selection |
| Migration 0026 converts all legacy rows before 0027 drops nine columns | Database conversion, foreign-key and schema snapshot tests; LD-3 when candidate identity allows | Conversion must preserve effective values and attention children across the schema change |
| Header, side panel and CLI show recorded values; CLI omits definition env | Session operation and CLI rendering tests | Displayed fields must match Resume and JSON must not disclose keys |
| Mission slots use recorded launch inputs, effective overrides and migrated deleted-slot settings | LD-2, MIS-CONFIG-01, focused MIS-ROSTER-01 and MIS-ROSTER-02 on macOS | Jason narrowed the remaining mission selection to paths affected by this PR; unrelated delivery/draft/watch regressions are outside scope |
| Unedited-role argv stays unchanged | Existing spawn/resume/fork argv goldens | Detects accidental runtime argument changes |
| Terminal `mission_stopped` renders a stopped divider on archive | Feed grouping regression test; MIS-FEED-01 and native owned-mission stop/archive screenshots | Pre-existing feed bug explicitly added to this PR by Jason on 2026-10-13; the empty signal payload previously rendered as a raw `{}` card. Stop all pauses slots without emitting this terminal signal under the unchanged lifecycle contract |

The mission human response on 2026-10-12 resolved the permission-args conflict: definitions retain configured permission args. Direct chats strip them live; mission Auto/Bypass strip and replace; RoleDefault retains recorded flags.

## Automated validation

Final candidate checks on macOS, 2026-10-12. Each command wrote its output to the listed log; exit codes came from the command process itself.

| Command | Exit | Result | Log |
| --- | --- | --- | --- |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,251 passed, 6 skipped | `/tmp/runner878-nextest-final.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | Passed | `/tmp/runner878-clippy-final.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings` | 0 | Passed | `/tmp/runner878-clippy-updater-final.log` |
| `cargo fmt --all --check` | 0 | Passed | `/tmp/runner878-fmt.log` |
| `git diff --check` | 0 | Passed | `/tmp/runner878-diff-check.log` |

Earlier workspace runs exited 100 while existing fixtures still lacked launch definitions, old expectations followed edited roles, and the new CLI test used a short mission-session ID. The fixtures and expectations now match the recorded-definition contract, and the CLI test uses a full session ID. The final workspace run above passed. Schema snapshot, migration conversion, permission-mode resume, executable override, roster conversion, UI values and CLI redaction checks are included in that run. Argv goldens remain unchanged; the catalog-speed fixture only changes its recorded storage values.

Working-tree review found that a failed runtime-changing Restart could publish a new native conversation key with the old definition. Restart now commits its runtime, definition, key and generation together before spawning, and has no fallible definition write after PTY creation. A failed spawn leaves a consistent stopped row; a failed launch-definition write rolls back the transaction. The focused regression command, `cargo nextest run --locked -p runner-daemon --cargo-profile ci -E 'test(failed_runtime_changing_restart) | test(resume_rolls_back_key)'`, passed both tests with exit 0 (`/tmp/runner878-review-fix-focused.log`); both also passed in the final workspace run. Its first compile attempt exited 101 due to a fixture caller and test import, corrected before the passing run.

The first [Windows CI run](https://github.com/yicheng47/runner/actions/runs/38258867764/job/114843917321) passed Clippy but failed the migration test: `/old/codex` is rooted without a drive on Windows, so it does not exercise native absolute-path conversion. The Test step exited 1 with 2,166 passed, 1 failed and 4 skipped. The fixture now uses `C:\old\codex.exe` on Windows and `/old/codex` elsewhere, asserts that the path is absolute on the host, and keeps the conversion assertion unchanged. Production conversion code is unchanged.

The first local workspace rerun after that fixture correction exited 100: the migration passed, but two unchanged model-discovery process tests failed with a missing-catalog assertion and a query timeout (`/tmp/runner878-nextest-ci-fix-first.log`). A focused rerun of those two tests and the migration passed all three, exit 0 (`/tmp/runner878-ci-fix-focused.log`), without further code changes. The full workspace suite was then rerun without an overlapping Cargo build; its result is recorded in the validation table above.

The second [Windows CI run](https://github.com/yicheng47/runner/actions/runs/38261155611/job/114851690350) passed the absolute-path conversion but exposed another Unix-only assumption in the same test: shell args were expected to include `-l` on Windows. The Test step exited 1 with 2,166 passed, 1 failed and 4 skipped; macOS CI passed. The shell fixture now records a native Windows `cmd.exe` path or Unix `/bin/zsh`, and explicitly expects no login args on Windows and `-l` on Unix, matching the existing launcher. Shell-command preservation remains asserted. Production code is unchanged.

## Pre-existing feed bug added at Jason's request

On 2026-10-13 Jason authorized fixing the pre-existing stop/archive feed card in this PR as a separate commit after #878. `mission_stopped` now uses the same divider as `mission_start`, with the label derived from the signal: `MISSION STOPPED · <time>` or `MISSION STARTED · <time>`. The regression test retains the empty stop payload and verifies both lifecycle events are dividers. The fix changes no launch, migration or mission lifecycle behavior. QA verified MIS-FEED-01 and native owned-mission stop/archive on the reviewed rebuilt candidate: Stop all shows the paused overlay and zero live slots; Archive emits one `mission_stopped` event and renders the stopped divider with a timestamp, matching the started divider and showing no raw `{}` card. Screenshots and exact event/state evidence are in the [2026-10-13 run](../runs/2026-10-13-macos.md).

Automated checks on the working-tree feed fix above `0b0542e7`, 2026-10-13. Logs are under `/private/tmp/runner878-feed-fix-validation-w3r_e1po/`; each exit code was read from the command's process result and retained in `results.json`.

| Command | Exit | Result | Log |
| --- | --- | --- | --- |
| `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,252 passed, 6 skipped, including stopped/start divider tests | `nextest.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | Passed | `clippy.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings` | 0 | Passed | `clippy-updater.log` |
| `cargo fmt --all --check` | 0 | Passed | `fmt-check.log` |
| `git diff --check` | 0 | Passed | `diff-check.log` |

After QA released C2, the brief and records were folded into #878 and the feed fix kept as a separate following commit, then rebased onto `origin/main` at `ea5dc0e6` on 2026-10-13. Both QA-tested feed source hashes and the complete launch/migration/CLI trees remained unchanged. Main additionally supplied #885's Feed-selection change and regression test. The rebased workspace suite passed 2,253 tests with 6 skipped; both Clippy variants, fmt-check, `git diff --check` and the committed-range diff check exited 0. Direct process exits and logs are retained under `/private/tmp/runner878-feed-fix-validation-w3r_e1po/rebased/` in `results.json`. C1/C2 native results retain their recorded candidate identities.

## Live QA

Jason's 2026-10-13 request superseded the earlier waiver. The bounded macOS/Codex run is recorded in [2026-10-13-macos.md](../runs/2026-10-13-macos.md). Verdict: **Failed blind-recall attempts retained; focused Runner behavior verified, no established Runner regression.** LD-3's precise retry passed; both MIS-ROSTER-01 recall attempts missed the role token, while the owned resumed transcript proves the original launch message preserved both role and addendum tokens. The worker prompt does not name its own handle or lead, so that earlier recall expectation was invalid. Do not describe this as an unconditional pass or complete mission regression pass.

C1 was clean at HEAD `0b0542e7b4bed8c8be93187d053b139fde6b9344`; C2 retained that HEAD with reviewed diff SHA-256 `45bdd9c8b28f38f7e4b753c0d59fb4a170c07ff467e3d4c27d717aff9a2a148e`. C1 supplied completed LD/migration evidence; C2 added only the feed source change and supplied focused roster/config/feed evidence. Source/artifact/running executable/development endpoint identities were verified and frozen through testing. Evidence is `/private/tmp/runner878-qa-20261013.cg_1hn93/evidence/`; stopped development database-file backups are retained in its `db-backup/` with hashes. QA never opened SQLite. Native Windows remains unverified; CI is separate automated evidence.

| Case | Result | Evidence |
| --- | --- | --- |
| LD-1 — direct chat Resume ignores role edits | Passed | Native original key/model/low/prompt and fresh environment retained after edits; run LD-1 row |
| LD-2 — mission Resume freezes and Restart rebuilds | Passed | Both slots retain original values on Resume; native Restart uses edited values and new conversation keys; run LD-2 row |
| LD-3 — previous-build session conversion | Failed first recall; Passed precise retry and migrated values | Wrong internal marker first, original token on blind retry; original key and converted launch values retained; both attempts preserved |
| MIS-ROSTER-02 — v1 conversion/deleted slot/idempotence | Passed | Approved backed-up legacy fixtures; session-derived deleted-slot settings, running recall, archived read-only view and byte-identical second startup |
| MIS-ROSTER-01 — focused preserved launch/roster | Passed preservation; Failed first/precise model recall | Original roster/launch/env/key and transcript tokens retained after edits/remount; role-token recall misses retained, invalid worker-name expectation marked not applicable |
| MIS-CONFIG-01 — slot overrides and live Bypass | Passed | Start/Resume use slot gpt-5.6-luna/low over role gpt-6-luna/medium, live never/danger-full-access; configured old role permission args unchanged |
| MIS-FEED-01 and owned stop/archive divider | Passed | Pending card/content/author persist, native answer clears pending, pause has zero slots, archive shows STARTED/STOPPED time dividers and no raw `{}` card |
| MIS-CREW-01 and MIS-LIFE-01 | Passed, supplementary | Completed before scope narrowed; native form and same-key stop/resume/recall/archive evidence retained |
| Unrelated remaining mission rows | Skipped — outside final scope | Jason narrowed coverage to mission behavior affected by this PR; incomplete attempts receive no pass claim |
| Full SET-AGENT-02/03 and RT-LIFE-07 variants | Skipped — outside final focused selection | LD overlap is recorded; global executable, fast-speed and full runtime-row variants were not claimed |
| Native Windows / non-Codex runtimes | Skipped; Windows unverified | Outside this native macOS/Codex run; Windows CI remains required |

Cleanup completed: all six owned missions and two direct chats stopped/archived; owned apps and development daemon stopped and `ps` confirmed absent; temporary wrappers unregistered but retained. Only six owned trust stanzas were removed, preserving every other TOML value and restoring pre-QA semantic configuration. Existing sessions were never targeted; authorized daemon lifecycle effects and all setup/control/fixture deviations are recorded. Roles, crews, worktrees, evidence and archived conversations remain. At the original QA handoff, records were uncommitted for coder's authorized folding and separate feed-fix commit.

### Final rebased build/display recheck

After coder folded the original records and rebased onto `ea5dc0e6`, QA accepted the bounded C3 handoff at HEAD `96d99879b87df6355992aaf2ac2379ade5aa8e66`, with documentation-only dirty diff SHA-256 `b064147dc73baa4ca4b588652f63b374a80872bc188e52c243d10a980d4dc307`. Main includes #885's Feed-selection input change; reviewed feed source and launch/migration/daemon/CLI source were unchanged. QA rebuilt, verified source/artifact/running executable/development endpoint identities, and opened only the existing owned archived mission. **Passed:** native STARTED/STOPPED timestamp dividers and no raw `{}` card; archive timestamp, zero live slots and feed events unchanged. No new agents/chats/missions or expanded regressions were run. [C3 run addendum](../runs/2026-10-13-macos.md) retains exact hashes and native `C3-archived-feed-start-stop-dividers.png`.

C3 cleanup stopped the owned app and development daemon and verified their PIDs/executables absent; Codex configuration remained byte-identical, with no trust additions. Existing sessions were untargeted; wrapper files/evidence remain. The bounded C3 display pass leaves the C1/C2 matrix, retained recall failures and focused overall verdict unchanged. QA leaves these final record addenda uncommitted for coder and releases source/Cargo.
