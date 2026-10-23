# 1007 — Rows per page for Roles and Crews

Tracking: [#1007](https://github.com/yicheng47/runner/issues/1007).

## Decision

Jason selected option B on 2026-10-23: server-side pagination with a 10 / 25 / 50 selector, defaulting to 10. He approved the compact `10 / page` control instead of the separate label and boxed number. Roles and Crews remember their sizes independently in the existing UI settings file.

Design: `runner/desktop/specs/1007-list-page-size.pen` in the private design repo, committed as `45c085e`. Roles is `FQVg3`, Crews is `vGI5F`, and selector details and the narrow footer are `A0tnE`. The main product canvas remains the shipped design until this feature ships.

## Behavior

- A new installation or existing settings file without these preferences starts both lists at 10 rows per page.
- The footer's compact select offers 10, 25 and 50. Its tooltip reads “Rows per page”; its menu marks the current choice and supports the existing select's mouse and keyboard interactions.
- Changing the size returns to page 1, scrolls to the top, retains the current search, and requests that page and size from the daemon. Late replies from the previous request cannot replace the new page.
- The choice survives searches, route changes and app restarts. Other open windows synchronize the preference for that list.
- Long pages scroll within the table. The column header and footer stay visible, and changing pages starts at the top.
- The footer keeps its total or search-match count and existing windowed pager. Below 480 logical pixels of table width, the count and selector share the upper footer row and the pager is centered underneath.
- Empty lists keep their existing empty state. A search with no matches keeps its count and size selector.

## Validation and guide

Automated coverage includes request size, page-count math, stale replies, search preservation, settings compatibility and persistence, actual role/crew page loads, selector keyboard and mouse operation, scrolling, and narrow/wide footer bounds. Run runner-app tests, workspace Clippy and formatting checks. Native checks cover the two lists' selectors, large-page scrolling, resize and saved preference; these remain a separate reviewer check under AGENTS.md.

Guide pages: `docs/guide/concepts/roles.md` and `docs/guide/concepts/crews.md`.

[Validation results and remaining native checks](../tests/1007-list-page-size.md).
