# 955 — Claude Code accounts

Shipped in [0.17.0](https://github.com/yicheng47/runner/releases/tag/v0.17.0) through [#970](https://github.com/yicheng47/runner/pull/970), 2026-10-21. Retained for the native-account identity, transcript-origin, configuration-ownership and Windows compatibility decisions. Scope and probe results below are historical; current instructions live in the [account guide](../../guide/guides/claude-accounts.md), with evidence in the [validation record](../../tests/archive/955-claude-accounts.md).

Tracking issue: [#955](https://github.com/yicheng47/runner/issues/955).

## Motivation

People with more than one Claude subscription want to use all of them from Runner: keep one setup, and when one account nears its limit, move work to another. Claude Code supports this officially by giving each account its own configuration directory (`CLAUDE_CONFIG_DIR`); Runner should make that a visible, one-click choice instead of an environment variable typed into a role.

The design changed the issue's original scope after review with Jason on 2026-10-20. There is no per-chat, per-role or per-crew-slot account choice. One **active account** applies to every new Claude Code session, one **main account** owns the configuration every account uses, and a chat can continue on another account. This spec supersedes the issue body where they differ.

## Design

The signed-off Pencil source is `runner/desktop/specs/955-claude-accounts.pen` in the design repo (`349e8fe`). Frames:

- `YjEPm` Settings → Agents as short rows; `QqpJd` the Claude Code agent page with its Accounts section; `zRWFp` adding an account; `G2d4rL` signing in a new account; `mVsNW` the Use Personal's settings confirmation; `IsGR4` an account's ⋯ menu.
- `COzhP` the usage popover with the active account and its dropdown; `UQd0O` the dropdown open.
- `hVeD0` a chat still running on Personal while Work is active; `Dx1PQ` a paused chat from another account.
- `VLPNt` the design decisions note.

## Scope

### Settings → Agents

The Agents page becomes two lists, Installed and Not installed, with one 64 px row per agent: the provider mark on a tile, the name and Default badge, then version · executable · a short detail, an Update button when one is available, the enable toggle and a chevron. Claude Code's detail reads `Work active · 4 accounts` once more than one account exists. Every row, installed or not, opens that agent's page, `Agents › <agent>`.

An agent page holds what the card holds today, plus sections only that agent has: a breadcrumb back to Agents; the title with Set as default and the enable toggle; the version and detection line; **Executable** (Auto or a chosen path, Browse, Reset when overridden); **Defaults** (model and effort, read from the agent's own settings, read-only as today); and for Claude Code, **Accounts**. A not-installed agent's page holds its description, install link and Browse. The existing detection states, captions and update behaviour (#533) keep their meaning; only their place changes.

### Accounts

An account is a name and a configuration folder. The existing `~/.claude` is the first account and the **main account**, with no migration.

**Nothing changes until a second account has been added.** In an untouched one-account setup, launches get no `CLAUDE_CONFIG_DIR` and no new flags, resume and fork keep today's code path, the conversation check and fresh-start fallback are unchanged, usage reads only the main account, and no account chip or dropdown appears; the Accounts section shows that one row. New account presentation and selection branches are gated on more than one account; existing chats with recorded account metadata retain their origin-aware resume/fork path after another account is removed (Jason’s clarification, 2026-10-20), and tests pin the one-account launch command and environment to today's.

Each row shows the name, an independent `main` badge and `Active` badge (both when the main account is active), the email and plan, the folder, whether it uses the main account's settings, and the sign-in status in the label (Signed in, Not signed in, Signing in). Email, organization and plan come from `claude auth status` run with that folder.

- **Add account** asks for a name and fills the folder from it as `~/.claude-<name>` (shown as `Auto — ~/.claude-<name>`, like Executable); Browse picks another folder. A **Use Personal's settings** checkbox is on by default. **Add and log in** saves the account and starts sign-in.
- **Log in** and **Add and log in** run `claude auth login` with the account's `CLAUDE_CONFIG_DIR`. The browser opens Claude's sign-in page; the row shows Signing in with **Open sign-in page** and **Cancel**, then fills in the email and plan. Nobody types a command, and Runner stores no credentials.
- After sign-in succeeds, including adding an already signed-in folder, Runner sets only `hasCompletedOnboarding: true` in that added account’s `<folder>/.claude.json`, preserving all unrelated bytes and skipping the write when already true. It never writes the original native account’s folder or `~/.claude.json`. The throwaway-folder probe showed that `lastOnboardingVersion` is unnecessary (Jason’s decision and probe, 2026-10-20).
- **Use** on a signed-in row makes it the active account.
- The **⋯ menu** offers Make main, Stop using Personal's settings (only on linked accounts), Rename, Reveal in Finder (Show in Explorer on Windows) and Remove. The main account, the active account and the original native account cannot be removed; the disabled Remove item explains why on hover; menu rows have no inline Remove explanation. Account menus are right-aligned with consistent single-line labels, truncating unusually long account names instead of wrapping. Removing deletes the links Runner created in the account's folder and forgets the account; the folder, its own files and its sign-in stay on disk.

### The main account's configuration

The main account owns the user-level configuration: global `CLAUDE.md`, skills, agents, commands, enabled installed plugins, settings and user-scope MCP servers. Another account uses it after **Use Personal's settings** on its row, or the checkbox in Add account. **Runner only adds links and never moves, overwrites or deletes a user's file.** `CLAUDE.md`, `agents/` and `commands/` link only where the destination item is absent; Windows uses a memory import with escaped spaces and directory junctions. The account's `skills/` remains a real directory, created if absent. Runner links each missing named skill directory from main into it, skipping `synced/` and dot-entries, and leaves same-named account skills unchanged. Windows uses one junction per skill. A user-owned skill-parent symlink/junction stays untouched and receives no new entries. New main skill entries are reconciled before shared launches, with ownership records persisted even after a partial failure. Existing Runner-owned parent links from an earlier candidate are removed only after exact-target verification before adopting this layout.

`plugins/` is never linked: Claude Code creates account-specific installed/synced/cache state there on first use, so linking the parent would write another account's state into main. For each plugin enabled in main's settings and installed there at user scope with an existing install directory, Runner adds one `--plugin-dir <installPath>` at launch. Disabled, missing and project-only installs are excluded. Plugin flags are omitted from launch definitions and their values are hidden from command summaries. The account row reports shared skill/plugin counts and retained local skill/plugin state. The 2.1.304 probe loaded the main-installed plugin skill and its exact reply with no changes to the fake main plugin tree; a control without the flag did not load it.

`settings.json` is never linked: the write-through probe showed that Claude can change the main file through that link. When the account has no settings file, Runner merges main settings into its existing single `--settings` JSON argument, preserving main hooks alongside Runner hooks. Existing account settings and explicit role `--settings` take precedence over that shared fallback. Existing items stay as they are, and the row describes local exceptions such as `own settings.json`. **Stop using Personal's settings** and Remove delete only verified Runner-owned links, including individual skill entries; the real skill directory and its own/synced items remain. The confirmation explains this. Make main removes the promoted account's managed links and retargets other linked accounts, preserving each account's own entries and local plugin state. The new owner's own configuration becomes authoritative. If removing or retargeting links fails, the previous main stays selected and Runner retains a pending transition with its ownership records. Settings shows Retry, which also works after a reload. Until it finishes, participating accounts cannot launch and other sharing mutations are blocked; main-owned panes and discovery still use the previous owner.

Each account keeps its own `.claude.json` (sign-in identity, onboarding, folder trust) and credentials, so those are never linked. MCP servers reach a linked account through `--mcp-config` at launch, built from the main account's `mcpServers` (in `~/.claude.json` for `~/.claude`, in `<folder>/.claude.json` for any other folder). Conversation history stays per folder.

Settings → Skills and Settings → MCP keep editing the main account's configuration, which reaches every linked account. When Make main moves the main account to another folder, those panes, model discovery and the update channel lookup follow it. Sign-in targets stay fixed: Make main changes only the configuration owner, never the native account’s unset environment or an added account’s explicit folder (Jason’s clarification, 2026-10-20). The original native account remains represented and cannot be removed, so returning to one account restores the unchanged native launch path.

### The active account

One active account applies to every new Claude Code session: direct chats, role chats, crew slots, members added to a mission, and resumes. It is set from the usage popover's dropdown or **Use** on the Claude Code page; only signed-in accounts can be active. The original native account always launches without `CLAUDE_CONFIG_DIR`. Every added account always launches with its folder in `CLAUDE_CONFIG_DIR`, plus launch-time `--mcp-config` and any enabled installed `--plugin-dir` entries when it uses the main account's settings. Running sessions keep the account they started on.

A role or launch environment that already sets `CLAUDE_CONFIG_DIR` keeps it: it is an explicit pin, and the session shows that folder (as an account name when it matches one).

### Sessions

Each Claude Code session records the account it last ran on and the account folder its conversation started in. Once more than one account exists, Claude Code tabs show an account chip after the title, and the chat side panel shows **Account** (name and email) beside Runtime and a `config` row with the folder.

**Resume always continues on the active account**, including resume after a restart, a relaunch or daemon recovery. A paused chat from another account keeps the shipped Resume and Archive buttons; its text says where it ran and where it will continue. The resume rule:

- When the conversation file is in the active account's folder, which is every chat with one account, Runner resumes by session ID exactly as today.
- Otherwise Runner resumes by path: `claude --resume <origin folder>/projects/<encoded cwd>/<session-id>.jsonl`, built with the existing `claude_code_project_dir` encoding, plus `--permission-mode` with the session's last mode, because the path form does not restore it. The session then records the active account as where it runs; its origin folder does not change.
- If Claude starts a new conversation in that process, such as with `/clear`, the new ID and the running account's transcript origin are persisted together. An unchanged ID keeps its existing origin, and selecting an existing transcript in the recorded origin preserves that location. Changing Runner's active selection does not change a running process's account. Regression coverage combines `RT-LIFE-08` and `RT-LIFE-09` with cross-account resume, reload and fork.
- The conversation check and the fresh-start fallback use that same file. Runner never starts a fresh conversation because the file lives under another account.
- Forking follows the same rule: `--resume <id> --fork-session` when the file is in the active account's folder, the path form otherwise.

The rule is one small function; it only diverges from today's behaviour when the conversation's folder differs from the active account's.

### Usage

The usage popover shows only the active account's Claude Code usage. The Claude Code header has a compact text trigger with a small chevron and a single-line account menu, with a leading check and trailing weekly usage (Jason’s UI revision, 2026-10-20); it lists signed-in accounts and picking one makes it active, and Manage accounts… opens `Agents › Claude Code`. The footer meter shows the active account's value. Accounts that are not signed in do not appear.

The token comes from the account's own credential: on macOS the Keychain item `Claude Code-credentials` for `~/.claude` and `Claude Code-credentials-<first 8 hex of SHA-256 of the folder path>` for any other folder; on Linux and Windows `<folder>/.credentials.json`. Keychain prompts, denial and unavailability keep #706's behaviour per account.

### Out of scope

Choosing an account per chat, role or crew slot; accounts for other runtimes (the agent page leaves room, and Codex has `CODEX_HOME`); automatic rotation when an account hits its limit; switching a running session without stopping it; Claude Console sign-ins without an API key, which Claude Code stores outside the folder; a `runner` CLI for accounts.

## Verified facts

Checked on 2026-10-20 against Claude Code 2.1.304 and its documentation:

- Separate `CLAUDE_CONFIG_DIR` folders are the documented way to use several accounts; each holds its own settings, history and login, and the macOS Keychain entry is keyed to the folder ([authentication](https://code.claude.com/docs/en/authentication#log-in-with-multiple-accounts)).
- `claude --resume <transcript-path>` is documented ([sessions](https://code.claude.com/docs/en/sessions)); `--resume <id>` searches only the projects under the current config folder. Live test with two accounts: `--resume <id>` on the second account reported `No conversation found`; `--resume <path>` recalled the first account's turn, kept the session ID and appended to the original file.
- The docs list permission-mode restore for `--continue`, `--resume <id>` and `--resume <name>`, not the path form.
- Thinking blocks from Claude Sonnet 5.5 and Haiku 5.5 work only in the account that produced them; the API drops them silently in another account ([preserved thinking](https://platform.claude.com/docs/en/build-with-claude/preserved-thinking)). Other models' thinking carries over. The first turn after a switch has no warm prompt cache.
- `claude auth login` and `claude auth status` (JSON: `loggedIn`, `email`, `orgName`, `subscriptionType`, `configDirectory`) work per folder.
- A second account's Keychain item is `Claude Code-credentials-a6d23975` for `/Users/jason/.claude-work`, the first 8 hex of SHA-256 of that path.
- Setting `CLAUDE_CONFIG_DIR=~/.claude` explicitly uses `<folder>/.claude.json` and the suffixed Keychain item, unlike the native sign-in with the variable unset. Explicit pins remain explicit even when the folder path equals `~/.claude` (verified in the mission brief on 2026-10-20).
- `claude auth status` may return `subscriptionType: null` while `loggedIn: true`; sign-in eligibility follows `loggedIn`, and an absent plan is omitted from the row (verified in the mission brief on 2026-10-20).
- Phase 2 fork-placement probe (2026-10-20, Claude Code 2.1.304, Haiku): a `--resume <path> --fork-session --session-id <new-id>` fork from an owned transcript copied into a separate `/tmp` origin configuration wrote the new transcript under the active work account's `projects/` directory. It kept the assigned new ID and recalled the source marker. Both the copied origin and original work transcript stayed unchanged. This probe used the existing work sign-in and a fresh `/tmp` working directory; two-account native resume/fork remains a separate QA check.

- Phase 2 usage probe (2026-10-20, explicitly authorized before both Keychain reads): native and work credentials each returned HTTP 200 with valid five-hour and weekly windows. Credentials stayed in memory; the retained report contains only service names, response codes and numeric usage.

Phase 2 onboarding probe (2026-10-20, Claude Code 2.1.304): isolated HOME/config folders, dummy API key, no submitted prompt and OS-denied home writes. `hasCompletedOnboarding: true` alone skipped the first-run connectivity/theme flow and reached project trust; `lastOnboardingVersion` alone did not. Native configuration hashes were unchanged in the OS-sandboxed run. A path-resume comparison reproduced the same `ECONNRESET` before onboarding when the flag was absent and reached trust when it was present; a bounded real Work headless path-resume/fork returned OK with a new ID and unchanged owned source. This identifies the startup gate as the observed failure point; native resume/append/permission checks remain QA work.

Phase 3 opening probes (2026-10-20, Claude Code 2.1.304): a throwaway linked `settings.json` remained a symlink after `/theme`, but Claude wrote the selected theme through it into the fake main file. Sharing therefore uses the specified single-argument settings fallback on every platform and never creates a settings link. Two isolated Haiku runs loaded symlinked global memory, skills, commands, agent definitions and a locally registered plugin skill. The initial incomplete plugin fixture failed to load; registering its local marketplace fixed the fixture, and both subsequent runs passed. No agent was invoked (`--tools ""`). A second run loaded the Windows-style `@` import with escaped spaces on macOS; import syntax follows the [memory documentation](https://code.claude.com/docs/en/memory). Probe HOME/config roots were under `/tmp`, real home writes were denied by the OS sandbox, and credentials remained in memory. Windows directory junctions are implemented with the privilege-free junction API and covered by Windows-only tests, including spaces, missing targets and retargeted links. Native Windows live verification remains pending for Jason.

The 2026-10-21 design correction (`01M5D13JM93RNYYTXTPQNMTB4S`) supersedes the original whole-folder skill/plugin route. Live Work had its own `skills/synced`, `.trash` and plugin metadata/cache directories, so absent-only parent linking did not share main's skills and would otherwise allow state write-through. The isolated `--plugin-dir` control/positive comparison is retained in `target/probes/955-phase3/plugin-dir-result.json`; main plugin file contents and mtimes stayed unchanged. The first marketplace fixture used an invalid absolute source, and a later control also loaded through injected marketplace metadata; both attempts are retained separately, with the final control passing only `enabledPlugins` to remove that second loading path.

## Implementation phases

The work ships in one pull request (#970), with one commit per phase, each revertable on its own with `git revert`, so the part every user sees is implemented and checked before any account code exists. No schema migration: accounts are a JSON value in the app-state store beside `runtime_overrides`, and a session's account and origin folder live in the existing versioned `launch_definition_json`; a session without them ran on the original native account.

1. **Agents page redesign.** The Agents list and agent pages for every runtime, carrying today's settings and behaviour unchanged. Checked with `SET-AGENT-01` and `SET-AGENT-02`.
2. **Accounts.** The account list and its operations over the client protocol (add, rename, remove, make main, use, log in, cancel, status); launch environment for added accounts; the session's account and origin folder; the resume and fork rule and the conversation check; per-account usage. In the app, Claude Code's Accounts section with add, sign-in and the menu; the usage dropdown and footer; the tab chip, side panel rows and paused-chat text. Starts with the fork and usage probes.
3. **Sharing the main account's configuration.** Exact-target root and individual skill links, launch-time settings/MCP/plugins, the per-account button, Add checkbox, row sharing details and confirmation. Starts with the symlink/Windows probes and the later isolated `--plugin-dir` probe.

Each phase updates its guide pages and tests.

Guide pages affected: `docs/guide/reference/supported-agents.md` (the Agents page), `docs/guide/concepts/sessions.md` (resume continues on the active account), `docs/guide/guides/mcp-and-skills.md` (the panes edit the main account), and a new `docs/guide/guides/claude-accounts.md` covering setup, the active and main accounts and the settings and history separation, listed in `docs/guide/README.md`.

## Verification

Automated: a one-account setup produces today's launch command, environment, resume and fork arguments exactly; the folder name derivation and Keychain item name; launch arguments and environment for another account; the resume and fork rule (session ID versus path with `--permission-mode`) and the conversation check; link and unlink, including leaving existing files untouched and deleting only Runner's own links; account operations and the active-account rules; usage per account; per-skill conflict/synced/dot and parent-link protection, owned-entry cleanup/reload/retargeting and launch reconciliation; enabled user-installed plugin selection and flags excluded from definitions/summary values; single-line menu geometry and simultaneous main/Active badges.

Live, in the development app with two real accounts (`~/.claude` and `~/.claude-work`): add an account through browser sign-in; link it to the main account and check a global skill, `CLAUDE.md` and an enabled installed main plugin load while native hashes and account synced/plugin state stay unchanged; verify owned-entry cleanup and retargeting; switch the active account from the usage dropdown and start a chat on it; check the tab chip, side panel and footer; pause a chat, switch accounts and resume it, confirming the conversation continues in the same file with its permission mode; and a single-account pass showing resume unchanged. On Windows, a folder with spaces, the credentials file and the junction route.

Selected regression cases: `SET-AGENT-01` and `SET-AGENT-02` (the Agents page moves into rows and pages), `SET-USAGE-02` (the usage popover), `SET-SKILL-02` and `SET-MCP-02` (the panes edit the main account), `DAE-STOP-01` and `DAE-CRASH-01` (resume after relaunch and recovery).
