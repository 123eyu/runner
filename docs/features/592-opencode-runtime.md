# OpenCode v2 runtime (local fork)

Historical [issue #592](https://github.com/yicheng47/runner/issues/592) is closed as NOT_PLANNED. This is user-requested local fork work, not an upstream commitment. Scope and authorization: [mission brief](../impls/briefs/592-opencode-runtime.md).

## Native inventory, 2026-10-12

The installed executable reports `opencode v2.0.26`. Help and session-free metadata were probed with HOME/XDG/cache/state/tmp under this worktree's `target/opencode-probe`. No inference, login, package installation, or native chat was run. The first metadata attempt failed because the temporary HOME did not exist; it was created and the isolated probe repeated. Private metadata servers were terminated after each query.

The original `/docs/cli` describes V1. Use the official [v2 config](https://opencode.ai/v2/docs/config/), [TUI plugin API](https://opencode.ai/v2/docs/build/plugins/cli/), [CLI settings](https://opencode.ai/v2/docs/cli/config/), [skills](https://opencode.ai/v2/docs/skills/), [permissions](https://opencode.ai/v2/docs/permissions/), and the installed executable's `/openapi.json` instead. No external source checkout was inspected.

| Contract | Native mechanism / implementation target | Evidence / platform limits |
| --- | --- | --- |
| P0.1 identity | `opencode`, OpenCode, executable override; shared registry drives chats/roles/slots/API/archive/settings | Native help/version on macOS; selection/persistence covered synthetically; Windows native pending |
| P0.2 PTY | Default command is interactive; `--standalone` keeps a private server tied to the launched process and its environment | Help verified; actual alternate-screen/input/resize/exit fixtures pending authorized smoke on both platforms |
| P0.3 prompt | TUI `--prompt`; composed first turn once on fresh launch, shared delayed paste for Windows wrappers | Native help; synthetic launch/resume checks; native receipt pending |
| P0.4 bus | Private server inherits RUNNER environment; native `--auto` handles external-directory asks while retaining denies | Help; mission configuration checks; actual bus/tool work pending |
| P0.5 permissions | `--auto` auto-approves requests not explicitly denied; attended chats strip this flag | Help; no permissive global rewrite; configured denies can intentionally prevent mission work |
| P0.6 models | No TUI `--model`/`--variant` flags in installed help. Inline `OPENCODE_CONFIG_CONTENT` model changes fresh-session default. Efforts come from each native model's `variants`; a server prompt-admission plugin saves the selected native variant before the first request | Isolated `/api/config` normalized `test/model`. Root model config drops variants, so effort uses `session.switchModel` instead. History, children, subsequent native choices and resume retain their selection. Variant implementation is local and unverified |
| P0.7 identity | Fresh launches omit `--session` and leave the key unset until the TUI plugin captures OpenCode's own ID. Resume passes that exact captured ID through `--session`; the plugin reports new/load/focus IDs and clears on home. Exact read-only `session_v2.id` lookup for history availability | Native ID versus supplied UUID comparison is recorded in the macOS run; the local allocation fix remains unverified. No newest/cwd inference |
| P0.8 status | Per-TUI plugin, existing generation-authenticated hook transport; exact current root only; baseline if bridge unavailable | Native TUI API documented; synthetic plugin/status tests; visible approval/question detection deferred until a native surfaced signal is established |
| P0.9 platform | Shared native PTY and Windows launch quoting paths | macOS help/metadata only; both platform smoke gates remain unverified |
| P1 discovery | Bounded private `api GET /api/model`, registered cache/offline fallback | Installed OpenAPI, empty isolated response; parse/error/query tests. Native snapshots can precede plugin settlement; empty results keep fallback until refresh |
| P1 skills | Personal `$XDG_CONFIG_HOME/opencode/skills` (default `.config/opencode/skills`), `.claude/skills`, `.agents/skills`; managed Runner skill only shared `.agents/skills` | v2 documented roots; ownership/preservation fixtures; no global skill toggle. Runner's current folder catalog does not enumerate native flat Markdown or deeply nested skill forms |
| P1 MCP | v2 `mcp.servers`, local command array/environment and remote URL/headers, `disabled` | Installed schema; preserve comments, unrelated config and native fields with JSONC splicing. A global directory containing both JSON and JSONC is refused for MCP editing to avoid exposing lower-priority entries after removal |
| P1 brand/version | User-selected [favicon-v3.ico](https://opencode.ai/favicon-v3.ico), converted only for embedded loader; standard bounded version probe | Asset tests; live themes pending |
| P2 | Fork/update/usage disabled; session token/cost data is not provider quota | No inference or distribution/account assumptions |

## Acceptance boundary

Implementation and synthetic validation do not certify native support. macOS and Windows require later explicitly authorized PTY smoke for native first-turn receipt, exact session switching/resume, mission permission/bus access, terminal interaction and plugin status. Full CI and complete regression suites are outside this mission. See [focused validation](../tests/592-opencode-runtime.md) for selected checks and results.

The TUI plugin polls its own current route and synchronized session data, not global events. Root switches and returning home report the exact new ID or clear it. Viewing a child clears Runner's root resume identity; it cannot silently replace the root with a subagent ID. Reporter snapshots include a monotonic plugin epoch and sequence so reloads accept new reports and reject older callbacks; daemon admission additionally authenticates the spawn generation. Historical completion on loading an idle conversation is suppressed. Visible approval/question mapping is deferred: pending server requests alone do not prove a dialog is surfaced in this TUI.

Snapshots remain pending until `runner hook report --acknowledge` confirms daemon admission. The plugin maintains only the latest state and one bounded reporter, retries with backoff, and sends idempotent one-second heartbeats. Three failures trigger the existing explicit bridge-loss report. The watcher clears uncertain identity and falls back through `StatusBridgeFailed` after bridge loss or eight seconds without a heartbeat (startup allows sixty seconds). A terminal restart is required to reestablish semantic telemetry after this fallback. Outcomes require an observed native running boundary with a complete native time baseline; otherwise ending native running becomes Ready without an outcome. Unsynchronized route metadata also clears uncertain identity until the parent relationship is known.

Skills catalog/read/save use the same captured shell environment as launch and MCP. Shell discovery captures OpenCode's inline config variables and XDG data home; process inheritance remains the lower-priority fallback, with role overrides retained. No global environment mutation is used.

No external source repository was accessed. The isolated config metadata also lists the installed executable's `/Users/lr/.opencode` directory as a discovery source; no contents from that directory were copied into evidence. A session-free plugin metadata query returned an empty initial snapshot, which is not a plugin-load pass.
