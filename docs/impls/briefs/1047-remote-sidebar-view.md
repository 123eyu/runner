# #1047 — Remember a remote computer's sidebar view

Jason authorized this codex duo mission on 2026-10-27. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-1047-remote-sidebar-view` on `fix/1047-remote-sidebar-view`, based on `origin/main` at `6c2108b9e467fda916f5f02ae374bffb2846efee`, with this worktree's own `target/`. The root checkout stays on main.

## Problem and outcome

In a viewer window, choosing **Show active only** on remote computer A, switching to this computer or to computer B, and coming back to A resets A's sidebar to **Show all** ([#1047](https://github.com/yicheng47/runner/issues/1047)). The collapse choices kept with it reset the same way: collapsed projects, collapsed Active groups and the Projects/Chats sections.

Cause: `visit_computer` builds a new `Sidebar` on every visit, and `Sidebar::set_front_page_target` seeds its in-memory `remote_settings` from defaults. #1037's fix (`remember_remote_selection`, `ClientUi::selections`) remembers only the selected item and the layouts, so the sidebar's view settings are dropped whenever the window leaves a computer.

Intended behavior: each window remembers each remote computer's sidebar view, in memory, exactly as #1037 remembers its selection, and restores it on the next visit. The first visit in a window still starts on Show all with nothing collapsed, independent of the local sidebar; that default from #1016 is deliberate. Local sidebar settings are never changed by remote toggles. Nothing is persisted: after a restart every window opens on this computer, as today.

## Read first

- `crates/runner-app/src/surfaces/remote_client/workspace.rs`: `visit_computer`, `return_to_this_computer`, `remember_remote_selection`, and where `client_ui.selections` is consumed when the `RemoteWindow` is built.
- `crates/runner-app/src/surfaces/remote_client/mod.rs`: `ClientUi::selections`.
- `crates/runner-app/src/notifications.rs`: the third leave path, following a local notification, which already calls `remember_remote_selection`.
- `crates/runner-app/src/surfaces/sidebar/model.rs` (`set_front_page_target`) and `sidebar/state.rs` (`remote_settings`, `settings`, `update_app_settings`).
- `crates/runner-app/src/surfaces/remote_client/tests.rs`: `switching_back_to_a_computer_restores_its_selection_and_split_focus`, the #1037 regression test to extend or sit beside.

## Deliverables

1. When a window leaves a remote computer by any of the three paths, it remembers that computer's sidebar view settings alongside its selection; the next visit restores them into the new `Sidebar` before it renders. Prefer extending the existing per-computer memory over a parallel map, and keep the restore path through the sidebar's existing remote settings rather than a new flag.
2. A remembered view is discarded under the same condition as the remembered selection (the `overviews` check in `visit_computer`), unless there is a reason to keep it; say which in the handoff.
3. Guide: in `docs/guide/guides/remote-access.md`, the sentence about reopening the tab or mission you left also says the window keeps that computer's Show active only choice and collapsed sections. Keep the statement that remote collapse choices stay separate from the local sidebar.

## Non-goals

Persisting across restarts; the remote whole-sidebar hidden state (`RemoteWindow::sidebar_collapsed`) and width; scroll position; the new filters proposed in #990; any change to local sidebar behavior or to #1016's first-visit defaults. No UI or Pencil work.

## Validation

Name each changed behavior and the check that covers it before running checks. Selected, as `runner-app` tests against the existing remote-client fixture:

- Remote A (Active) → this computer → A restores Active.
- A (Active) → B, left on Show all → A: each computer keeps its own choice.
- A (Active) → Show all → this computer → A restores Show all.
- A collapsed project or section survives the same round trip.
- The local sidebar's `sidebar_active_only` and collapse settings are unchanged by remote toggles.
- #1037's selection and split-focus test still passes.

Then `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, `cargo nextest run --locked -p runner-app --cargo-profile ci`, and `git diff --check`.

Live checks are required, since the change alters what a person sees in the running app, but this crew has no qa slot: neither slot runs the development app. The reviewer names the live checks in its clean review; the coder lists them in the PR as not run, for Jason.

## Roles and handoffs

The coder implements and hands to the reviewer with the worktree, a summary, the changed files and the checks run. The reviewer reviews the working-tree diff against this brief and returns must-fix findings or a clean review naming the live checks. Iterate until clean. After a clean review, the coder commits, pushes, opens the PR and drives macOS and Windows CI green; the reviewer confirms the final pushed SHA is the one it reviewed.

## Authorization

Jason authorized this two-slot crew (coder, reviewer), implementation, validation, commits, pushing this branch, and opening one PR against `main` after a clean review. Shape the history as AGENTS.md's Crew Missions says: fold this brief into the implementation commit, with a subject that names the change; fold review and CI fixes into the commit they correct and push with `git push --force-with-lease`, never plain force. The PR body says `Fixes #1047`. No co-author trailers and no Claude session URLs in the PR body. Rebase onto `origin/main` only when AGENTS.md requires it; never merge main into the branch.

Stop at an open PR with green CI on both platforms and report the PR URL, final SHA, a short summary, the validation results and the live checks left for Jason. Do not merge, cut a nightly or release, archive the mission, delete branches or remove worktrees. No internal subagents, extra missions or other delegated agents. When waiting for a teammate, finish your turn and rely on Runner's pushed inbox notification; do not poll or watch the feed.
