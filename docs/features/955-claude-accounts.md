# 955 — Claude Code accounts

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

**Nothing changes until a second account exists.** With one account, launches get no `CLAUDE_CONFIG_DIR` and no new flags, resume and fork keep today's code path, the conversation check and fresh-start fallback are unchanged, usage reads only the main account, and no account chip or dropdown appears; the Accounts section shows that one row. Every new branch in the daemon and the app is gated on more than one account, and tests pin the one-account launch command and environment to today's.

Each row shows the name, a `main` badge or an `Active` badge, the email and plan, the folder, whether it uses the main account's settings, and the sign-in status in the label (Signed in, Not signed in, Signing in). Email, organization and plan come from `claude auth status` run with that folder.

- **Add account** asks for a name and fills the folder from it as `~/.claude-<name>` (shown as `Auto — ~/.claude-<name>`, like Executable); Browse picks another folder. A **Use Personal's settings** checkbox is on by default. **Add and log in** saves the account and starts sign-in.
- **Log in** and **Add and log in** run `claude auth login` with the account's `CLAUDE_CONFIG_DIR`. The browser opens Claude's sign-in page; the row shows Signing in with **Open sign-in page** and **Cancel**, then fills in the email and plan. Nobody types a command, and Runner stores no credentials.
- **Use** on a signed-in row makes it the active account.
- The **⋯ menu** offers Make main, Stop using Personal's settings (only on linked accounts), Rename, Reveal in Finder (Show in Explorer on Windows) and Remove. The main account and the active account cannot be removed; the disabled item says why. Removing deletes the links Runner created in the account's folder and forgets the account; the folder, its own files and its sign-in stay on disk.

### The main account's configuration

The main account owns the user-level configuration: global `CLAUDE.md`, `skills/`, `agents/`, `commands/`, plugins, `settings.json` and user-scope MCP servers. Another account uses it after **Use Personal's settings** on its row, or the checkbox in Add account. **Runner only adds links and never moves, overwrites or deletes a user's file.** It links `CLAUDE.md`, `settings.json`, `skills/`, `agents/`, `commands/` and `plugins/` (`plugins/` because `settings.json` enables plugins by name) only where the account's folder does not already have that item. An item the folder already has stays as it is, and the row says so, such as `own settings.json`. A new account, or a folder that only holds a login, therefore gets full sharing, while a folder with its own configuration keeps it. **Stop using Personal's settings** and Remove delete only links Runner created that point into the main account's folder. The confirmation explains this.

Each account keeps its own `.claude.json` (sign-in identity, onboarding, folder trust) and credentials, so those are never linked. MCP servers reach a linked account through `--mcp-config` at launch, built from the main account's `mcpServers` (in `~/.claude.json` for `~/.claude`, in `<folder>/.claude.json` for any other folder). Conversation history stays per folder.

Settings → Skills and Settings → MCP keep editing the main account's configuration, which reaches every linked account. When Make main moves the main account to another folder, those panes, model discovery and the update channel lookup follow it.

### The active account

One active account applies to every new Claude Code session: direct chats, role chats, crew slots, members added to a mission, and resumes. It is set from the usage popover's dropdown or **Use** on the Claude Code page; only signed-in accounts can be active. The main account launches exactly as today, without `CLAUDE_CONFIG_DIR`. Another account launches with its folder in `CLAUDE_CONFIG_DIR`, plus `--mcp-config` when it uses the main account's settings. Running sessions keep the account they started on.

A role or launch environment that already sets `CLAUDE_CONFIG_DIR` keeps it: it is an explicit pin, and the session shows that folder (as an account name when it matches one).

### Sessions

Each Claude Code session records the account it last ran on and the account folder its conversation started in. Once more than one account exists, Claude Code tabs show an account chip after the title, and the chat side panel shows **Account** (name and email) beside Runtime and a `config` row with the folder.

**Resume always continues on the active account**, including resume after a restart, a relaunch or daemon recovery. A paused chat from another account keeps the shipped Resume and Archive buttons; its text says where it ran and where it will continue. The resume rule:

