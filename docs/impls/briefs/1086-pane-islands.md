# #1086 — Pane islands, and terminals without a lifecycle

Jason requested a `codex trio` mission for [#1086](https://github.com/yicheng47/runner/issues/1086) on 2026-10-30. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-1086-pane-islands` on `feat/1086-pane-islands`, based on `origin/main` at `40e96fde3a2b0a9ad04d509d539a93769940b0fe`. The root checkout stays on `main`. Use this worktree's own `target/`; do not set a shared `CARGO_TARGET_DIR`.

## Problem and outcome

Chat and terminal tabs become islands: each pane is its own card with a header that replaces the tab bar, a chat's status doubles as its Stop/Resume control, chat tabs lose the terminal drawer, and shells lose their lifecycle and close on exit.

The spec is [`docs/features/1086-pane-islands.md`](../../features/1086-pane-islands.md); read it in full before starting. Where this brief and the spec disagree, the spec wins; where the spec and the PNGs disagree on detail, ask through Runner.

The design is exported as PNGs in this worktree's ignored `target/design-ref/`: `today-work-card.png` (the shipped layout, for contrast), `split-chat-tab.png`, `chat-tab-file-preview.png`, `terminal-tab.png` and `island-header-states.png` (every status, hover and menu state). Match their layout, spacing and colours with the existing theme tokens; terminal text is illustrative. Do not delete them, and do not open or edit `.pen` files.

## Read first

- Pane chrome: `crates/runner-app/src/surfaces/panes/` — `tab.rs` (the tab bar and its session control, Stop all/Resume all), `pane.rs` (pane header, status indicator, overlays), `tree.rs` (splits and gutters), `drawer.rs`, `mod.rs` (`pane_action_items_for`, `pane_close_behavior`, `header_fork_state`, `UNFOCUSED_PANE_OPACITY`).
- `crates/runner-app/src/ui/session_control.rs`, `ui/agent_status.rs` (`pane_status_indicator`, label and detail widths), `ui/overlay.rs` (`SessionOverlay::shell_exited`).
- `crates/runner-app/src/surfaces/chat/actions.rs` (`stop_chat`, `stop_focused_session`, `resume_chats`), `chat/close.rs`, `chat/layout.rs`, `pane_layout.rs`, `keymap.rs` (`toggle-terminal-drawer`, `stop-session`, `resume-session`).
- Mission shells: `surfaces/mission_workspace/terminal_pane.rs` and `state.rs`.
- Daemon: `repo/node.rs` (stored tab layout and `drawer`), `daemon/resume.rs` (launch resume and its drawer exclusion), `companion/actions.rs` (remote drawer requests), `session/manager/shell.rs`.
- Chrome per platform: `platform_ui/{macos,windows,linux}.rs` and `desktop.rs`; the work card in `ui/workspace_header.rs`.

## Deliverables

Land them as the spec's three phases, each commit building and passing its tests:

1. **Terminals have no lifecycle,** for tab panes and mission workspace shells: no Stop, stopped state or Restart anywhere (header, menu, ⇧⌘X, ⇧⌘R, Stop all); a pane closes on any shell exit, including shells left stopped by a daemon crash; launch resume of terminal panes after a planned stop is unchanged.
2. **No drawer in chat tabs:** retire ⌥F12 in chat and terminal tabs, close a chat tab's saved drawer shells on upgrade and drop the drawer from its layout, and make the companion's drawer requests for chat tabs fail clearly while mission drawers keep working. Mission tabs keep their drawer.
3. **Islands and the island header** as the spec describes, on macOS, Windows and Linux, including the side panel as an island, the toggle on the top-right island, the status lifecycle button, and the menu (Stop or Resume first in danger or accent, Fork chat, Rename…; no Archive chat).

Then update the guide pages, architecture and regression rows the spec lists, English and zh-CN together. Decide for any case the design does not draw, such as an empty New chat pane or a pane too narrow for its header, by keeping today's behaviour inside an island, and list each such case in the PR.

Out of scope: mission tab layout and its drawer, the work card on other routes, the browser client's code, `.pen` files, unrelated refactors.

## Validation

Before running checks, list the changed behaviors and the check that covers each. Expected: `runner-app` tests for pane menus, the header lifecycle button by status, the toggle's island, keymap and close-on-exit; `runner-daemon` tests for the drawer upgrade, launch resume and companion drawer requests; workspace Clippy with `--profile ci`, `cargo fmt --all --check`, `git diff --check`. Coder and reviewer never run the app.

After the clean code review, `qa` runs live checks in the development app under AGENTS.md Test Scope (inexpensive models, low effort, tiny prompts, fresh `/tmp` folders; archive only what it created):

- A split chat tab with an idle chat, a working chat and a shell: islands, focus, header contents, the toggle on the top-right island across a re-split.
- Stop chat and Resume chat from the status; Stop and Resume from the menu; × on a running chat asks before archiving.
- The #634 side panel open, as its own island.
- A terminal tab: a shell exiting with `exit` and with `exit 3` closes its pane; no Stop or Restart appears; ⌥F12 does nothing.
- A mission workspace shell has no Stop and closes on exit.
- Upgrade: on a build of `origin/main`, open a chat tab with a drawer shell, then start the candidate; the drawer and its shell are gone and the chat is intact.

`qa` records results in `docs/tests/1086-pane-islands.md` with screenshots, and marks anything not run with the reason. Windows and Linux native checks are left to Jason. A failed check goes back to `coder` with evidence.

## Roles and handoffs

`coder` implements and runs the checks, then hands the uncommitted diff to `reviewer` through Runner. `reviewer` posts must-fix findings first with file:line pointers; iterate until clean. The clean review confirms the live checks or names changes. `coder` then commits and pushes and hands the SHA to `qa`; after qa passes, `coder` folds the test record into the commits, opens the PR and drives CI.

## Authorization

Jason authorized this `codex trio` mission (coder, reviewer, qa) with implementation, validation, commits on this branch, pushing it and opening one PR against `main`, then driving the required CI green. No other agents, missions or internal subagents.

Fold this brief and the spec into the commits; fold later fixes into the commit they correct and push with `git push --force-with-lease`. Rebase onto `origin/main` only when AGENTS.md requires it; never merge `main` into the branch. No tool-specific co-author trailers. The PR closes #1086, keeps `core-review` if the workflow adds it, takes one risk label (expect `risk-high`), and lists the live checks run and left to Jason.

Stop at an open PR with green CI and report through Runner with the PR URL, final SHA, changes and validation. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. When waiting, finish the turn and rely on the inbox notification; do not poll.
