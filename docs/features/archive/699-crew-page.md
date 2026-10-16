# 699 — Crew page redesign

> Status: shipped in [0.12.0](https://github.com/yicheng47/runner/releases/tag/v0.12.0) on 2026-09-27; archived 2026-10-01. The original scope and dated decisions follow.

> Tracking issue: [#699](https://github.com/yicheng47/runner/issues/699)
> Priority: P1, 0.12 release blocker. Platforms: macOS and Windows.
> History: split from [393](./393-role-page.md) on 2026-09-22. The role page goes first, and this page follows in the same layout language. It is built against the roster model that [#562](./562-mission-spawn.md) settles.

## Motivation

The crew page is still the MVP draft (frame `CUKjM` in `design/runner-mvp-design.pen`). It was carried over as-is through the GPUI cutover and the #604 rename, and it buries its primary content under prompt prose:

- **Slots last**: the slot roster is the crew's actual substance, but it renders last (`crates/runner-app/src/surfaces/crews/editor.rs`). It sits below Purpose, and below Default goal and Team conventions (`editor_sections.rs`). With real prose in those sections the slots land below the fold.
- **Crowded slot rows**: each slot row (`slots.rs`) packs everything into one block: the handle, the LEAD badge, the runtime badge with its override, "from @role", a one-line prompt preview and the `$ command` summary.

## Scope

Redesign the crew page and the crew list in Pencil first, in `design/specs/archive/393-role-page.pen` beside the role page and role list, then implement to match. Both pages show the same data and run the same commands, except for the dropped fields below. The backend changes twice: `ops::mission` stops falling back to the crew's goal, and crew search (`SEARCH_PREDICATE` in `repo/crew.rs`) stops matching purpose and goal. The list joined the scope on 2026-09-27, following the role list that shipped in #393 (PR #720).

- **Profile split, like the role page**: the left column holds the crew's picture, name, a one-line summary (slot count and lead), Start mission and Edit, then the slots and the details. Team conventions fill the right column, with the crew's recent missions under them, taken from the mission summaries the app already loads. The missions card shows the four latest (frame `R68DTB`), with a footer row, "Show all 14 missions ⌄", that expands the full list in place, newest first (frame `ab98G`); the footer then reads "Show less ⌃". The card's header keeps only its title and counts. A crew with four missions or fewer has no footer.
- **Crew picture**: built from the slots' `RoleAvatar`s in slot order, the same way at every size (95 px on the page, 25 px in the list). One slot fills the tile; two sit side by side, centred vertically; three are two over one, the third centred; four or more are a 2×2 grid of the first four. Nothing is drawn for a missing slot.
- **Conventions are the crew's only prose**: purpose, default goal and team conventions were three prose fields doing overlapping jobs, and conventions alone are enough (Jason, 2026-09-24). Team conventions sit beside the slots and render as markdown, collapsed when long, the way the role page shows its system prompt. Edit turns the name into a field and the conventions into an editor that works like the role page's prompt editor: markdown text in the mono font, a Markdown | Preview switch that opens on Markdown, and a neutral card border, not the accent. Slot changes keep saving on their own.
- **Purpose and default goal dropped from the app**: purpose never reached an agent; it appeared on this page, the crew list card, the create form and crew search, which stop showing and asking for it. The default goal pre-filled the Start mission dialog and was the lead's goal when a mission started without one; every mission now states its own goal, and a repeatable job's goal belongs to the job (#630), not the crew. The dialog stops pre-filling it, and `ops::mission` stops falling back to `crew.goal` when a mission starts or resumes without a goal, so a goal nobody can see never reaches the lead. The database columns and the CLI's `--purpose` and `--goal` on `crew create` and `crew update` stay for now.
- **Slot rows**: each row reads at a glance: the slot's pixel avatar (`RoleAvatar` seeded with the slot handle, as the mission rail and feed draw it), handle, LEAD, role, and the effective runtime, model and effort, with overrides marked. Clicking a slot opens a popup beside it with its setup against the role's defaults, the command, a prompt preview, and Edit overrides, Set as lead, Open role and Remove. Edit overrides edits the slot's runtime, model and effort in the popup and saves to the slot only; the role is edited on its own page. This replaces the slot menu's Edit role drawer.
- **Reorder by drag**: today's drag-to-reorder stays, with a lighter handle. The always-visible `⋮⋮` becomes a grip in the left gutter shown only on the hovered row, and the whole row is the drag source: a click opens the popup, and a drag starts only once the pointer moves. It saves through the existing `slot_reorder`, which keeps `position` dense per crew. The order sets the crew picture's first four and the roster order a mission starts with; the lead stays whichever slot is marked lead.
- **Settled model**: the page is implemented against #562's settled roster model, not the one before it.
- **Crew list as a table** (frame `MT5Gd`), built like the role list: no panel, a rule under the header and hairlines between rows. Columns: Crew (the crew picture, then the name and `5 slots · lead @lead` in mono), Runtimes (each runtime's mark with its slot count, in slot order), Missions (how many the crew has run) and Last mission (the latest mission's start as `Sep 23, 09:40`, or `—` if none; a crew with a running mission shows `1 mission running` in the accent instead). The mission figures come from the mission summaries the app already loads, so the list needs no backend change. A row opens the crew; hovering a row turns its play icon into a Start mission button, and ⋯ keeps its menu. The subtitle becomes "Teams of roles you start missions from: slots, one lead, and the conventions they share.", and the search placeholder "Search crews, slots and roles". The pager, the `9 crews` total and narrow windows behave as on the role list; the table drops Missions first, then Runtimes and Last mission.
- **Slot popup fields**: a focused field in the popup has a neutral border, not the accent.

## Non-goals

- New fields or slot operations.
- The create and add-slot forms (`crews/create.rs`, `crews/add_slot.rs`), beyond what the new layout needs for visual consistency.
- The role page ([393](./393-role-page.md)).

## Implementation phases

1. **Design**: Pencil frames for the crew page and the crew list, reviewed before any code.
2. **Crew page**: slots first and the restructured slot row. Drag-reorder, set lead, overrides and remove keep working.
3. **Crew list**: the table and its hover Start mission, inside the existing paginated list page.

## Verification

- `runner-app` tests pass, extended where the row restructure moves behavior (`surfaces/crews/tests.rs`); workspace clippy is clean.
- Manual pass with a crew of five or more slots, per-slot overrides, and long conventions prose: the slots are visible without scrolling.
- Manual pass with more than eight crews, one with a running mission, one that never ran and one with a single slot: the table pages at eight, the running crew shows it, and search narrows the rows and the count.
- On macOS and Windows, both pages fit the 640 px minimum with no horizontal scroll, as the role pages do.
