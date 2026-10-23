# PR #1006: native wrapped-link QA

Jason requested one QA session on 2026-10-23 after the inline fix and PR: https://github.com/yicheng47/runner/pull/1006. Validate the behavior users see; this is focused feature QA, not a complete smoke or regression run.

## Candidate and prior checks

Use branch `fix/wrapped-table-links` in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-wrapped-table-links`. Product source is commit `c314da2099afde854f706fe24dc00a8616a15d2a`; the launch brief adds documentation only. Record the actual HEAD and dirty diff before building, prove the source has not changed from that commit, and keep source fixed throughout QA. The coder has stopped all builds and will not edit while you hold the candidate.

The fix groups fragmented OSC 8 hyperlinks by destination and compatible IDs within table cells, retains separate highlight spans around unlinked punctuation, and preserves hyperlink metadata in terminal snapshots and cursor state. Relevant files are `crates/runner-terminal/src/terminal.rs` and `crates/runner-terminal/src/snapshot.rs`. The regression `terminal_links_follow_fragmented_osc_8_in_table_cells` reproduces the reported Neighbour URL, with a split at `www.shopneighbour.com/en` and `-us/` and an adjacent table cell linking to the same destination.

Terminal tests passed (142 passed, one existing ignored measurement), app tests passed, workspace and updater Clippy passed, and formatting passed. GitHub macOS and Windows CI both passed on the source commit. Keep these separate from native evidence; do not rerun full suites without a relevant failure or changed candidate.

## Required native checks

Use `TERM-LINK-01` in `docs/tests/regression/terminal.md` for the URL, wrapping, file-link and reattach behavior affected here. Before running, list the selected variants and their pass criteria. macOS is the required accessible native platform; disclose that Windows native testing is unavailable on this host separately from its green CI.

1. Reproduce the reported layout: a long hyperlink with a linked label, an unlinked opening parenthesis, a visible URL split over several rows in a table narrower than the terminal, and an unlinked closing parenthesis. Include separate anonymous hyperlink IDs within one row, table padding/borders, and another table cell with the same destination. Hover and hold the normal activation modifier over each fragment. The complete intended link highlights consistently; unlinked punctuation, padding, borders and the other cell remain excluded.
2. Activate representative fragments from the first, middle and last rows through native modifier-click. Prove the browser receives the full intended destination. A scratch localhost receiver with the same long path is acceptable for observable activation; retain the original Neighbour display layout. CLI `open` is not proof of activation.
3. Repeat the relevant hover/activation checks after a partial redraw and native detach/reattach or closing/reopening the test session, so snapshot metadata is exercised. Include a hyperlink label whose text is not itself a URL.
4. Confirm separate explicit hyperlink IDs and unrelated rows remain separate, and that an existing scratch file link with line/column still follows the normal editor activation path. Scope this to relevant link regressions.

## Authorization, isolation and evidence

Start validation immediately; this brief and Jason's request are the validation handoff. Follow `AGENTS.md` Test Scope and the applicable candidate identity, isolation, native control, evidence and cleanup sections of `docs/tests/full-smoke-test.md`. Build and run only this candidate in Runner's development namespace. Use the installed Runner CLI only to coordinate this QA mission. Product tests use the absolute development CLI, with mission identity variables removed as documented. Verify the actual executable/hash and development endpoint before typing. Do not operate installed Runner's existing chats or missions, inspect its database, change global configuration or authentication, or stop a development app that was already running without the required authorization.

Prefer a bounded scratch shell fixture; live agent test chats are authorized only where needed under the repository's standing development-app authorization. Use canonical scratch/evidence paths under `/private/tmp`, native computer use for input and screenshots, and a ledger of test-owned objects. A missing native-control prerequisite is Blocked, not a pass. Clean up only test-owned processes/chats/missions and preserve evidence and history.

Do not edit implementation, tests or expectations, add more agents, commit, push, create a PR, merge, release, delete branches/worktrees, or perform destructive cleanup. You may write the QA run record under `docs/tests/runs/2026-10-23-macos.md`, preserving existing content. Report through Runner with must-fix findings first and a Passed/Failed/Blocked/Skipped matrix, concrete reproduction steps, native screenshot/evidence paths, cleanup and remaining limits. Broadcast the final handoff, then end your turn. The original implementer will handle fixes and publishing the evidence.
