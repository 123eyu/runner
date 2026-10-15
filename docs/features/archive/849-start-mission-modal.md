# 849 — Start mission dialog redesign

> Tracking issue: [#849](https://github.com/yicheng47/runner/issues/849)
> Priority: P2, milestone 0.16 (0.14 until 2026-10-10). Platforms: macOS and Windows.
> Status: merged on 2026-10-15 in [#906](https://github.com/yicheng47/runner/pull/906) with #562 Phase 3, as its own commit (`docs/impls/562-mission-spawn/plan.md`, Phase 3 decision 11); #849 is closed.
> Design: `design/specs/562-mission-spawn.pen`, drawn together with #562 Phase 3: Start mission from a crew (`AVJ15`) and the crew picker open (`iJuGG`).
> Related: [562](../562-mission-spawn.md)'s Phase 3 adds **No crew** as the first entry of this dialog's crew picker, for a mission that starts with no roles (settled 2026-10-15), and is drawn and built together with this redesign. This spec covers the crew mode; the No crew entry is 562's.

## Motivation

The role and crew pages ([393](./393-role-page.md), [699](./699-crew-page.md)) and Start a chat ([735](./735-start-chat-modal.md)) lead with identity and setup: pixel avatars, the crew picture, provider marks, and each slot's runtime, model and effort. Start mission is the last create dialog still in the old form style (`crates/runner-app/src/surfaces/start_mission.rs`):

- **The crew is a text dropdown.** The selected crew shows its name over `lead: @coder · 1 worker: @reviewer` (`summarize_crew`), and the other crews in the menu show only `N roles`. Nothing shows who will spawn or what each slot runs, so a wrong model or runtime on a slot only shows up once the mission is running.
- **The footer only counts.** "2 sessions will spawn" is the only roster information near the Start button.
- **Advanced is dead UI.** The collapsible "Advanced: env overrides · per-role args · attach files" opens onto "Reserved for v0.x — custom env, dry-run mode. Inert in v0 MVP." and has never done anything.

## Scope

Design the dialog (drawn in `design/specs/562-mission-spawn.pen` with #562 Phase 3) using the visual language of the shipped crew page (`runner.pen`'s ROLES & CREWS band, crew page `lRLKB`) and Start a chat (`design/specs/735-start-chat-modal.pen`), then implement the approved frames.

- **Crew card.** Like Start chat's role card, the crew is one card. Its top row is the crew picker: the crew picture (`crew_picture`), the crew name, a summary in mono in the crew list's form (`2 slots · lead @coder`) and an up-down chevron. The picker's menu lists crews the same way: picture, name and summary. The menu needs no new loading because `CrewListItem.members` already carries every crew's slot handles, lead and runtime.
- **The roster under the picker.** One read-only row per slot, in slot order. Each row has the slot's `RoleAvatar`, handle, LEAD, role, and the effective runtime, model and effort with overrides marked, the same as the crew page's slot rows (`SlotSetup`, `slot_setup_line`, `lead_badge`). These rows replace the footer's session count. A crew of eight slots scrolls with the dialog body, not inside the card.
- **Fields.** Mission title, Goal and Working directory keep their behavior and move to the new field style. The Goal hint still names the lead ("Delivered to @coder (lead) on mission start."). The Working directory keeps Browse and the project's directory as its starting value. Its hint says where a blank field starts, in Start chat's form.
- **Dropped.** The Advanced section and its placeholder. The footer becomes Cancel and Start mission, as in Start chat, with the same `esc` and ⌘↩ hints.
- **Room for 562.** 562's Phase 3 lists **No crew** first in the crew picker, above the crews; picking it turns the crew card into an empty state. The picker and card are drawn so that entry fits without a mode switch.
- **States.** The frames cover a crew that is picked and the picker menu open. A crew with five or more slots and some overrides follows the same card with more rows, and a crew with no slots keeps today's "This crew has no roles" notice under its picker; neither has its own frame. Other states that are not drawn keep today's copy in the new layout: loading, no crews yet, the error banner, and Starting….
- **Unchanged behavior.** Crew preselection from the crew page and the crew list's Start mission. The project scope and its working directory. ⌘↩ from every control, including an open picker and the goal textarea. Focus order, minus the Advanced toggle. The dialog fits Runner's 640 × 480 minimum window, scrolls vertically and does not clip horizontally. Long crew names, handles and paths truncate.

## Non-goals

- **Overrides for one mission.** The roster is read-only. Changes to a slot's runtime, model or effort stay on the crew page. Overrides that apply to one mission only would need `MissionStart` and the daemon to take per-slot input. They fit after 562 gives each mission its own slots.
- **562's empty mission.** Starting a mission with no roles, and the picker's No crew entry for it, are designed and built in 562's Phase 3.
- **Backend changes.** `MissionStart`, `ops::mission` and the CLI's `mission start` stay as they are.
- **Changes to the crew page, the crew list or Start a chat.**

## Implementation phases

1. **Design.** Drawn in `design/specs/562-mission-spawn.pen` with #562 Phase 3 and signed off on 2026-10-15.
2. **Dialog.** Update `crates/runner-app/src/surfaces/start_mission.rs`. Move the crew page pieces the roster reuses (`crew_picture`, `SlotSetup` and its resolution, `slot_setup_line`, `lead_badge`) from `pub(super)` to `pub(crate)` instead of copying them. Remove the `advanced_open` state and its focus handle.

## Verification

- The `runner-app` tests pass. The keyboard tests in `start_mission.rs` are updated for the new focus order. New tests cover the picker's crew summaries and the roster rows for a crew with overrides. Workspace Clippy is clean.
- Manual pass on macOS and Windows. Start a mission from the crew page, the crew list and ⇧⌘M. Switch crews and check that the roster follows. Use a crew of five or more slots with per-slot overrides, a crew with no slots, and a start that fails, and check the error banner.
- At 640 × 480 the dialog can be used by keyboard and mouse with no horizontal scrolling.
