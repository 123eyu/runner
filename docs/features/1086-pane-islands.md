# 1086 — Pane islands, and terminals without a lifecycle

> Tracking issue: [#1086](https://github.com/yicheng47/runner/issues/1086), which absorbed [#900](https://github.com/yicheng47/runner/issues/900) (closed 2026-10-30).
> Status: in development. Design signed off by Jason on 2026-10-30.
> Design: `runner/desktop/specs/1086-pane-islands.pen` in `yeelinks_design`; the #634 screens in `runner/desktop/specs/634-file-preview.pen` are drawn in the same islands.
> Related: [634](./634-file-preview-and-review.md) (the side panel's file preview), [#567](https://github.com/yicheng47/runner/issues/567) (pane × archives).

## Motivation

A chat tab is one work card: a 44 px tab bar over its panes and the side panel, split by 1 px rules. A split tab adds a 26 px header to each pane and dims the unfocused ones to 70%. Tab-wide controls (Stop all, Resume all, fork, the drawer and side panel toggles) live in the tab bar, away from the pane they act on, and a chat tab also has a bottom terminal drawer (⌥F12), a second way to hold a shell beside the pane tree.

Drawing each pane as its own island gives every session a clear edge and one header for its name, status and controls, for any split. The drawer then has no reason to exist: a shell is just another pane.

Shells also carry a lifecycle they do not need. Stopping a shell leaves a dimmed pane with a "Shell exited · exit 1" card and two different Restart controls, and the "exit 1" is really the SIGHUP Stop sent. Stop exists for agents because their conversation can be resumed. A terminal such as Ghostty has no stop: a shell ends by closing its pane or exiting, and Ctrl+C handles a stuck command.

## Scope

### Islands

- Every pane of a chat or terminal tab is its own island: a card in the workspace background (`theme::bg`) with a 12 px radius and a 1 px border, 8 px apart on the window material, with the same 8 px inset the work card has today. The focused island's border brightens (white at about 20% in dark, ink at about 22% in light); unfocused panes keep `UNFOCUSED_PANE_OPACITY`.
- There is no tab bar. Its controls move into each island's 40 px header.
- The #634 side panel is the last island in the row, unchanged inside; its own header (Info, Files, collapse) is the island header.
- Mission tabs keep their layout. Role, crew, Settings and the other routes keep the work card.

### Island header

- **Left:** runtime mark (shell mark for a terminal), name, the Claude account chip when there is one, and the status.
- **Right:** the menu (…), the split icon (Split Right ⌘D, Split Down ⇧⌘D) and ×, on every island.
- **Side panel toggle:** the island whose top-right corner is the tab area's top-right corner ends its header with the toggle, after a 1 px divider that separates it from that pane's own buttons. It stays where the tab bar's right end puts it today, whatever the split. Hidden while the panel is open (the panel's collapse button replaces it) and while a shell is focused, as today.
- **Status:** at rest exactly as `agent_status::pane_status_indicator` draws it today: ring for Idle; spinner for Working, Starting and Resuming; square for Stopped; the warning colour when the agent needs you; danger for errors; label only from 480 px (320 px when the agent needs you), detail from 560 px (400 px).
- **A chat's status is its lifecycle control.** On hover and focus it becomes a button for what the session allows, following today's tab-bar control and pane menu: **Stop chat** (square, danger) while the session runs, whatever the agent is doing; **Resume chat** (play, accent) once stopped; no action during Starting and Resuming, or when the lifecycle is disabled. Stop ends the agent process and keeps the chat; it is not an interrupt. Below the label width the button keeps only its icon. No tooltip repeats the button's label. Secondary (duplicate-window) views keep today's rule and offer no lifecycle action.
- With a control on every chat island, Stop all and Resume all go away. ⇧⌘X and ⇧⌘R keep acting on the focused chat.
- **×** archives a chat through today's confirmation ("…is still running. Archiving stops it…") and closes a terminal; ⌘W does the same on the focused pane.

### Menu (…)

- A chat: **Stop** (danger, ⇧⌘X) or **Resume** (accent, ⇧⌘R) first, then **Fork chat**, then **Rename…**. Fork keeps `header_fork_state`'s rules: hidden for shells and secondary views, disabled without a reason when the runtime cannot fork, and with "No session key captured yet" when it has no key.
- **Archive chat** leaves the menu, because × archives.
- A terminal: **Rename…** only.

### Terminals have no lifecycle

- No Stop, no stopped state and no Restart for shells: in the island header, the menu, ⇧⌘X, ⇧⌘R and Stop all. The terminal status reports activity only and is never a button.
- A shell ends with × or ⌘W, or by exiting; its pane closes on any exit, clean or not. The shell-exited overlay (`SessionOverlay::shell_exited`) goes away for every shell.
- Shells left stopped by a daemon crash close their panes the same way.
- Launch resume is unchanged: after a planned stop (an update, or quitting under a policy that resumes), terminal panes come back as today.
- This applies to every shell: tab panes and mission workspace shells.

### No drawer in chat tabs

- A chat tab has no terminal drawer. A shell is an ordinary pane, split from a chat or terminal pane.
- ⌥F12 (Toggle terminal drawer) is retired in chat and terminal tabs.
- On upgrade, the shells of a chat tab's saved drawer are closed and the drawer leaves the tab's layout.
- Mission tabs keep their drawer.

## Implementation phases

1. **Terminals have no lifecycle.** Independent of the islands, and can land first.
2. **No drawer in chat tabs,** including the upgrade that closes saved drawer shells and the remote companion's drawer requests for chat tabs.
3. **Islands and the island header,** on macOS, Windows and Linux, including the #634 side panel.

## Verification

- Unit tests for the pane menu items (chat running, chat stopped, no session key, terminal), the island header's lifecycle button by status, the side panel toggle's island, the removed ⌥F12 binding, the drawer upgrade and the close-on-exit rule for shells, including after a crash, and launch resume of terminal panes.
- Live checks in the development app: a split chat tab with a chat, a working chat and a shell; Stop chat and Resume chat from the status; × on a running chat; the side panel toggle across splits; the #634 panel open; a terminal tab whose shells exit cleanly and with a non-zero code; a mission workspace shell; a chat tab saved with a drawer on the previous build, after the upgrade.
- Windows and Linux in CI, with native checks left to Jason.

## Affected documentation

[Chats and windows](../guide/guides/chats-and-windows.md), [First chat](../guide/getting-started/first-chat.md), [Remote access](../guide/guides/remote-access.md) where it lists drawer shells, [Keyboard shortcuts](../guide/reference/keyboard-shortcuts.md), and their zh-CN copies; [architecture §3.6](../arch/arch.md#36-surface-hierarchy--project--window--tab--pane) and the drawer and shell paragraphs it links; the [workspace regression suite](../tests/regression/workspace.md) rows for drawers (WS-DRAWER-01/02), shells (WS-SHELL-*) and the work card (WS-CARD-*).
