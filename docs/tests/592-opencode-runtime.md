# OpenCode v2 focused validation

Scope: [brief](../impls/briefs/592-opencode-runtime.md), local uncommitted work on `feat/592-opencode-runtime`. Native version: **v2.0.26**, macOS read-only help and session-free metadata probes on 2026-10-12. No native inference/smoke, global skill/config mutation, installation, full daemon/app/workspace suites, commit/push/PR.

## Selected behaviors and checks (before execution)

| Changed behavior | Smallest selected check and reason |
| --- | --- |
| Identity/catalog/API/settings/asset | Core runtime wire round-trip; filtered catalog expectations; app provider/asset checks: covers registration and presentation |
| PTY launch/persona/permissions/model/resume | OpenCode adapter and fake-session tests: covers composed prompt once, native args/config, same-cwd distinct keys and mission policy |
| Exact native history | Synthetic v2 SQLite ID query tests: covers missing history without newest/cwd fallback |
| Active-route report/status | Stub v2 TUI plugin and hook watcher tests: covers new/load/home, child and stale events, bounded cleanup and existing generation admission |
| Dynamic model query | Parser and synthetic interrupted process tests; existing focused model-cache tests: covers malformed/empty/offline fallback and timeout |
| Skills/MCP | Managed-skill ownership tests and JSONC copy/edit/remove preservation tests: covers shared install root and unchanged foreign files/config |
| Native smoke | Later authorization: RT-LIFE-01–09 (PTY/model/interrupt/resume), RT-START-01–03 (crew/persona/wrappers), RT-NAV-04 (same-cwd isolation); OpenCode-specific load/home/child/bridge cases; MIS-CONFIG-01/MIS-MSG-01/MIS-ASK-01 (bus and permissions); SET-AGENT-01/02, SET-SKILL-01/03, SET-MCP-01/02 (metadata/config/brand). These are required future OpenCode instances/adaptations, not current passes |

## Results

All commands use this worktree's own Cargo target directory and `--profile ci`. GPUI commands include `--features gpui_platform/runtime_shaders`. No full daemon/app/workspace test suite ran.

| Command / selected check | Result |
| --- | --- |
| `cargo test --locked -p runner-core --profile ci runtime_wire_names_round_trip` | Passed, 1 test |
| `cargo test --locked -p runner-daemon --profile ci opencode --no-fail-fast` | Passed, 18 tests after review fixes: adapter/config/native store/route epochs/stale generation/Node TUI stub/MCP copy and symlink/fake chat+mission+resume, heartbeat loss and shell-only/inherited environment |
| `RUNNER_UPDATE_GOLDEN=1 cargo test --locked -p runner-daemon --profile ci golden --no-fail-fast` | Passed, 15 characterization tests; updated only registry/spawn/config expectations affected by adding a runtime. This filter also matched the existing synthetic session scenario replay (passed); no live agents |
| Daemon exact filters `hooks_are_gated_per_runtime`, `compose_path_includes_fallback_cli_dirs`, `spawn_failure_timeout_and_cache_write_failure_are_contained`, `concurrent_refreshes_share_one_query_and_obsolete_results_are_discarded` | Passed, 1 test each; bounded synthetic query is killed/reaped and obsolete result discarded |
| Daemon exact managed-skill filters `refresh_updates_marked_folder_and_leaves_foreign_folder_alone`, `antigravity_root_preserves_user_symlink_even_with_marker` | Passed, 1 each; exercises shared ownership rules in temporary roots |
| `cargo check --locked -p runner-app --profile ci --features gpui_platform/runtime_shaders` | Passed |
| App `--bin Runner` exact filters `opencode_brand_icon_is_bundled`, `provider_and_mission_marks_keep_their_tint_only_while_live`, `agent_enablement_preserves_shipped_defaults_and_explicit_overrides`, `root_runtime_ownership_matches_agent_locations` | Passed, 1 test each, both theme identity assertions included |
| `RUNNER_UPDATE_CATALOG_GOLDEN=1 cargo test --locked -p runner-app --bin Runner --profile ci --features gpui_platform/runtime_shaders ui_catalog_golden` | Passed, 2 tests; added OpenCode skill caption, no usage/quota surface |
| App exact model enablement filter `cursor_and_grok_model_discovery_follow_defaults_and_explicit_disablement` (now also covers OpenCode) | Passed, 1 test |
| Daemon `ops::runtime::tests::`, `catalog_golden`, `spawn_goldens` without update flags | Passed, 4 / 7 / 1 tests; normal verification of selector, registry and stable fresh/resume launch expectations |
| `cargo clippy --locked -p runner-core -p runner-daemon -p runner-cli -p runner-app --all-targets --profile ci --features gpui_platform/runtime_shaders -- -D warnings` | Passed; selected changed crates, not workspace Clippy |
| `cargo fmt --all -- --check`; `git diff --check` | Passed |
| Original favicon ICO frame versus PNG decoded RGBA pixels | Equal, 48×48. PNG SHA-256 `1ee881194c44c8f5c1cf24d53d2e8e2b040936d1dc54b759f14d7e41266b6176` |

