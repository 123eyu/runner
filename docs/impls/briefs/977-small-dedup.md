# #977 — Small duplicate cleanups (items 2, 9 and 10)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. This is the first of them. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-small-dedup`, on branch `refactor/977-small-dedup`, based on `origin/main` at `3d860523`. The root checkout stays on `main`; this worktree uses its own `target/`.

Read the issue body and its [2026-10-26 re-audit comment](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449) first. This mission covers items 2, 9 (what remains of it) and 10. Items 1, 4, 5, 7 and 8 are later missions: do not start them here.

No behavior change is intended, except the trust-root question in item 2, which needs evidence first.

## Deliverables

One pull request with one focused commit per item, each building and passing its tests on its own.

### Item 2: one trust-root helper

`is_broad_trust_root` is copied in `runtimes/codex/codex_trust.rs`, `runtimes/copilot/copilot_trust.rs` and `runtimes/antigravity/agy_trust.rs` (all under `crates/runner-daemon/src/`). Move it to `runtimes/helpers.rs` and have the three adapters call it. Keep the existing tests passing, and move the shared case to the helper.

The issue also claims that Copilot and Antigravity "trust a worktree path differently" because only Codex calls `resolve_worktree_main_root`. Don't assume that's a bug. Codex keys trust by project root, and Codex itself resolves a linked worktree to its main root. Copilot and Antigravity trust the canonical cwd. Before you change their semantics, check what each CLI actually compares against its trusted list. Use its documentation or source, and the existing adapter docs in `docs/arch/runtime-integration.md` and `docs/guide/reference/supported-agents.md`.

- If a CLI needs the main root for a linked worktree, share `resolve_worktree_main_root` and add a worktree test for that adapter.
- If it doesn't, leave its behavior unchanged, and say so with the evidence in the PR description so the issue can be corrected.

### Item 9: one attached-terminal setup

`#1035` already moved the cursor-style mapping into `app_settings::configure_terminal`. What remains is the setup copied between `ChatSurface::ensure_attached` in `crates/runner-app/src/surfaces/chat.rs` and `ensure_mission_terminal_attached` in `crates/runner-app/src/surfaces/mission_workspace/attach.rs`. About 40 lines are nearly identical: the running-status check, the bridge session lookup, palette, scrollbar, interaction, input and focus subscriptions. Extract the shared part into one helper that both call. Remote panes in `surfaces/remote_client/workspace.rs` should use it too where their setup is the same. Keep each caller's genuinely different parts at the call site. Also check the remaining palette-setup sites the re-audit listed. Fold any that are the same operation into `app_shell::terminal_style_for` or the new helper, and leave any that differ.

### Item 10: smaller items

- `surfaces/start_chat.rs` pulls in `mission_workspace/add_role.rs` and `role_setup.rs` through `#[path]`. Give them a home where the module tree matches the folders, and remove the `#[path]` attributes. The platform-selected `#[path]` uses for `*_linux.rs` / `*_windows.rs` files are not part of this.
- The near-copies between `surfaces/crews/mod.rs` and `surfaces/roles/mod.rs` (`set_library_load`, `invalidate_library` and the block after them), and between `surfaces/crews/list.rs` and `surfaces/roles/list.rs`.
- The block of about 46 lines shared by `surfaces/settings/about.rs` and `surfaces/settings/updates.rs`.

For the near-copies, extract a shared helper only where it is simpler than the copies; AGENTS.md prefers three similar lines over a premature abstraction. Where you leave a copy, say why in the PR description. Note: the external draft PR #997 also edits `crews/` and `roles/`. It is stale and already conflicting, so do not wait for it, but keep the moves small.

## Read first

- `AGENTS.md` (Engineering Conventions, Test Scope, Crew Missions) and `CONTRIBUTING.md`.
- The files named above, plus the callers of each changed function.

## Validation and handoff

Before running checks, list the changed behaviors and the check that covers each. These are refactors that the in-process tests cover: run the `runner-daemon` runtime tests for item 2 and the `runner-app` tests for items 9 and 10, then `make clippy` and `make fmt`. `make verify` must pass before the PR. Add tests only where behavior or a shared helper's contract is new, such as the worktree case if item 2 changes an adapter.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` reviews the diff and names any live checks it needs under Test Scope, or says why none are needed. This crew has no qa slot, so live checks are left to Jason and listed in the PR as not run.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: implementation, tests, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977" (not "Closes"), and driving the required macOS, Windows and Linux CI green. Fold this brief into the item 2 commit instead of committing it on its own. Fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the checks that were run, and the item 2 evidence through Runner.
