# 955 — Claude Code accounts

Implement [#955](https://github.com/yicheng47/runner/issues/955), the whole feature. Jason requested a codex trio mission on 2026-10-20 and chose one pull request, the existing draft [#970](https://github.com/yicheng47/runner/pull/970), with one commit per spec phase instead of the spec's three pull requests. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-955-claude-accounts`, on the existing branch `feat/955-claude-accounts`: the spec commit `53eac0d3` and this brief, on top of `origin/main` `8e523b5e`. The root checkout stays on main. Do not create another branch or worktree, and do not share a Cargo target directory. Other worktrees (`.worktrees/fix-active-sidebar-polish`, `.worktrees/nightly-remote`) are other machines' checkouts.

## Read first

- `AGENTS.md`, including Worktrees, Test Scope and Crew Missions.
- **`docs/features/955-claude-accounts.md`**. It wins over issue #955 and over this brief on any detail; Verified facts, the phase list and Verification are binding.
- **The design**, exported as PNGs into this worktree's ignored `target/design-ref/`, named by frame: `YjEPm` Agents as short rows; `QqpJd` the Claude Code page with Accounts; `zRWFp` adding an account; `G2d4rL` signing in; `mVsNW` the Use Personal's settings confirmation; `IsGR4` the ⋯ menu; `COzhP` and `UQd0O` the usage popover and its open dropdown; `hVeD0` a chat on Personal while Work is active; `Dx1PQ` a paused chat from another account; `VLPNt` the design decisions. Match their layout and hierarchy with the existing theme tokens; the sample data is illustrative. Do not delete them or open `.pen` files.
- Phase 1, under `crates/runner-app/src/surfaces`: `settings/agents.rs` (`render_runtime_row` ~771, `render_not_installed_row` ~694, the override commit/browse/reset handlers ~434–585, `render` ~1033), `settings_page.rs` and `settings/mod.rs` for pane routing, `agent_update.rs` (#533 updates).
- Phase 2, daemon, under `crates/runner-daemon/src`: `runtimes/claude_code/mod.rs` (`claude_settings_args` 41, `claude_code_project_dir` 114, `claude_code_conversation_exists` 145, `launch_env` 189, `resume_plan` 225, `fork_plan` 262, `launch_args` 283, `npm_dist_tag` and the discovery `config_home` ~341, `USAGE` 368); `runtimes/claude_code/usage.rs` (the Keychain read ~73–175 and `read_claude_credentials_file`); `usage.rs`; `db/app_state.rs` (`runtime_overrides` 65, the key accounts sit beside); `repo/session.rs` (`LaunchDefinitionRow`, version 1); `session/manager/spawn.rs` (`enter_claude_launch_gate` 402, `resume` 2316, `resume_with_fresh_fallback` 2384); the client protocol in `crates/runner-core/src/protocol/` (`runtime.rs`, `usage.rs`, `session.rs`).
- Phase 2, app, under `crates/runner-app/src`: `surfaces/app_shell.rs` (`usage_popover_runtimes` 240, `usage_popover_panel` 501, `render_usage_popover` 553), `surfaces/panes.rs` (`render_chat_side_panel` 1039 and the tab title), `i18n.rs` with `crates/runner-app/i18n/{en,zh-hans}.toml`.
- Phase 3: `crates/runner-daemon/src/runtimes/claude_code/skills.rs`, the `MCP` config at `claude_code/mod.rs:378`, and the Skills and MCP panes in `crates/runner-app/src/surfaces/settings/{skills,mcp}.rs`.

## Facts found after the spec was written

Checked on 2026-10-20 against Claude Code 2.1.304. Add both to the spec in the phase 2 commit:

1. **Setting `CLAUDE_CONFIG_DIR=~/.claude` explicitly is not the main account.** With the variable set, Claude Code reads `<folder>/.claude.json` (not `~/.claude.json`) and the suffixed Keychain item, so `~/.claude` set explicitly runs signed out while the same folder with the variable unset is signed in. So the main account is "variable unset", never "folder equals `~/.claude`". A role or environment pin of `~/.claude` stays an explicit pin, shown as its folder, and is never shown as the main account.
2. **`claude auth status` can report `subscriptionType: null` for a signed-in account** (the test work account does). Signed in means `loggedIn: true`; a row with no plan shows the email without a plan, never `null`, and is still signed in.

## Deliverables

Build the spec's phases in order, one commit each. Every commit builds and passes its tests on its own.

1. **Phase 1, Agents page redesign.** Rows and agent pages for every runtime, carrying today's settings and behaviour unchanged (detection states, captions, executable override, defaults, Set as default, enable, #533 Update). No account code, no behaviour change. Update `docs/guide/reference/supported-agents.md`. Fold the spec and this brief into this commit, and change the spec's "three pull requests" wording to this one PR with a commit per phase, each revertable with `git revert`.
2. **Phase 2, Accounts.** Everything the spec's phase 2 lists, daemon and app. Start with the fork-placement and per-account usage probes (below) and record the results in the spec. Keep the one-account path identical to today: launch argv and environment, resume, fork, the conversation check, the fresh-start fallback and usage, pinned by tests. Gate every new branch on more than one account. The resume and fork rule is one small function. A session's account and origin folder go in `launch_definition_json` without a schema migration; a session without them ran on the main account. Decide whether `enter_claude_launch_gate`, which serialises fresh Claude Code launches against the OAuth refresh race, should be per account, and say why in the handoff. New UI strings go through `i18n` in English and Simplified Chinese. Update `docs/guide/concepts/sessions.md` and add `docs/guide/guides/claude-accounts.md`, listed in `docs/guide/README.md`.
3. **Phase 3, sharing the main account's configuration.** Start with the symlink probes. Link and unlink exactly as the spec says: add a link only where the account's folder has no item of that name, and delete only links Runner created that point into the main account's folder, checking the target before deleting. `--mcp-config` from the main account's `mcpServers`. If `settings.json` must move to `--settings`, merge it into the single `--settings` JSON `claude_settings_args` already passes; never pass the flag twice. The Windows route (junctions, `@` import, `--settings`) is implemented and unit-tested as far as CI allows. Update `docs/guide/guides/mcp-and-skills.md`.
4. **Tests**: every automated item in the spec's Verification, beside the existing tests of each file. Gate imports and helpers used only by `cfg(unix)` tests with `#[cfg(unix)]`; Windows CI fails on unused ones.
5. **A test record** at `docs/tests/955-claude-accounts.md`: probe results, commands and exit codes, QA's matrix per phase, and what stays unverified.

If implementation forces a deviation from the spec, update the spec in the same commit and say why in the handoff. Out of scope: everything in the spec's Out of scope, `.pen` files, README screenshots.

## Probe boundaries

Two accounts are signed in on this Mac: the main account (`~/.claude`, `CLAUDE_CONFIG_DIR` unset) and `~/.claude-work`, which Jason created on 2026-10-20 as this feature's test account. Bounded `claude` probes on either account are authorized: a fresh folder under `/tmp` per probe, `--model haiku`, tiny prompts. Never print, log or commit a token, credential or account email. A Keychain read Runner has not made before can raise a macOS prompt on Jason's screen: ask Jason through Runner before the first one.

Symlink probes (phase 3) use throwaway folders under `/tmp` as both the main and the linked folder. Nothing in the mission writes into `~/.claude` or `~/.claude.json`, and nothing links `~/.claude-work` until a probe has shown that Claude Code does not write through a linked `settings.json` into its target. QA's link check is the only change to `~/.claude-work`: list the folder before and after, record both, and leave it as found.

## Live checks for QA

QA runs after each phase passes review and before the next phase starts, against the exact commit the coder hands over, following Test Scope in `AGENTS.md` and the spec's Verification. The coder does not edit the worktree while QA is building or testing.

- **Phase 1**: `SET-AGENT-01` and `SET-AGENT-02`; every row, installed or not, opens its page and the breadcrumb returns; the Update button and the executable override work from the page.
- **Phase 2**: add `~/.claude-work` with Browse (it is already signed in); switch the active account from the usage dropdown and start a chat on it; the tab chip, the side panel's Account and `config` rows and the footer meter; pause a chat started on the main account, switch, resume it, and confirm it appends to the same transcript with its permission mode; fork across accounts; remove the work account and confirm one-account resume is unchanged; `SET-USAGE-02`, `DAE-STOP-01`, `DAE-CRASH-01`. **Add and log in** needs a person at the browser: ask Jason through Runner to complete the sign-in for a fresh folder under `/tmp`, or record it as Blocked.
- **Phase 3**: link `~/.claude-work` to the main account in the development app, confirm a global skill and `CLAUDE.md` load in a work-account chat and the main account's MCP servers are listed; Stop using Personal's settings removes only Runner's links; `SET-SKILL-02` and `SET-MCP-02`. Hash `~/.claude/settings.json` and `~/.claude/CLAUDE.md` before and after and confirm they are unchanged.

Native Windows is unavailable to the crew: list the spec's Windows checks (a folder with spaces, the credentials file, the junction route) as pending for Jason.

## Review, verification and authorization

The coder owns implementation, probes, tests and fixes. The reviewer waits for an explicit Runner handoff per phase, then reviews that phase's whole diff against the spec, the PNGs and this brief, must-fix findings first with file:line pointers. It checks in particular:

- that a one-account setup's launch argv, environment, resume, fork and usage are byte-for-byte today's, and every new branch is gated on more than one account;
- the resume and fork rule, and that Runner never starts a fresh conversation because the file lives under another account;
- that the main account is the unset variable, the Keychain item names, and that no credential is stored, logged or copied;
- that link and unlink never move, overwrite or delete a user's file, and `--settings` is passed once;
- phase 1's absence of behaviour change, and the zh-Hans strings.

Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`, then hand the commit to QA, until QA posts `NO REMAINING MUST-FIX QA ISSUES` or names its Blocked checks. QA findings go to the coder; fixes go back through review. No extra agents, crews or subagents.

Run each of these per phase and report its exit code: `cargo test --locked -p runner-app --profile ci --no-fail-fast`, `cargo test --locked -p runner-daemon --profile ci --no-fail-fast` and `-p runner-core` when they change, `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, the same with `--features updater`, `cargo fmt --all --check`, and `git diff --check`. Log each to a file and read `$?` from the command itself; never pipe a gate through `tail` or `grep`.

After each phase's clean review and QA, Jason authorizes rebasing the branch into the three-commit shape on top of current `origin/main` (never merge main in), imperative subjects naming the change (for example `feat(ui): give each agent its own settings page`, `feat(session): run Claude Code on the active account`, `feat(session): share the main account's configuration`), no co-author trailers, and pushing `feat/955-claude-accounts` with `git push --force-with-lease`, which updates #970. Watch CI with `gh pr checks 970 --watch` on macOS and Windows; fold any fix into the commit it corrects (`git commit --fixup` with an autosquash rebase), have the reviewer check it, and push again. The next phase may start while CI runs.

When all three phases are pushed with CI green, rewrite #970's body (`Closes #955`, a summary per commit, test evidence, the manual checks for Jason including the Windows ones, what is unverified, no agent session links) and mark it ready with `gh pr ready 970`. **Do not merge**, delete the branch or worktree, or cut a nightly or release.

The final handoff goes to everyone through Runner: the PR URL and CI result, the three commits, checks with exit codes, probe results, spec changes, QA's matrix, what is untested, and the reviewer's verdict. Then all slots stand by.