Earlier attempts are retained here: initial native metadata failed with missing scratch HOME and was repeated after creating it; fake-session test initially passed cwd in the model-override argument (fixed fixture); status test expected Idle where the existing reducer returns Ready at turn end (fixed expectation); new test fixture needed OsString and avoided unwrap_err on a non-Debug native entry. Registry enablement expectations needed the new runtime. An app `--lib` filter ran zero tests; the incorrect binary name `runner-app` was rejected; both were corrected to `--bin Runner` and the exact asset test passed. These are not native smoke failures. Cargo reports an existing future-incompatibility notice for dependency `block 0.1.6`; the checks passed without dependency changes.

Native observations: installed help/version, private session-free `/openapi.json`, empty `/api/model` snapshot, inline v2 model normalization, and JSON then JSONC config precedence. Session schema comes from the isolated native SQLite database. A plugin metadata query returned an empty initial snapshot; it does not establish TUI plugin loading. Probe artifacts remain under `target/opencode-probe` (ignored), without account data. Owned metadata server process groups were stopped. No chat/mission or managed user-root mutation was performed.

Native macOS PTY, TUI plugin loading/status shape, first-turn receipt, actual selected model, permission denials/auto approvals, session switching/resume, and mission bus/tool behavior remain unverified. Windows ConPTY and wrapper execution remain unverified. Full CI/platform gates were not run. No terminal fixture is fabricated from synthetic output. Surfaced approval/question events, native Fork, update and quota controls remain unavailable. Native variants were subsequently implemented locally; see the deferred validation note below. Native flat/deeply nested skill forms are not enumerated in Runner's folder catalog. Dual JSON/JSONC global MCP editing is refused explicitly; single-file JSON/JSONC and symlink targets are covered.

Reviewer completed read-only full working-tree review: **Clean review**, 2026-10-12 08:27 UTC. Implementation remains uncommitted; no push, PR, merge or issue change. The human's brief edits remain intact.

## Review round 1 and focused fixes

Reviewer reproduced two plugin failures with an in-memory Node stub: dropped identity/outcome snapshots after reporter failure, and historical completion arriving after delayed cache synchronization. Reviewer also found Skills reading process XDG while launch uses captured shell XDG. Architecture tracing matched current RuntimeAdapter/StatusHooks, runnerd SessionModel persistence, metadata and GPUI; no old Tauri/newest-cwd implementation was found.

Fixes add opt-in acknowledged reports without changing other providers' default neutral output/exit behavior; latest-state retry/backoff and one-second idempotent heartbeats; explicit bridge loss and a watcher timeout clearing the uncertain key before baseline fallback. History completion requires an observed running boundary and complete native time baseline. Node fixtures now cover nonzero/error/timeout recovery, delayed history, unknown route metadata, repeated heartbeat, bridge fallback and cleanup. Node-less hosts follow the existing pi test skip convention; this host executed the fixture with Node available.

