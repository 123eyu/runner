# #977 — Runtime knowledge behind the adapter (item 1)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. This is the second mission. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-runtime-adapter`, on branch `refactor/977-runtime-adapter`, based on `origin/main` at `3d860523`. The root checkout stays on `main`; this worktree uses its own `target/`. The first mission (items 2, 9 and 10, branch `refactor/977-small-dedup`) touches `runtimes/helpers.rs` and the three `*_trust.rs` files. Leave those alone here so the two PRs don't conflict.

Read item 1 of the issue body and the [2026-10-26 re-audit](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449). No behavior change is intended.

## What the issue's numbers miss

The issue's table counts `Runtime::<Variant>` matches including inline test modules, and most of them are tests. In production code, `ops/skills.rs` has 1, `runtime_status/versions.rs` 2, `pty_runtime.rs` 0, and `settings/agents.rs` 3. `RuntimeAdapter` in `crates/runner-daemon/src/runtimes/mod.rs` already carries skills, MCP, usage, launch args, the launch gate, trust, fork plans, resume and status hooks. What remains is concentrated in a few places (re-measured on `3d860523`):

- **`session/manager/spawn.rs`** (3,213 production lines, about 100 code lines naming a runtime): `claude_permission_arg`, `enter_claude_launch_gate`, `codex_fork_sessions_root`, `codex_capture_prompt_marker`, `start_antigravity_capture`, and runtime checks such as `Runtime::Cursor | Runtime::Grok | Runtime::OpenCode`.
- **`session/state/agent/mod.rs`**: the status reducer branches on `runtime == Runtime::Copilot / Codex / Pi / Antigravity / ClaudeCode` about 38 times (compaction, interruption, turn clearing, owners, startup batches).
- **`TerminalAdapter::take_codex_attention`** in `runtimes/mod.rs`: a runtime-named method on a shared trait. `#1052` just landed OSC 7501 program status, so check its current shape first.
- **Smaller sites:** `session/manager/mod.rs`, `session/state/mod.rs`, `usage.rs`, `ops/skills.rs`, `ops/session.rs`, `ops/mcp.rs`, `skills.rs` and `companion/actions.rs`.

## Deliverables

1. **Inventory before moving code.** Post an inventory and plan to `reviewer` before the bulk of the work, and let `reviewer` check the direction. The inventory covers every production site outside `runtimes/` that branches on a specific runtime. For each site, give the adapter method or catalog field it should move behind, or why it should stay. This is the cheap point to catch a wrong direction.
2. **`spawn.rs`.** Move the Claude permission argument, the Claude launch gate, the Codex fork session root and prompt markers, and the Antigravity capture into `runtimes/<name>/`, reached through `RuntimeAdapter` (or `TerminalAdapter` where it is terminal behavior). `spawn.rs` should dispatch through the adapter and stop naming runtimes. Extend existing adapter methods where they fit before adding new ones.
3. **Status reducer.** Replace the per-variant checks in `session/state/agent/mod.rs` with named behavior that the adapter declares, for example a small `Copy` struct of status semantics. The reducer then asks what a runtime does instead of which runtime it is. This is the riskiest part: inbox delivery and Needs-you depend on it. The existing reducer tests must pass without being edited. If a branch can't be expressed as a clear named behavior, leave it and list it.
4. **`take_codex_attention`.** Give it a runtime-neutral name and contract.
5. **Smaller sites.** Move the rest where the adapter or the runtime catalog already has, or can simply gain, the fact. App-side matches are mostly tests and the icon/label presentation in `runtime_ui.rs`; leave that presentation alone.

Size the commits so each one stands on its own: for example spawn, then the reducer, then the rest.

## Non-goals

- `claude_accounts` (daemon and app), which is a Claude-only feature by design.
- The shell lifecycle split, which is [#795](https://github.com/yicheng47/runner/issues/795) (0.21). Keep `Runtime::Shell` handling as it is, but don't make #795 harder.
- Item 2's trust files; models, usage or skills adapter shapes that the issue only flagged to check; new runtimes; and any behavior change.

## Read first

- `AGENTS.md` (Engineering Conventions, Test Scope, Crew Missions), `docs/arch/runtime-integration.md`.
- `crates/runner-daemon/src/runtimes/mod.rs` and one full adapter such as `runtimes/codex/`.
- `session/manager/spawn.rs`, `session/state/agent/mod.rs`, `session/pty_runtime.rs`.

Update `docs/arch/runtime-integration.md` where it describes what an adapter owns.

## Validation and handoff

Before running checks, list the changed behaviors and the check that covers each. Run the `runner-daemon` tests (all of them, since spawn and the reducer are shared), then `make clippy` and `make fmt`. `make verify` must pass before the PR. Reducer and spawn tests stay unedited unless a test only names a moved symbol, and the PR lists any test you touched and why. Add adapter-level tests where a moved fact gains a new home without coverage.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` reviews it and names any live checks under Test Scope, or says why none are needed. Live checks are left to Jason and listed in the PR as not run.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: implementation, tests, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977", and driving macOS, Windows and Linux CI green. Fold this brief into the first commit, fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the checks that were run, and any sites deliberately left behind, through Runner.
