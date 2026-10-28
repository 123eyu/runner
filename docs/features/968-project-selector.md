# #968 — Project selector in New chat and New mission

This spec covers [#968](https://github.com/yicheng47/runner/issues/968): choosing the project of a new chat or mission inside the New chat and New mission dialogs. The signed-off design is `runner/desktop/specs/968-project-selector.pen` in the design repo; frame ids below refer to that file.

## Motivation

Both dialogs file their work under a project that the user cannot see or change. The sidebar's New Chat, ⌘N and ⇧⌘M use the *active* project (the one whose chat or tab was last selected), a project's + menu uses that project, and the crew page's Start mission always starts unfiled. The only way to pick a project today is to navigate into it first. The main workflow starts from the sidebar's New Chat, so the choice has to be visible and editable there, before the chat starts.

## Decisions

Jason signed off option A and every recommendation on 2026-10-28.

1. **Placement** (board `hXymk`): a Project field directly above Working directory (`dNbRf`). It adds the least new UI (the existing `Select` with the crew picker's rows), reads as one pair with the directory it drives, and is identical in both dialogs. The rejected options stay on the board: B, a project chip in the dialog title, was a small target far from the directory; C, one card for project and directory, stacked a second card under the agent card.
2. **Default from the sidebar's New Chat and ⌘N**: the active project, as today but now shown; No project when Recents or nothing is active (board `ky8Ar`, state 2).
3. **Crew page Start mission**: the active project instead of always unfiled, so every entry point without its own project follows decision 2.
4. **A directory inside another project switches the selection** (state 5), as the same directory files from the CLI under #680, rather than keeping the choice and only warning.
5. **An empty pane of an existing tab** shows the selector read-only with a lock, because a pane's chat files with its tab (state 7).

## Behaviour

### The field

Project sits between Chat name and Working directory in New chat, and between Goal and Working directory in New mission (`xO5YM`). Its trigger shows a folder icon and the project name, or a history icon and **No project**; it does not repeat the path, which the working directory below already shows. The menu (`XkvxJ`) lists No project ("Listed under Recents. Starts in your default directory."), a separator, then every project in sidebar order with its path under the name in mono and a check on the chosen row, as the crew picker lists No crew and then crews. Keyboard handling is the existing `Select`'s, and the field joins the dialog's focus order after Chat name or Goal. With no projects the field still shows, with No project as its only choice.

### Initial choice

| Entry point | Project |
| --- | --- |
| A project's + → New chat or New mission | That project |
| Empty pane's New chat | The tab's project, locked |
| Sidebar New Chat, ⌘N, ⇧⌘M, crew page Start mission | Active project, else No project |

A chosen project fills the working directory with its folder, as today. No project leaves it blank, so it starts in the role's or the default directory.

### Project and working directory

The selector always shows where the work will be filed, and the last explicit action wins (board `ky8Ar`):

- Choosing a project sets the working directory to its folder, unless the directory is already inside that project, such as a worktree under `.worktrees/` (states 3 and 4).
- Choosing No project leaves the directory unchanged.
- When the working directory loses focus or Browse returns a folder, a directory inside a project chooses that project, the deepest one when projects nest (state 5). The match uses the same rule as `repo::project::find_for_path`: path components rather than string prefixes, canonical paths where both exist, longest ancestor wins.
- A directory outside every project keeps the current choice. With a project chosen, the hint turns amber: "Outside runner's folder. It is still filed under runner." (state 6). Nothing is blocked.
- In a locked pane, a directory in another project does not switch the selection; it shows the state 6 hint (state 7).

### Submission

The dialog sends the scope it shows: `ProjectScope::Project(id)` for a project and `ProjectScope::Root` for No project, through the existing `or_root` mapping in `front_page/client.rs`. App starts never use `Infer`, so runnerd files the work exactly where the dialog said.

### Remote computer

In a window on a remote computer, the menu lists the host catalog's projects and paths, as the folder menu already does, and submission sends the chosen host project id. Matching a typed directory against host projects is lexical only, since host paths cannot be canonicalised locally.

## Out of scope

Creating a project from the dialog (the sidebar keeps that), filtering or searching the menu, moving an existing chat or mission to another project, and changes to CLI flags or `--project`/`--cwd` semantics.

## Implementation Phases

1. **Shared matcher.** Move the longest-ancestor match out of `repo::project::find_for_path` into a pure function over project folder paths and a path in `runner-core`, with a lexical-only variant for host paths. `find_for_path` calls it; its existing tests keep passing.
2. **New chat.** Add a project `Select` to `StartChatModal`, built from `app_store.projects` or the host catalog. Selection updates the field's scope and applies the directory rules; working-directory blur and the Browse result run the matcher. The pane target renders the select disabled, with the subtitle "Chats in a pane file with their tab." in place of the design's lock icon and tooltip, which `StyledSelect` does not support; No project's history icon is the app's `clock.svg`. Both deviations await Jason's confirmation on the PR. Route the sidebar New Chat, ⌘N and pane entry points through the table above; the + entry is unchanged.
3. **New mission.** The same field and rules in `StartMissionModalState`, local and remote forms, with the crew page's Start mission passing the active project.
4. **Guide.** Update the pages under Documentation in English and `zh-CN`.

## Verification

Automated, in `runner-app` and `runner-core`:

- Matcher: exact match, deep descendant such as `.worktrees/<branch>`, nested projects choose the deepest, `runner` versus `runner-web` siblings do not match, no match returns none, lexical host paths including Windows separators and UNC roots.
- New chat: each entry point opens with the expected project; choosing a project rewrites a directory outside it and keeps one inside it; choosing No project keeps the directory; a directory typed into another project switches the selection on blur and after Browse; a directory outside every project keeps the choice and shows the amber hint; a pane chat is locked to its tab's project; submission sends `Project(id)` or `Root` with the shown directory.
- New mission: the same rules, and the crew page entry using the active project; the remote form sends the chosen host project.
- Workspace Clippy and `make fmt`.

Live checks, in the development app, because the change alters what a person sees and operates:

- From the sidebar's New Chat, choose a project and start a chat; it appears under that project.
- Start a mission with an explicitly chosen project; it appears under that project.
- Open New chat and New mission from project A's +; A is chosen. Change it before starting; the work lands under the new choice.
- Type or browse a directory inside another project; the selection follows. A directory outside every project shows the amber hint and keeps the choice.
- An empty pane's New chat shows the locked field.
- The selector and filing on macOS and Windows.

## Documentation

- `docs/guide/concepts/projects.md`: choosing a project in the dialogs, and how the selection follows the working directory.
- `docs/guide/getting-started/first-chat.md` and `docs/guide/getting-started/first-mission.md`: the Project field in the steps.
- `docs/guide/guides/chats-and-windows.md`: the remote section's host projects in the menu.
- The matching pages under `docs/guide/zh-CN/`, checked with `python3 script/check-guide-translations.py`.
