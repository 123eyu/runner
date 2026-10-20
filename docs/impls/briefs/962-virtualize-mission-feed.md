# 962 — A long mission feed stalls the mission view

Fix [#962](https://github.com/yicheng47/runner/issues/962). Jason requested this mission on 2026-10-20. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-962-virtualize-mission-feed`, on the existing branch `fix/962-virtualize-mission-feed`, created from `origin/main` at `a9d3b290`. The root checkout stays on main. Do not create another branch or worktree, and do not share a Cargo target directory. Other crews work in `.worktrees/feat-836-m2-host-trust`, `.worktrees/feat-887-active-sidebar`, `.worktrees/feat-session-shortcut-hints` and `.worktrees/nightly-remote`; treat them as other machines' checkouts.

This is a frontend-only fix in `crates/runner-app`. No daemon, protocol, CLI or event-log change.

## The bug

The mission feed is a plain `overflow_y_scroll` div that holds an element for every feed block. Every render of `MissionWorkspace` clones all `feed_blocks`, builds every block (off-screen ones included), and re-parses every message's markdown. Every appended event also does a linear duplicate scan, re-sorts the whole `events` vector and re-runs `rebuild_event_projection`, which clones every event into new `feed_blocks`. The feed composer lives in the same entity and its observer calls `cx.notify()` on every input change, so each keystroke pays for the whole feed too. The cost grows with feed length until the view stalls.

## Read first

- `AGENTS.md`, including Worktrees, Test Scope and Crew Missions.
- Issue #962, and #947 (feed scrollbar, not part of this mission; see Boundaries).
- `crates/runner-app/src/surfaces/mission_workspace/feed.rs`: `render_mission_feed` (about line 206; the clone at 211 and the scroll div at 238–250), `render_feed_markdown` (423), `render_mission_feed_block` (473), `render_mission_message_group` (535), `render_mission_ask_card` (771, its `pending_ask_choices` read at 812), the "New messages" pill (274–305).
- `crates/runner-app/src/surfaces/mission_markdown.rs`: `render_markdown` and its `parse_blocks(text)` call (about line 330); `FeedSelection` (34), which is scoped to one `event_id`.
- `crates/runner-app/src/surfaces/mission_feed.rs`: `FeedBlock`, `group_feed_blocks` (137) including `can_join_group`, the 5-minute group gap and the `relayed` set. A later message whose `payload.source_event` names an earlier `human_said` signal hides that earlier block, so an append can change an earlier block, not only the tail.
- `crates/runner-app/src/surfaces/mission_workspace/events.rs`: the `mission/event-appended` handler (about lines 61–77) and the resync replay (about 315–331).
- `crates/runner-app/src/surfaces/mission_workspace/state.rs`: the composer observer (97–105), the mission-switch reset (262–271), `rebuild_event_projection` (357), `feed_is_near_bottom` and `handle_feed_append` (461–480).
- The other `feed_scroll` users: `attach.rs:132` (scroll to bottom on load) and `input.rs:108` (on returning to Feed).
- gpui-pre 0.3.7's `src/elements/list.rs` in the Cargo registry: `list`, `ListState::new`, `ListAlignment`, `splice`, `reset`, `FollowMode::Tail` (`set_follow_mode`), `scroll_to_reveal_item`, `logical_scroll_top`, and the scrollbar hooks `set_offset_from_scrollbar` and `max_offset_for_scrollbar`. Its own tests show how follow-tail suspends on user scroll and re-engages at the bottom.
- `crates/runner-app/src/ui/scrollbar.rs`: `Scrollbar::app` takes a `ScrollHandle`.

## Deliverables

1. **Render only the visible feed.** Replace the scroll div with gpui's `list` over a `ListState`, rendering each block by index from `feed_blocks`. Keep the feed looking exactly as it does now: the same padding, the same 18px gap between blocks, the "No events yet" row and the empty-mission panel. Keep these scroll behaviours, using `FollowMode::Tail` where it fits instead of the hand-rolled near-bottom check:
   - Opening or reloading a mission lands at the bottom.
   - A human-authored append always scrolls to the bottom.
   - Any other append while at or near the bottom stays pinned to the bottom.
   - An append while scrolled up keeps the viewport still and shows the "New messages" pill; clicking it scrolls to the bottom and hides it; reaching the bottom by hand hides it too.
   - Returning to the Feed tab while you were at the bottom shows the bottom.
   - Switching missions resets the list state (the reset at `state.rs:262–271`).

   Anything that changes a rendered block's height must re-measure that item: an ask card being answered or resolved, a pending choice appearing, a message group gaining a message, and a settings change that affects text size. Use a `splice` of the affected index; use `reset` only for whole-feed changes.
2. **Stop rebuilding the whole feed per event.** On a normal append at the tail, dedupe by id with a set instead of a linear scan, and insert in sorted position instead of re-sorting the vector (event ids are ULIDs, so appends are almost always last). Update `feed_blocks` incrementally: extend the last `MessageGroup` or push a new block, then splice only the changed range. When an append can change earlier blocks (a `source_event` that hides an earlier `human_said`, an out-of-order id) or on resync replay and mission load, fall back to the full `group_feed_blocks` and a list `reset`. The incremental result must always equal `group_feed_blocks` over the same events. Leave the session-status and ask projections as full rebuilds unless your measurement shows them mattering; if you change them, say why in the PR.
3. **Parse markdown once per message.** Cache `parse_blocks` output per event id on the workspace (events are append-only, so a message's text does not change), clear it on mission switch, and have `render_markdown` take the parsed blocks. Only the visible rows build elements; the cache removes the parse cost from every render.
4. **Composer keystrokes stay cheap.** After 1–3 a keystroke re-renders only the visible rows. Measure it. Only if typing in the composer still shows feed cost on a 2,000-event feed, move the composer or the feed into its own entity so composer edits stop notifying the feed. Do not split the entity on speculation.
5. **Selection and copy keep working.** Drag-select inside a message, Cmd-C copy, selection that survives scrolling the message out of view and back, and the clear-selection rules in `rebuild_event_projection` all behave as before.
6. **Tests.**
   - Incremental blocks: for append sequences covering group joins, the 5-minute gap split, author changes, dividers, ask cards, hidden system signals and a relayed `human_said` being hidden by a later message, assert that after each append the incremental `feed_blocks` equals `group_feed_blocks` over the same events.
   - Insertion: duplicate ids are ignored and an out-of-order id lands in sorted position.
   - Render cost: with a 2,000-event fixture, assert through a test-only counter that a render after warm-up parses no markdown, and that appending one message parses at most that message. Do not assert wall-clock times.
   - Scroll state: with gpui's `TestAppContext`, assert the list item count tracks `feed_blocks`, an append at the bottom stays pinned, and an append while scrolled up sets the new-messages flag without moving the viewport.

   Reuse the existing `mission_workspace/tests.rs` harness; do not build a new one.
7. **Measurement for the PR.** Report a before/after number for building the feed with 2,000 events, and for one append, from a throwaway timing run (not committed, not a gate). Name the profile and machine.

## Boundaries

No visual change, no design file edits, no daemon, protocol or CLI change, and no `docs/guide/` change: the feed behaves the same, only faster. Do not implement #947. Do leave it possible, and say in the PR body how #947 would mount the app scrollbar on the new list, since `Scrollbar::app` takes a `ScrollHandle` and `ListState` has its own scrollbar hooks. Do not change the shared `Scrollbar` in this mission.

Coders and reviewers never run the dev app or drive Jason's Runner. A `qa` slot may join this mission later to run the live checks below in the development app; when it reports findings, the coder fixes them, folds the fixes into the commit they correct, and hands back to the reviewer. Do not start extra agents, crews or subagents.

## Live checks for QA

These run only after a qa slot joins, against the branch's development build, in a fresh owned folder under `/tmp`, following Test Scope in `AGENTS.md`. Seed a long feed without spending agent turns: post about 1,500 bounded messages to an owned test mission with the development CLI, mixing short lines, multi-paragraph markdown, lists and fenced code, plus one ask card.

- Scrolling the long feed top to bottom stays smooth; typing in the feed composer has no visible lag.
- Opening the mission lands at the bottom. Appends at the bottom stay pinned; appends while scrolled up show the "New messages" pill without moving the view; the pill returns to the bottom.
- Drag-select and Cmd-C inside a message; scroll it away and back and the selection is still there.
- Answering the ask card resizes it without overlapping or gaps around neighbouring blocks. Resizing the window reflows the feed without gaps or overlap.
- Selected regression cases, chosen because the feed's rendering and scroll state change: MIS-FEED-01 (content, authors and pending state survive switching away and back), MIS-FEED-02 (stop and start dividers) and MIS-LIFE-02 (switching Feed and slot tabs leaves no duplicated feed rows).

## Review, verification and authorization

The coder owns implementation and checks. The reviewer waits for an explicit Runner handoff, then reviews the full branch diff against #962 with must-fix findings first and file:line pointers. Focus on: the incremental blocks never diverging from `group_feed_blocks` (especially the `relayed` case), every height change re-measuring its item, the scroll behaviours in deliverable 1 being preserved, no stale markdown cache across a mission switch, and no speculative entity split or abstraction beyond what the measurement justified. Iterate until the reviewer posts `NO REMAINING MUST-FIX ISSUES`.

Run `cargo test --locked -p runner-app --profile ci`, workspace Clippy with warnings denied (`cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`), macOS updater Clippy (`--features updater`), `cargo fmt --all --check`, and `git diff --check`. Record the exact commands and real exit codes; do not pipe a gate through `tail` or `head`. Windows CI has repeatedly failed on imports or helpers used only by `cfg(unix)` tests; gate any such import or helper with `#[cfg(unix)]`.

After a clean review, Jason authorizes rebasing all work on this branch into clean history on top of current `origin/main`, this brief folded into a commit, usually one commit with a subject that names the fix (for example `fix(ui): virtualize the mission feed so long feeds stay responsive`), pushing `fix/962-virtualize-mission-feed`, and opening a PR against main whose body says `Fixes #962`. If main has moved, rebase; never merge main into the branch. Review, QA or CI fixes after the push are folded into the commit they correct and pushed with `git push --force-with-lease`. Drive CI green on macOS and Windows. Do not merge, delete the branch or worktree, or cut a nightly or release. Final Runner handoff: PR URL, what changed, tests and exit codes, CI result, the before/after measurement, the reviewer's verdict, and what Jason should smoke-test (a long feed scrolls and accepts typing smoothly; the pill and stick-to-bottom behave as before). Then stand by.

## Scope amendment — 2026-10-20

After the reviewed #962 candidate reached PR #964, Jason explicitly asked to fold [#947](https://github.com/yicheng47/runner/issues/947) into the same PR. This supersedes the original boundary excluding #947 and its guide change. Implement the existing app scrollbar on the virtual feed as specified in `docs/features/947-mission-feed-scrollbar.md`, using its shared control and preserving the feed layout and interaction contracts. No new scrollbar design or shared-control API change is required. Include `Fixes #947` in the expanded PR body. The same working-tree review, scoped commit/push, macOS/Windows CI and exact-SHA QA workflow applies to this added scope; the PR remains open and must not be merged. QA stopped and released the old candidate before implementation began, retaining its incomplete checkpoint. Review and publish the added feature before handing QA its replacement SHA.

## Performance follow-up — 2026-10-20

Jason reported remaining lag, especially in large post blocks, on `0f454bd1`. The retained fixture groups 1,500 messages into a single virtual row, whose renderer still builds every message. A single long post likewise builds all of its markdown. The alternating-author automated fixture missed these cases. Keep canonical `group_feed_blocks` behavior and add finer presentation rows for individual messages and markdown sections, retaining group appearance, selection positions and scroll anchors. Include same-author and single-long-post work-count regressions and a comparison against `0f454bd1`.

Jason removed the QA slot and requested continued iteration until the native feel is right. The old QA app PID was confirmed absent before editing/building this follow-up. Preserve both incomplete checkpoints and the human-reported performance failure. Keep iterative attempts uncommitted until Jason confirms the behavior works; obtain working-tree review before publishing accepted fixes to the existing PR and rerunning macOS/Windows CI. The coder builds and checks but does not launch or drive the native app; Jason performs these manual trials. No merge is authorized.

Jason subsequently confirmed “visual smoothness passed,” agreed that no QA slot was needed, and explicitly authorized merging after final-candidate CI passes. This supersedes the earlier no-merge boundary. Publish the accepted, cleanly reviewed changes and drive both platform checks green before merging PR #964 with a merge commit. Retain the earlier incomplete QA evidence without claiming a complete regression pass. Confirm clean working trees and refresh the root checkout’s `main` after merge; local branch/worktree deletion remains unauthorized.