Settings Skills catalog/read/save now use captured shell env, inherited OpenCode inline configs/data home have a lower-priority fallback, and shell discovery captures these variables. A core shell-only XDG fixture exercises catalog/read/save without mutating process env. Validation added `cargo test --locked -p runner-cli --profile ci --test hook_process acknowledged_opencode --no-fail-fast` (2 passed), `command_failures_are_neutral_without_autostart` (1 passed), and daemon `captures_agent_config_roots_from_login_shell` (1 passed). The 18-test OpenCode filter, changed startup-file golden, normal spawn/skills goldens and changed-crate Clippy passed. Native platform limitations above remain unchanged.

Round 2 retained failure: a running snapshot with incomplete time data followed by synchronized idle/outcome could keep Working alive through healthy heartbeats. Fix: when observed native running ends without a provable outcome, publish Ready without an outcome; late historical completion stays suppressed. The Node fixture repeats twelve seconds of healthy heartbeats after this incomplete-time transition and verifies Ready with no fabricated completion. An intermediate inherited-home fixture and old ACK-argument stub expectation were corrected during round 1 implementation; final checks passed. Round 3 read-only review requested after focused rechecks.

Round 3 outcome: reviewer independently reproduced the incomplete-time transition and twelve healthy heartbeats, confirmed Ready without outcome and suppressed late history, and checked plugin-source/expectation equality and diff whitespace. Runner review message `01M4QKJVG3ER4Y4ZNP13FSVYFA` states no remaining issues and confirms current architecture. This is a clean code review, not native platform certification. Final local handoff was sent to the mission channel; coding sessions stop under the brief's authorization, with the worktree retained.

## Draft PR handoff — 2026-10-12

After the local clean-review handoff, the user requested a draft upstream PR. The coordinator verified explicit repository author identity, checked that upstream `main` still matches base `cfcb2c43`, and folded the mission brief into the implementation commit. Submission changes no runtime behavior and adds no native smoke evidence. The checks above remain the selected local evidence; macOS/Windows native acceptance and full CI/platform gates remain pending. The upstream historical issue stays closed and is linked as background rather than automatically closed/reopened by this PR.

## Shared integration scope review — 2026-10-13

The human requested an audit of shared hook, MCP, skill and adapter changes. Keep the opt-in acknowledged reporter because default neutral exit codes cannot establish admission; keep the explicit Runtime serde name because kebab-case would serialize `OpenCode` as `open-code` instead of its `opencode` key. Captured shell environment remains necessary for Settings Skills to find/read/save the same XDG roots used by launch. Launch-context environment remains necessary for v2's model setting, which has no corresponding TUI flag.

Two boundaries were tightened: OpenCode MCP definition decoding now belongs to its own module and is selected through an opt-in adapter parser. The original `from_claude` body was verified byte-for-byte against upstream; other JSON clients retain their original decoder. Raw native JSON and copy/edit preservation remain intact. Removed the redundant environment-free skill-roots interface; an optional environment-aware override is used only by OpenCode, with the original static/CODEX_HOME mapping retained in the shared skills scanner for other agents.

Selected checks before execution: exact native-decoder isolation regression; OpenCode MCP copy/edit/remove, JSONC preservation and symlink cases; existing Copilot JSON/local and generic cross-client MCP tests plus MCP catalog expectation; OpenCode captured-XDG skill catalog/read/save, and existing static/CODEX_HOME roots; Runtime wire-name round-trip. These cover changed parser dispatch and root resolution only. Relevant core/daemon library Clippy and changed-file formatting are the compilation/lint checks; no native agent, full suite or UI test run is added.

