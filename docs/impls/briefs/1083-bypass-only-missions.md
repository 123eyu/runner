# #1083 — Make Bypass the only mission permission mode

Jason requested a `codex duo` mission for [#1083](https://github.com/yicheng47/runner/issues/1083) on 2026-10-30. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-1083-bypass-only-missions` on `feat/1083-bypass-only-missions`, based on `origin/main` at `12b25f0c7b2abe03f732a007037579d868598d6a`. The root checkout stays on `main`. Use this worktree's own `target/`; do not set a shared `CARGO_TARGET_DIR`.

## Problem

#527 added an app-wide mission permission mode (Bypass, Auto, Role default); #749 removed its Settings control on 2026-09-29. Nothing in the app, CLI or daemon selects another mode now: `SessionManager` starts at the default, Bypass, and only tests call `set_mission_permission_mode`. The unreachable modes keep three paths in each runtime's mission arguments, the session manager's state and the permission tests where one runs. Making Bypass the only mode also states the contract #1012 relies on: a Codex mission member runs with `--sandbox danger-full-access`.

The issue is the spec; read it in full before starting. Where this brief and the issue disagree, the issue wins.

## Read first

- `crates/runner-core/src/protocol/permissions.rs` — `MissionPermissionMode` (remove) and `PermissionMode` (keep).
- `crates/runner-core/src/protocol/runtime_metadata.rs` — `RuntimeMetadata::mission_args` / `apply_mission` and the `mission_bypass` override.
- `crates/runner-daemon/src/session/manager/mod.rs` — the `mission_permission_mode` field, setter and getter; callers in `mission_spawn.rs` and `resume.rs`, and the comments in `fork.rs` and `direct_spawn.rs` that name the type.
- `crates/runner-daemon/src/ops/mission.rs` — the `permission_mode` parameter threaded into mission start and the `mission_start` payload.
- `crates/runner-daemon/src/router/runtime.rs` and `crates/runner-daemon/src/runtimes/mod.rs` — re-export and import.
- `crates/runner-app/src/surfaces/mission_workspace/mod.rs` — `mission_permission_mode_label` reads the payload string; keep it and its tests unchanged.

## Deliverables

1. `MissionPermissionMode` is gone. Mission spawns and resumes always apply the runtime's Bypass arguments (`mission_bypass` when set, else `mode_args(PermissionMode::Bypass)`). `mission_args` returns that one argument list and `apply_mission` takes no mode; keep them as small as the single case allows.
2. The session manager's `mission_permission_mode` state, `set_mission_permission_mode` and `mission_permission_mode()` are removed, along with the `permission_mode` parameter threaded through `ops/mission.rs` and its call sites.
3. The `mission_start` payload still records `"permission_mode": "bypass"`, and the mission rail still labels missions recorded under `auto` and `role-default`/`runner-default` as before.
4. Tests: delete the Auto and Role default cases; reduce the tests that loop over every mode (`session/manager/tests/permissions.rs`, `tests/runtime_override.rs`, `tests/golden.rs`, `runtimes/tests.rs`, `runtimes/test_support.rs` and the per-runtime `tests.rs`, the `ops/mission.rs` tests, `remote/client/action_tests.rs`, the `runner-app` runtime smoke tests) to the Bypass path. Keep every assertion that covers what Bypass produces for each runtime; golden launch arguments for Bypass must not change. Update golden snapshots only through their existing workflow and inspect the diff.
5. `docs/arch/arch.md`: the runtime-argv paragraph (around "follows `MissionPermissionMode` for mission slots") and the resume paragraph ("Auto/Bypass replace recorded permission args and RoleDefault retains them") describe Bypass as the only mission mode. Leave archived docs alone. The Missions guide already describes Bypass; no `docs/guide/` change is expected unless the diff shows otherwise.

Out of scope, as the issue states: role permission modes and attended chats; the client protocol, storage and CLI; the `--add-dir` grants for Codex, Copilot and Antigravity; Settings. No unrelated refactors.

## Validation

Before running checks, list the changed behaviors and the check that covers each. Expected selection: `runner-core` tests for `runtime_metadata`; `runner-daemon` tests for the session manager (permissions, runtime override, golden, resume), runtimes and `ops::mission`; `runner-app` mission-workspace label tests and the runtime smoke test files the diff touches. Then `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, `cargo fmt --all --check` and `git diff --check`. Confirm with `rg MissionPermissionMode crates docs/arch` that nothing remains outside archives. No full smoke or regression expansion. Neither slot launches the app; the reviewer's clean review names any live checks the diff needs, or says why none are needed, and those are left to Jason.

## Roles and handoffs

`coder` implements and runs the checks, then hands the working-tree diff to `reviewer` through Runner with no commit yet. `reviewer` reviews only after that handoff, posting must-fix findings first with file:line pointers. Iterate until review is clean.

## Authorization

Jason authorized this `codex duo` mission (one coder, one reviewer) with implementation, validation, commits on this branch, pushing it and opening one PR against `main`, then driving the required macOS and Windows CI green. No other agents, missions or internal subagents.

Fold this brief into the implementation commit; one focused commit is expected (for example `refactor(session): make Bypass the only mission permission mode`). Fold later review or CI fixes into the commit they correct and push with `git push --force-with-lease`; never plain force. Rebase onto `origin/main` only when AGENTS.md requires it; never merge `main` into the branch. Do not add tool-specific co-author trailers. The PR closes #1083, carries the `core-review` label (the workflow adds it; never remove it) and lists the reviewer's named live checks as not run, or the reviewer's reason for needing none.

Stop at an open PR with green CI and report through Runner with the PR URL, final SHA, changes and validation results. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. When waiting for a teammate or Jason, finish the turn and rely on the pushed inbox notification; do not start a feed watcher or polling loop.