- When the conversation file is in the active account's folder, which is every chat with one account, Runner resumes by session ID exactly as today.
- Otherwise Runner resumes by path: `claude --resume <origin folder>/projects/<encoded cwd>/<session-id>.jsonl`, built with the existing `claude_code_project_dir` encoding, plus `--permission-mode` with the session's last mode, because the path form does not restore it. The session then records the active account as where it runs; its origin folder does not change.
- The conversation check and the fresh-start fallback use that same file. Runner never starts a fresh conversation because the file lives under another account.
- Forking follows the same rule: `--resume <id> --fork-session` when the file is in the active account's folder, the path form otherwise.

The rule is one small function; it only diverges from today's behaviour when the conversation's folder differs from the active account's.

### Usage

The usage popover shows only the active account's Claude Code usage. The Claude Code header has an account dropdown listing the signed-in accounts with their weekly usage; picking one makes it active, and Manage accounts… opens `Agents › Claude Code`. The footer meter shows the active account's value. Accounts that are not signed in do not appear.

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

Still to prove before the code that depends on it: whether Claude Code keeps a symlinked `settings.json` when it writes it (if not, settings move to `--settings` at launch); that symlinked `skills/`, `agents/`, `commands/` and `plugins/` folders load; the Windows route (directory junctions, an `@~/.claude/CLAUDE.md` import, `--settings`); where a fork made with the path form writes its new transcript; and the per-account usage read.

## Implementation phases

The work ships in one pull request (#970), with one commit per phase, each revertable on its own with `git revert`, so the part every user sees is implemented and checked before any account code exists. No schema migration: accounts are a JSON value in the app-state store beside `runtime_overrides`, and a session's account and origin folder live in the existing versioned `launch_definition_json`; a session without them ran on the main account.

1. **Agents page redesign.** The Agents list and agent pages for every runtime, carrying today's settings and behaviour unchanged. Checked with `SET-AGENT-01` and `SET-AGENT-02`.
2. **Accounts.** The account list and its operations over the client protocol (add, rename, remove, make main, use, log in, cancel, status); launch environment for non-main accounts; the session's account and origin folder; the resume and fork rule and the conversation check; per-account usage. In the app, Claude Code's Accounts section with add, sign-in and the menu; the usage dropdown and footer; the tab chip, side panel rows and paused-chat text. Starts with the fork and usage probes.
3. **Sharing the main account's configuration.** Link and unlink, `--mcp-config`, the per-account button, the Add account checkbox and the confirmation. Starts with the symlink and Windows probes.

Each phase updates its guide pages and tests.

Guide pages affected: `docs/guide/reference/supported-agents.md` (the Agents page), `docs/guide/concepts/sessions.md` (resume continues on the active account), `docs/guide/guides/mcp-and-skills.md` (the panes edit the main account), and a new `docs/guide/guides/claude-accounts.md` covering setup, the active and main accounts and the settings and history separation, listed in `docs/guide/README.md`.

## Verification

Automated: a one-account setup produces today's launch command, environment, resume and fork arguments exactly; the folder name derivation and Keychain item name; launch arguments and environment for another account; the resume and fork rule (session ID versus path with `--permission-mode`) and the conversation check; link and unlink, including leaving existing files untouched and deleting only Runner's own links; account operations and the active-account rules; usage per account.

Live, in the development app with two real accounts (`~/.claude` and `~/.claude-work`): add an account through browser sign-in; link it to the main account and check a global skill and `CLAUDE.md` load; switch the active account from the usage dropdown and start a chat on it; check the tab chip, side panel and footer; pause a chat, switch accounts and resume it, confirming the conversation continues in the same file with its permission mode; and a single-account pass showing resume unchanged. On Windows, a folder with spaces, the credentials file and the junction route.

Selected regression cases: `SET-AGENT-01` and `SET-AGENT-02` (the Agents page moves into rows and pages), `SET-USAGE-02` (the usage popover), `SET-SKILL-02` and `SET-MCP-02` (the panes edit the main account), `DAE-STOP-01` and `DAE-CRASH-01` (resume after relaunch and recovery).