Results: **11 exact tests passed**, with one test executed for every filter; commands and outputs are recorded under ignored `target/opencode-scope-review/results.json` and per-case logs. `cargo clippy --locked --profile ci -p runner-core -p runner-daemon --lib -- -D warnings`, changed-file `rustfmt --check` and `git diff --check` all exited 0. The shared adapter diff decreased from 48 to 38 added lines against the PR base; its remaining additions are registration and optional native-format/root/launch-environment seams. The core MCP protocol diff now contains only client identity/metadata registration, not OpenCode payload decoding. No native smoke evidence or full CI pass is added by this review.

## Existing-agent compatibility review — 2026-10-13

Selected checks: verify every pre-existing runtime retains the default optional MCP parser/read/write and skills-root dispatch, plus empty launch-context environment for fresh missions and resumed chats; verify existing hook failure neutrality/no autostart and native-command UTF-8/report behavior. These checks target the shared adapter and hook paths changed by this PR; they do not launch real agents.

Three exact tests passed, one test per filter: daemon `runtimes::tests::opencode_optional_extensions_keep_existing_agents_on_baseline`; CLI `command_failures_are_neutral_without_autostart` and `native_command_reporters_preserve_utf8_and_provider_neutral_results` in `hook_process`. The last test covers existing Codex, Claude Code, Copilot and Antigravity reporters. Its first attempt failed because the sandbox prohibited binding a temporary Unix socket; the same exact test passed outside the sandbox. This host exercised macOS paths, not Windows. Changed-file formatting and diff whitespace checks passed.

Two changes intentionally apply to shared launch setup: login-shell capture now includes `XDG_DATA_HOME`, `OPENCODE_CONFIG_CONTENT` and `OPENCODE_CLI_CONFIG_CONTENT`, and the fallback PATH includes `~/.opencode/bin`. Other agent processes receive the captured variables too; `XDG_DATA_HOME` can affect a CLI that honors it. The extra fallback directory can affect executable lookup when an earlier PATH entry does not resolve the executable. These effects are not isolated by adapter defaults, and the focused checks do not prove that every native agent ignores them. Existing explicit/captured PATH precedence and role environment overrides remain intact. Native/platform acceptance stays pending.

## macOS native feature run — 2026-10-13

See the [separate native run record](runs/2026-10-13-mac.md). Native testing found and locally fixed an early model-snapshot race and normal Stop clearing OpenCode's resume key. Dynamic model display and exact-key/native-history restore now have real macOS evidence. Inference-dependent chat, tool, persona and crew checks remain blocked by two native free-model request errors and the lack of a working private account. This is a partial feature validation, not a complete native pass. The user requested that these fixes stay local: no commit, push or PR update was performed.

## Native session allocation fix — implemented, verification deferred

The later Ling comparison in the run record obtained successful requests in the same isolated environment with OpenCode-generated IDs, while a fresh supplied `ses_<UUID>` returned 403. The earlier account-prerequisite explanation is not the established cause.

At the user's request, OpenCode's fresh resume plan now uses the existing `ResumePlan::fresh()` path: no assigned key and no `--session` argument. Existing TUI hooks capture the native ID after OpenCode creates the conversation. An actual captured prior key still produces the original exact `--session <key>` resume plan. No shared adapter interface, startup method or other runtime implementation changed in this allocation fix.

Existing OpenCode adapter/manager fixtures were updated to expect initially unset keys and then inject native conversation reports, preserving direct-chat isolation and mission-resume coverage. Only OpenCode fresh/missing/mission rows in the startup snapshots were edited; resume rows retain their prior expectations. These are prepared assertions, not executed evidence. No build, test, Clippy, snapshot regeneration or native run was performed after this change; only source formatting was applied. Changes remain uncommitted and unpushed.


## OpenCode effort variants — implemented, verification deferred

The user requested effort selection while keeping the prior no-validation and no-push constraints. Model discovery now reads each model's native `variants` IDs, deduplicates them and exposes only that model's choices through the existing effort selector. Empty variants (including the saved Ling 3.1 Flash Free metadata) expose only Default; custom variant IDs retain their native meaning.

