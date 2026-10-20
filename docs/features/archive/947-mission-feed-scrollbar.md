# 947 — Mission feed scrollbar

Tracking issue: [#947](https://github.com/yicheng47/runner/issues/947). Jason folded this feature into #962 and PR #964 on 2026-10-20 after the virtual-feed candidate was reviewed. Work stays on `fix/962-virtualize-mission-feed` in its existing worktree.

## Motivation

The feed accepts wheel input but has no visible position indicator or way to drag through a long history. It should use the existing app scrollbar already used by the sidebar and Settings.

## Scope

Reuse `ScrollbarKind::App` and its 10px gutter, resting/active colors, track click and drag behavior. This is the issue’s fully specified `agent-ready` reuse of an existing control; there is no new visual design or Pencil file. Place it at the feed’s right edge inside the existing horizontal inset so content keeps its width and padding. Hide the thumb when content fits. Wheel input over the gutter scrolls the feed.

#962 replaced the old `ScrollHandle` with `ListState`, so adapt the shared control through `Scrollbar::external` rather than `Scrollbar::app`. Include vertical padding in metrics, freeze the list’s height estimate during a drag, and release it on mouse-up inside or outside the gutter. Whole-feed loads receive lazy uniform height hints so a drag can reach unmeasured history; real row measurements refine the estimate. Changed rows retain their previous height as an estimate while awaiting remeasurement. Newly inserted rows have their missing estimates repaired before the next scrollbar gesture, once per interaction rather than on every append. Ordinary appends keep their incremental update path. Do not eagerly measure the entire feed or change the shared scrollbar’s visuals or API.

Preserve landing at bottom, near-bottom following, human-appended messages, New messages actions, selection/copy, ask-card resizing, zoom and mission switching. Update `docs/guide/concepts/missions.md` for the visible scrollbar.

The performance follow-up uses individual messages and markdown sections as presentation rows inside canonical groups, so scrollbar counts, estimates and anchors follow those finer rows. The shared control remains unchanged. Jason now performs manual smoothness trials after removing the QA slot; iterative fixes stay uncommitted until he confirms the behavior works, then follow working-tree review and the existing PR’s CI gates.

## Implementation phases

1. Adapt virtual-list metrics and scroll writes to the shared app scrollbar, including drag lifecycle and gutter wheel input.
2. Add regressions in the existing mission-workspace harness, run the required local gates and obtain working-tree review before committing/pushing the added feature.
3. Push the reviewed candidate to PR #964, drive macOS/Windows CI green, and hand QA the new exact SHA for the selected live checks plus scrollbar checks. Keep the worktree fixed during QA’s build/app work. Fold follow-up fixes into the commit they correct; do not merge.

## Verification

Check no thumb for fitting content; correct 10px gutter and unchanged content insets; track clicks and drags through unmeasured history; wheel input over the gutter; stable drag metrics with newly measured row heights; release outside the gutter; bottom pinning and New messages while reading history; selection/copy and ask/window resizing. Retain #962’s 2,000-message parse/build counters and incremental-equivalence tests. Run `runner-app` tests, workspace Clippy, updater Clippy, formatting and whitespace checks, followed by macOS/Windows CI. QA’s earlier `17dac27e` checkpoint is incomplete and remains evidence for that candidate only.
