# 723 — Grok Build runtime

Implement the Grok half of [#723](https://github.com/yicheng47/runner/issues/723). Jason asked on 2026-10-11 for a codex trio mission that ends in an open PR; a SuperGrok subscription is signed in on this Mac.

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-723-grok-build`, on branch `feat/723-grok-build`. The mission's directory is this worktree. Its tip is the spec and this brief, on top of `main` `bebabac5`. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## Read first

- `AGENTS.md`, including Worktrees, Test Scope and Crew Missions.
- **`docs/features/723-grok-build-runtime.md`**: the spec, probed against grok 1.0.50. It wins over this brief on any detail. Its Decisions and Verification are binding; rows marked *probe* are yours to verify.
- `docs/arch/runtime-integration.md`: the contract. Every P0 row applies.
- `docs/features/723-cursor-agent-runtime.md` and `crates/runner-daemon/src/runtimes/cursor_agent/`: the closest runtime (positional first turn, assigned key, `--trust`, `models` discovery). The commit that added it, `b815d359`, is your site list; follow its shapes.
- `runtimes/claude_code/` for the `--permission-mode` permissions and the hook event vocabulary; `runtimes/copilot/copilot_status.rs` and `runtimes/antigravity/agy_status.rs` for installing hooks and parsing reports; `runtimes/helpers.rs` (`hook_env`, `hook_report_command`, `config_home`) and `crates/runner-cli/src/hook.rs` for the reporter.
- `runtimes/codex/` for the TOML MCP client; `shell_path.rs` for captured variables; `runtime_status/models.rs` for discovery.
- grok's own guide at `~/.grok/docs/user-guide/`, especially `10-hooks.md`, `17-sessions.md`, `22-permissions-and-safety.md` and `26-config-reference.md`.

## Policy updates

Jason changed the default-on policy on 2026-10-11: enable Cursor and Grok by default; make TRAE CLI opt-in. Explicit saved enable/disable preferences retain precedence. After this change, Jason authorized merging the PR with no further QA. Working-tree review and automated/CI gates still apply; branch/worktree deletion and a nightly/release remain unauthorized.

## Deliverables

The spec's phases 1 to 5 on this branch:

1. **Adapter and launch**: the `Runtime::Grok` variant and identity row, catalog, `runtime_ui` row and icon, detection and override, `GROK_HOME` resolution, permissions and stripping, `--trust`, the positional first turn, model and effort argv, `-s` assigned key, `--resume`, the session probe, and every enumerating test and golden.
2. **Settings**: `grok models` discovery with per-model efforts from `models_cache.json`, native defaults, version, skill roots and Settings copy, and `README.md` with `README.zh-CN.md` together.
3. **Hooks on macOS and Linux**: the `$GROK_HOME/hooks/runner-status.json` installer, the reporter mapping, generation-checked rekey on `SessionStart`, and the status reducer wiring. Off on Windows.
4. **MCP**, then **fork** only if the probe proves it.
5. **A real grok first-turn terminal fixture** in the existing fixture corpus with its replay test, and **the validation record** `docs/tests/723-grok-build-runtime.md`: dated, with the CLI version, separating unit, CLI-probe and native-app evidence per platform.
6. **Subscription usage, added by Jason on 2026-10-11.** Probe `/usage` in a scratch cwd with a debug file, then implement one Week window through the existing usage refresh/cache/popover only if one read-only request returns weekly used and reset. Read the resolved home's access token without refreshing, rotating, writing or logging it; expired/401 is unavailable. If no such response exists, record the sources tried and leave usage off; never scrape the TUI or run it in a hidden PTY for usage refresh. This addition supersedes the original spec's subscription-usage exclusion.

When a probe contradicts the spec, correct the spec in the same branch, cite the evidence, and say so in the handoff. Also update the 723 Cursor spec's status line and its `docs/features/README.md` line, which still say Grok remains open.

Extend the existing expectation files in their current format. Grok's golden additions should be about the size of Cursor's (roughly 1,500 lines across the expectation JSON); add no new golden file other than the terminal fixture.

## Probe boundaries

Bounded live `grok` sessions on Jason's existing sign-in are authorized for the spec's *probe* rows: disposable directories under `/tmp`, tiny prompts, `--no-subagents`, and grok processes that are test subjects and never delegate work. Prefer a scratch `GROK_HOME` holding a symlink to `~/.grok/auth.json`; if the sign-in does not carry over, use the real `~/.grok` with a probe hook file named `runner-probe.json` and remove it afterwards. In the real `~/.grok`, never touch `orca-status.json`, `config.toml`, `auth.json` or Jason's sessions, and remove any `trusted_folders.toml` entries your `/tmp` probes added. Do not run `grok update`, `grok login` or `grok logout`, and do not change grok's update setting. Tests use temporary homes only. Do not edit `~/.claude`, `~/.codex`, `~/.cursor` or other agent configuration.

The coder and reviewer never run the dev app. Do not start, stop, restart or type into Jason's Runner apps, chats or missions.

## QA

After a clean review, the coder hands the working tree to `qa` with the changed behaviors. QA is authorized for one native pass of the spec's macOS row in the **development** app with computer use, following `docs/tests/full-smoke-test.md` for setup, candidate identity, isolation, evidence and cleanup. It covers a runtime-only grok chat, a role chat, a two-slot mission with a grok slot in an untrusted `/tmp` project that edits a file, runs a command and posts to the bus, interrupt, stop and resume, `/new` rekey, model and effort, and hook status. It creates and archives only its own test objects. If another dev app or a daemon owned by someone else is running, QA reports Blocked and does not stop it. Checks QA cannot reach go on Jason's smoke list.

## Validation

Run each, log to a file, and read the exit code from the command itself, never through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

Gate any import or helper that only `cfg(unix)` tests use with `#[cfg(unix)]`; Windows CI has failed on this repeatedly. Native Windows is unavailable; say what is unverified there.

## Review, verification and authorization

The coder owns implementation, probes, tests and fixes. The reviewer waits for an explicit Runner handoff, then reviews the whole working-tree diff against the spec and this brief, must-fix first with file:line pointers. It checks in particular:

- every `Runtime` match and enumerating test covers Grok, and the other runtimes' argv and goldens are unchanged;
- no path passes `--reasoning-effort` for a level the effective model does not list, or `--continue`;
- the hook installer writes only `runner-status.json`, never creates `$GROK_HOME`, registers no `PreToolUse`, sets every timeout, and is a no-op outside Runner;
- rekey cannot move to a subagent's session or accept a stale generation, and the session-end `Stop` never reads as a completed turn;
- every probe result the code relies on is in the validation record.

Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`, then QA as above. No extra agents, crews or subagents.

After clean review and QA, Jason authorizes rebasing all work into clean history on top of current `origin/main`, with this brief and the spec folded into a commit and subjects that name the change (for example `feat(runtime): add Grok Build CLI support`); pushing `feat/723-grok-build`; and opening a PR against `main`. The body says `Closes #723` when every macOS P0 row passed, otherwise `Refs #723` with the reason, and lists the summary, test evidence, probe results, QA matrix, the icon's source, and what waits for Jason (Windows ConPTY, anything QA could not reach), with no agent session links. Review or CI fixes after the push are folded into the commit they correct and pushed with `git push --force-with-lease`. Drive CI green on macOS and Windows. Jason subsequently authorized merging after the default-on policy follow-up, with no additional QA. Do not delete the branch or worktree, or cut a nightly or release.

Final Runner handoff to everyone: PR URL, CI result, what changed, probe results, checks with exit codes, spec corrections, the QA matrix, the reviewer's verdict, and Jason's remaining smoke list. Then all slots stand by.