The v2 root model configuration discards variants. Fresh explicit model/effort selection therefore adds the existing Runner OpenCode package as a native server plugin, with a prompt-admission hook that persists the variant using `session.switchModel` before model resolution. It initializes only one new root without history; child sessions, history, later native choices and resumed launches are preserved. Inline plugins and permissions are retained.

Parser/model filtering assertions, fresh/resume config assertions, fake chat/mission launch assertions and a native-API stub covering failed initialization, concurrent admission and later choices were added or updated, but **not run**. OpenCode spawn-effect expectations were edited directly, not regenerated by running tests. No build, Clippy, native inference, GUI verification, commit, push or PR update was performed. Native plugin admission ordering, session mutation and TUI variant synchronization remain required checks for later authorization.

## Crew working status — native contract corrected, verification deferred

The human reported successful Crew messages with missing working indicators. Read-only development CLI inspection of mission `01M4T142VRK7KTZ6MAGW66Z9Z3` showed both OpenCode root identities captured and hook observations reporting Ready. Its status events contained spawn Busy followed by hook Idle, with no hook Working transition.

The human explicitly authorized reading only installed program files under `/Users/lr/.opencode`. The installed executable's embedded TUI code implements `status(sessionID)` as a string read (`active[sessionID] ?? "idle"`), and native callers compare the result directly with `"running"`. Runner's plugin incorrectly read `.type` on that string, so its working branch never ran. The plugin now reads the native string directly. The existing stub now supplies native `"running"`/`"idle"` strings and asserts Working, replacing the object-shaped fixture that masked this defect.

Only the OpenCode TUI plugin, its test fixture and this record changed in this fix. No shared UI, runtime method, other agent, model request or retry behavior was changed. No agent was started, messaged, stopped or restarted. No build or test was run; no commit, push or PR update was performed. The running app and sessions still use their previously loaded plugin until the human rebuilds and restarts the affected sessions.

## Authorized local recheck — 2026-10-13

