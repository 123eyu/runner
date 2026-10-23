# 1007 — Review remembered list page sizes

## Candidate and objective

Review the implementation of issue #1007 on branch `feat/1007-list-page-size` in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/feat-1007-list-page-size`, based on `0696c9e1bb163052ecd2f90f40b7e9f3f5782010`. Capture the candidate commit and working-tree state before validation. Read the complete branch diff against that base, then run the selected native checks only after the code review is clean. Report concrete must-fix findings to the external operator through Runner; do not fix implementation code yourself.

Jason selected server-side pagination with 10, 25 and 50 rows, default 10, remembered separately for Roles and Crews. The approved compact selector reads `10 / page`. Its design is already committed and pushed as `45c085e` in the private design repo, at `runner/desktop/specs/1007-list-page-size.pen`: Roles `FQVg3`, Crews `vGI5F`, selector and narrow footer `A0tnE`. Rendered approved frames are available under `/tmp/runner-1007-approved/`. Open `.pen` files only through Pencil tools.

## Read first

- `AGENTS.md`, especially Test Scope and Crew Missions.
- `docs/features/1007-list-page-size.md` and `docs/tests/1007-list-page-size.md`.
- `docs/tests/full-smoke-test.md` for applicable candidate identity, development isolation, native control and evidence rules; this is a narrow feature review, not a full smoke run.
- The Runner and runner-dev skills at `/Users/jason/.agents/skills/runner/SKILL.md` and `/Users/jason/.agents/skills/runner-dev/SKILL.md`.

## Review and validation

Read the touched code and callers for list requests, invalidation, settings synchronization, footer layout and scrolling, plus the tests. Check that size changes reset page and scroll while preserving search, stale replies cannot overwrite a new request, and both saved preferences remain independent. The daemon already supports page sizes; its protocol is unchanged.

The coder completed `cargo test --locked -p runner-app --profile ci` (800 passed, 3 existing ignored), the focused `page_size` tests after adding a peer-window assertion (4 passed), workspace Clippy with updater, formatting and diff checks. Commands and logs are recorded in the validation document. Do not repeat unchanged broad checks merely for process; run focused checks when a finding warrants them.

Live checks are required because this changes visible controls, scrolling and persisted choices. Run LIST-SIZE-01 through LIST-SIZE-04 from the validation document in the development app, after the code review is clean. Use native computer control for UI assertions and runner-dev CLI state as supporting evidence. Build and run the exact worktree candidate with its own `target/`; never share `CARGO_TARGET_DIR`. Confirm the development app identity before input. Seed only bounded, uniquely marked owned role/crew fixtures, with enough rows to exercise more than 50 results, using the development endpoint. Preserve preexisting data and sessions; do not operate the installed production app or its user data. No extra agent sessions are needed for these checks. Record initial settings and restore any values changed only for testing. Clean up only owned fixtures, without bulk/destructive database operations.

Record candidate identity, environment, commands, screenshots and actual outcomes in `docs/tests/1007-list-page-size.md`. Each selected case must say Passed, Failed, Blocked or Skipped. Preserve failed attempts when documenting successful recovery. A missing native-control prerequisite or unavailable platform is Blocked, not a pass. macOS native validation is the available local platform; Windows CI is separate and is not a native UI pass. Do not expand into unrelated runtime, IME or chat checks.

## Handoff and authorization

Jason authorized one Runner reviewer and an implementation PR. You may review, build, run the selected development-app checks, create and clean up your own bounded fixtures, and edit only the validation record. Do not launch additional agents, delegate, change global configuration, commit, push, open or merge a PR, publish a release, or remove branches/worktrees. Leave code changes and Git writes to the external operator. Send actionable findings and your final verdict through Runner, then finish your turn; do not poll or sleep while awaiting the operator.

The operator will commit and push the implementation, open a PR against `main`, address findings and drive CI green on both platforms, then stop for Jason's review. There is no merge or release authorization. The brief is folded into the implementation commit. Review or CI fixes are folded into the commit they correct, following the repository's focused-history convention, rather than separate address-review commits.
