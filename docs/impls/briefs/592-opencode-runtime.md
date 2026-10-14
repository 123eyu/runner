# OpenCode runtime — local fork implementation

Created for this mission on **2026-10-12**. This is a new task brief, not the historical OpenCode design.

Implement OpenCode as a Runner agent runtime using the **Peer coding** crew. `coder` leads implementation and targeted validation; `reviewer` reads the working-tree diff and requests corrections until review is clean.

## Implementation authority

Use the current worktree's `RuntimeAdapter`/`StatusHooks` contracts, `runnerd` session spawn/resume flow, runtime identity/metadata and GPUI presentation as the implementation authority. Read their current callers and current Grok/Cursor/pi adapters before changing the OpenCode integration. This brief states the requested behavior; it does not prescribe an older registration, IPC or session-capture implementation. Historical issues and archived specs are background only, including old Tauri/frontend paths and SQLite newest-session suggestions. Where documentation conflicts with current code or the installed OpenCode v2.0.26 contract, investigate and update the spec rather than copying the historical mechanism. Reviewer must explicitly verify the implementation fits the current architecture and does not reintroduce obsolete shared dispatch branches.

## Work directory and references

- Branch: `feat/592-opencode-runtime`.
- Worktree: `/Users/lr/我的/runner/.worktrees/feat-592-opencode-runtime`, based on `origin/main` at `cfcb2c430f44fd8893be638f32922e8d90f9d696`.
- Root checkout stays on `main`. Work only in this worktree; preserve unrelated user edits and use its own Cargo target directory.
- Historical tracking: https://github.com/yicheng47/runner/issues/592 (closed as NOT_PLANNED). This task is explicitly requested local fork work; do not reopen or edit the upstream issue or imply upstream has scheduled it. The older `docs/features/archive/30-opencode-runtime.md` is historical, with obsolete stack details and session-capture assumptions.
- Read `AGENTS.md`, `CONTRIBUTING.md`, `docs/arch/runtime-integration.md`, current Grok/Cursor/pi adapters, and relevant live regression cases.
- Current version-specific documentation: https://opencode.ai/v2/docs/, https://opencode.ai/v2/docs/config/, https://opencode.ai/v2/docs/build/plugins/cli/, https://opencode.ai/v2/docs/cli/config/, https://opencode.ai/v2/docs/permissions/, https://opencode.ai/v2/docs/skills/. Use the installed binary's isolated `/openapi.json` for its API schema. The unversioned `/docs/cli` describes V1 and must not determine V2 flags or API shapes.
- User-approved logo source: https://opencode.ai/favicon-v3.ico. Use that brand mark in the existing runtime presentation pattern. Decode/convert its format only if the current asset loader requires it; preserve the mark. Do not edit `.pen` files or redesign unrelated UI.
- Native executable found at `/Users/lr/.opencode/bin/opencode`, absent from this host's initial PATH. An isolated version probe on 2026-10-12 returned **OpenCode v2.0.26**. Read-only help/version probes are allowed; isolate their logs/cache/config under this worktree. Do not infer installed CLI behavior solely from the older issue's 1.18.30 probe or assume all current documentation describes the same major version.

## Scope and acceptance

First inventory the **current** native CLI against every P0 contract row. Add the spec on this branch as `docs/features/592-opencode-runtime.md`, link the historical issue with its actual closed status, and record native evidence and unresolved platform checks separately.