The human requested another native-machine check after a successful manual run. Four exact OpenCode tests passed. Isolated real OpenCode replies, foreground shell execution, hook Working→Ready/Completed, addressed human inbox read/ACK, Step 5 `high` variant and exact-key variant-preserving stop/resume passed at the backend/native-data level. No new production fix was needed. GUI control timed out and macOS accessibility denied the fallback window read, so visible indicators, keyboard interruption/recovery and other required UI portions remain blocked. See the [focused recheck section](runs/2026-10-13-mac.md#focused-recheck-after-session-variant-and-working-state-fixes--2026-10-13-16001630-cst) for candidate identity, scope, evidence and cleanup. Test sessions/mission were archived, temporary roles/crew deleted, private daemon/app stopped, and the user's development app retained. No full suite, Windows/CI, commit or push was performed.

## Rebase onto upstream v0.16.0 — 2026-10-13

At the human's request, fetched `upstream/main` and rebased the local feature commit onto `8102dfe17f8ca4333feebe1553c2f03825f947e7`. Feature HEAD is now `3e9cb493` with the original explicit ruizer author identity. The 15 dirty/new files were backed up under ignored `target/opencode-rebase-backup-20261013-171125`, stashed including untracked files and restored after rebase; Git reported no text conflicts. Fourteen files initially restored byte-for-byte; the startup snapshot incorporated upstream changes.

Upstream #881 now requires persisted launch definitions. The first focused run passed 22 cases and failed the old OpenCode missing-history fixture, which inserted a row without its definition. It now uses upstream's existing `insert_session_row` test helper, matching the updated Cursor fixture. Upstream #889 adds a crew join/leave notice to the coordination prompt; the startup golden exposed six stale OpenCode prompt expectations, which were updated to the upstream text. No additional production behavior was changed to resolve either mismatch.

Verification: `cargo test --locked --profile ci -p runner-daemon --lib opencode` passed **23 tests**; the exact `session::manager::tests::golden::spawn_goldens` test passed; `cargo check --locked --profile ci -p runner-cli` and `git diff --check` passed. These focused checks cover the rebased runtime/resume seams and startup expectations; no full suite, live-agent rerun, native UI or Windows pass is claimed for this new base. The earlier native-machine evidence belongs to its recorded pre-rebase candidate. Local fixes and compatibility edits remain uncommitted. No push or PR update was performed.

## CI fixes and upstream rebase — 2026-10-14

CI run `38378213148` tested published head `b8ee6573`: macOS passed 2,288 tests and failed 11; Windows Clippy rejected an unused Unix-only temporary-directory variable in `hook_process`. These are current failures, distinct from the earlier CI logs. The macOS failures were nine runtime/MCP list assertions missing OpenCode and two snapshots retaining the previous standalone model query and plugin sources.

Rebased onto upstream `e2b7b4e6` without conflicts. Upstream adds clipboard image handling and recency ordering in Direct's agent picker. The CI fix gates the temporary directory with `cfg(unix)`, updates only the affected expected lists and refreshes only OpenCode entries in the two snapshots. OpenCode discovery now owns a private server, so the catalog golden's nonexistent `/golden/agent` cannot start it and captures no query; the separate synthetic-server test covers settlement, query arguments and process cleanup. No runtime production behavior changed in this CI repair.

Selected checks: the 11 exact failed cases to cover their assertions; the OpenCode filter to cover the rebased adapter, status/variant plugin and private-server discovery; the spawn golden to cover composed crew prompts; the exact acknowledged-hook process test to cover the platform-gated fixture. Workspace Clippy with runtime shaders checks compilation/lints without requiring Xcode; formatting and diff whitespace are the final gates. Full workspace tests remain remote CI's gate. No real agents or native UI are selected for this repair.

Focused test results on this base: **36 passed**, all commands exited 0. The hook test first failed because the sandbox denied binding its temporary Unix socket; the same exact command passed with that permission.

```sh
cargo test --locked --profile ci -p runner-app --features gpui_platform/runtime_shaders --bin Runner -- surfaces::crews::tests::slot_runtime_select_restores_role_and_override_in_selector_order surfaces::roles::tests::a_legacy_shell_role_saves_only_after_an_agent_is_picked surfaces::roles::tests::role_edit_restores_disabled_runtimes_in_selector_order surfaces::settings::mcp::tests::catalog_rows_filter_every_server_and_reflect_registration_and_conflict surfaces::settings::mcp::tests::runtime_dropdown_filters_unavailable_disabled_and_unknown_agents --exact
# 5 passed
cargo test --locked --profile ci -p runner-daemon --lib -- ops::role::tests::create_rejects_shell_runtime_with_the_slot_override_error ops::role::tests::legacy_shell_role_updates_only_without_a_runtime_and_still_deletes ops::role::tests::update_rejects_setting_shell_runtime ops::slot::tests::raw_runtime_override_validation_preserves_inherit_and_rejects_unknown_names session::manager::tests::golden::spawn_goldens --exact
# 5 passed
cargo test --locked --profile ci -p runner-daemon --lib -- runtime_status::models::tests::discovery_catalog_golden session::manager::tests::golden::startup_files_golden --exact
# 2 passed
cargo test --locked --profile ci -p runner-daemon --lib opencode
# 23 passed
cargo test --locked --profile ci -p runner-cli --test hook_process acknowledged_opencode_reports_confirm_admission_and_bridge_loss -- --exact
# 1 passed (outside sandbox)
```

`cargo clippy --locked --workspace --all-targets --profile ci --features gpui_platform/runtime_shaders -- -D warnings`, `cargo fmt --all -- --check` and `git diff --check` exited 0. macOS/Windows CI on the amended commit remains the full platform gate; the earlier native-machine evidence still belongs to its recorded pre-rebase candidate.
