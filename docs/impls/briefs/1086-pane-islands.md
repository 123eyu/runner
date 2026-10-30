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


## Authorized follow-up — 2026-10-30

Jason added two commits on the same branch and PR after phases 1–3 had clean review and QA. Open the PR for that scope first, then implement each follow-up through the same uncommitted code-review and QA gate before committing. Design revision: `runner/desktop/specs/1086-pane-islands.pen` at `yeelinks_design` commit `c716cbf`; exported references are `mission-tab-dark.png`, `mission-tab-light.png` and `split-chat-tab-light.png` under the ignored `target/design-ref/`. Do not open `.pen` files.

4. Mission tabs become a mission island and a Sessions/Info rail island. Keep header, member tabs, feed, composer and drawer unchanged inside the mission island. Use 6 px sidebar/rail gaps, 8 px outer insets, full-gap resize strips without accent, and chat-style focus/dimming. This supersedes the original exclusion of mission layout; its drawer remains. Update spec, architecture §3.6, bilingual mission guide and regression rows. QA compares both mission PNGs; new gutter drags go to Jason.

5. Apply Runner Light palette E only: sidebar/material `0xe2e4e9`, bg `0xf2f3f5`, panel `0xf8f8fa`, line `0xd9dbe1`, sidebar_selected `0xd6d9e0`, sidebar_selected_border `0xcaced7`, line_strong `0xcdd0d7`; keep raised/text and Catppuccin Latte unchanged. Rosé Pine Dawn background and cursor_accent become `0xf5f1eb`, with other colors unchanged; document this Runner ground override and update pinned tests. QA compares the split-chat light PNG and spot-checks light chat, terminal, mission and one Settings/Role/Crew surface for selected rows, inputs, menus and borders.

6. App scrollbars show only while in use (option B, `sidebar-scrollbar-b.png`, board “Sidebar scrollbar · B” at design revision `29aa9ca`): hidden and not clickable at rest, visible on parent-list hover/scroll/drag, 4 px muted-at-60% thumb 3 px from the right edge, widening leftward to 6 px over the 10 px strip or while dragging; hide after 900 ms from scrolling/leaving. Keep terminal scrollbars and reserved gutters. Pin geometry, hover, hidden clicks and linger in app tests; QA adds sidebar/Settings/select dark/light checks to the final pass if ready, otherwise runs a short focused delta. Thumb drag stays with Jason. This is its own commit after phase 5.

Test screenshots stay outside Git under the scratch evidence root; QA alone writes the test record and hands it back for staging/verification. The final PR still closes #1086 and keeps risk-high/core-review.

### Follow-up 4b and concurrent gates

Jason’s later mission-board design at `0c7df46` supersedes phase 4’s drawer-inside rule. References: `mission-tab-drawer-dark.png` and `mission-tab-drawer-light.png`; the closed-drawer PNGs remain valid. Stack a separate drawer island below the mission beside the full-height rail, with a 6 px row gutter (whole-gap grab strip, no line/accent), 40 px header/10 px side padding/no rule, and existing shell tabs/+ /hide. Keep saved height meaning, 280 px default and 120–600 px limits; update the terminal-size estimate. Shell focus brightens only the drawer border and restores the mission border. Preserve dimming, duplicate-window, remote and grouped take-back behavior. Pin geometry/estimate in tests and compare open/closed/focus transitions at 100%/130%; new gutter drags stay with Jason.

Status-pill correction finishes first, then 4b, then phase 5. Each receives its own clean snapshot code review, but one final QA candidate covers 4b and 5 together. Once QA confirms a candidate build is finished, coder may edit source and continue in parallel; every subsequent coder build/test uses `CARGO_TARGET_DIR=target/coder` inside this worktree, preserving QA’s binary. Review uses a frozen patch or local unpushed commit, never the moving working tree; QA relaunches its built binary and coordinates any rebuild. Push only after the applicable gates clear, folding status into phase 3, 4b into phase 4 and palette E last. The test record remains QA-only.

### Follow-up 3b — always-visible lifecycle icon

Jason’s design board “Island header · lifecycle control” at `2588dac`, exported as `island-header-lifecycle-button.png`, supersedes the hover-swap rows. Keep the current 12 px medium status and width thresholds as state only; after a 1 px × 14 px border-strong divider with 8 px gaps, show a 22 px/radius-6 button with a 13 px circled Stop (muted) or Resume (accent) icon. Hover/focus use the signed-off danger/accent tint and border, with an action/effective-shortcut tooltip. Existing lifecycle/capability guards hide both divider and button for shells, transitions, disabled and secondary views; narrow panes keep the button. Menu and Chat paused card stay. Remove the swap-pill code/tests, pin the button/divider/tooltip and update EN/zh guides. Review a frozen snapshot and fold into phase 3. Jason explicitly waived further native QA and checks this change himself; prior status-pill QA remains historical.


### Follow-up 3c — centered stopped-chat card and responsive headers

Jason’s “Stopped chat card” board at `7a0f20b`, exported as `stopped-chat-card.png`, supersedes 3b’s unchanged-card and fixed-threshold rules. Center chat Ended cards over the existing dim: width 360 px or pane minus 32 px, panel/border/radius 12, padding 24 top/sides and 20 bottom, 10 px gaps, a 36 px raised badge with 15 px square, Chat stopped title, unchanged 13 px reason at 1.5 line height, then 4 extra px before a single Primary Resume/play button with the effective shortcut at 55% opacity. Keep supplied titles and mission member slot cards unchanged; × archives. Island headers hide detail first, truncate names down to about 32 px, then hide status labels; retain lifecycle/right-side controls, including Stopped at 360 px with Claude/account and glyph/Resume at 240 px. Other status surfaces keep their thresholds. Pin rendered geometry and shortcuts, update EN/zh guides/spec, review a frozen delta after 3b and fold into phase 3. Jason waived further native QA and checks this himself.

The orchestrator supplied this design-intent short-card fallback after code review (distinct from Jason’s centered-card/narrow-header decisions and native-QA waiver): cap height at body minus 32 px; drop the badge, clamp the reason to two lines with ellipsis, then one, then omit it; keep title and Resume, dropping title only when both cannot fit. Resume stays reachable. Narrow long shortcuts stack within the button and padding contracts if required. Pin wide/short and 160/240 px-wide cases, minimum split body height, default macOS and Windows/Linux shortcuts, actual click and 100%/130% zoom; list this undrawn rule in the PR.


### Follow-up 7 — fixed opening window minimum

Jason replaced the dynamic sizing proposal with a fixed opening minimum (message `01M66P508AB19JBF0AZ8AX9ZJ6`), as a separate commit after phase 6. Pass `window_min_size` on every platform: saved sidebar width + 6 + MIN_SPLIT_PANE_WIDTH + 8, multiplied by zoom at open; height is a single constant that fits the fixed sidebar with the computer picker, or 8 + 40 + MIN_SPLIT_PANE_HEIGHT + 8 if larger, also times opening zoom. The in-process measurement was 364.5 px at 1×; use 400 px with headroom. Keep one small floor test. Remove every dynamic native hook, per-render sync/cache, mission minimum and subtree flex constraint. Splits behave as today below their size. Later sidebar/zoom changes do not update an open window’s minimum; list that limit in the PR. This supersedes both earlier orchestrator clarifications about dynamic panel sizes and split minima. Review a frozen snapshot; no live QA pass under Jason’s waiver.