1. Register internal ID `opencode`, display name `OpenCode`, default command `opencode`, executable override, catalog and brand asset. Integrate selection and persistence across runtime chats, roles, crew slots/missions, API/CLI, archived sessions and existing settings patterns. Preserve other agents' identities and behavior.
2. Launch the native interactive TUI in Runner's PTY. Implement composed persona/coordination/goal delivery once for fresh starts, suppress replay on resume, preserve cwd, Unicode, spaces, shell and Windows wrapper quoting. Prefer existing runtime abstractions and supported native flags.
3. Capture the **exact active session** for the launched process, persist its native ID, resume by that ID and follow native new/load operations when an evidenced mechanism exists. Never use newest/last-session or cwd-only SQLite heuristics: concurrent same-cwd sessions and child sessions must remain distinct. Use native events/plugins or a narrowly scoped native API when supported; document actual unobservable focus-only operations instead of guessing identity. Missing-history and late/stale report handling must be safe.
4. Respect model IDs and defaults (normally provider/model). Use dynamic CLI discovery where supported with bounded background queries, existing caching and safe offline fallback. Expose effort/variant only when the interactive TUI supports an evidenced mapping; never invent a ladder or reuse another agent's flags.
5. Implement verified attended permissions and Crew's unattended permission policy, including mission bus access and exact resume. Preserve native explicit denies and user config. Scope any override to the current launch and honor native config precedence; no global permissive rewrite.
6. Follow the current P1 contract where usable: native status/events and subagent/stale-generation filtering, surfaced approval/question waits, version, real skill roots and the existing managed Runner skill. Automatic managed-skill installation should use shared `.agents/skills` when OpenCode reads it; avoid duplicates in every scanned root. Implement MCP settings only when the native format and preservation behavior are established. Defer unsupported mechanisms explicitly.
7. Optional fork/update/usage controls require native evidence. Do not fabricate weekly quota from session tokens/cost or provider billing. No provider/account login, secret logging, credential refresh or paid inference. Unsupported capabilities remain disabled.
8. Update `README.md` and `README.zh-CN.md` together, the relevant expectations and a focused validation record. Clearly separate implemented behavior, automated checks and actual native platform observations.

## Validation scope

Before testing, list changed behaviors and the smallest checks that cover them. Use synthetic native events/processes/config/SQLite data and filtered adapter/session/protocol/catalog/asset checks, plus focused terminal fixtures where meaningful. Include empty/malformed discovery, concurrent same-cwd sessions, child/stale events, resume, permission retention, interrupted queries and ownership/preservation cases affected by the implementation.

The human explicitly prefers narrow checks. Do not run a full daemon/app/workspace suite or a full GUI regression for this ordinary feature. Formatting, relevant compilation/Clippy and directly affected checks are appropriate; document full CI/platform gates not run. Build/cache time is not a reason to broaden test scope.

Use `--profile ci` for local Rust checks and `--features gpui_platform/runtime_shaders` when GPUI compilation is needed on this Mac. Do not install Xcode or upgrade the system. Do not share another worktree's `CARGO_TARGET_DIR`.

No real OpenCode chat/mission, inference, account/config/global-skill mutation or development-app live smoke is authorized by this coding mission. Read-only help/version and bounded session-free metadata probes are allowed. An unavailable native prerequisite is recorded honestly; do not install OpenCode or any tool automatically. Native macOS and Windows ConPTY passes require subsequent user-authorized testing.

## Authorization and handoff

The user initially asked for local development with Peer coding. The repository required the coordinator to commit this mission brief before launch; that setup commit was authorized as mission preparation. On 2026-10-12, after the crew's clean review and local handoff, the user authorized committing the reviewed implementation, pushing this branch to their fork and creating a **draft** pull request against upstream `main`. Fold this brief into the implementation commit. Keep the PR in draft, describe the unverified native/platform checks, and do not merge, release, modify issues/milestones, change Git identity, or delete branches/worktrees. Do not add co-author trailers.

If the user later authorizes a code commit, fold the brief into the implementation commit rather than leaving a brief-only history; verify explicit Git name/email before every commit/rewrite. Review must be clean before any code commit/push/PR.

`coder` sends `reviewer` the branch, changed files, implementation summary and exact selected check results. Reviewer stays read-only, reports concrete must-fix findings first, and returns a clean-review statement only after corrections. Post milestones, blocking questions and the final handoff in this mission's Runner channel so the human can see progress. Final handoff includes current native version, changed behavior/files, test commands/results, remaining platform/native limitations and confirmation that no PR was submitted. Stop coding sessions after the clean handoff; keep the worktree for the user.

Do not inspect sibling repositories or other projects. Official OpenCode documentation is authorized; another project's source repository needs the user's explicit address-specific authorization under the current project boundary rule. Messages to this mission's crew are authorized for implementation/review coordination; unrelated messages are not.
