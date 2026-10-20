# Review #965: filtered message read feedback

## Goal

Review the uncommitted implementation of [#965](https://github.com/yicheng47/runner/issues/965). The spec is `docs/features/965-inbox-read-feedback.md`. Return concrete correctness, regression or scope findings with file and line references, or a clear approval with any residual validation gaps.

## Candidate

Worktree: `/Users/jason/repos/yicheng47/runner/.worktrees/feat-965-inbox-read-feedback`.

Branch: `feat/965-inbox-read-feedback`, based on `0d7ae68d`. Review `git diff HEAD` and the untracked feature spec and this brief. The root checkout stays on main; do not relocate or create a checkout.

## Review focus

- Filtered reads retain existing message selection and acknowledgement behavior while reporting the full unread inbox count on stderr. Empty selected results still produce a hint if unread messages remain; a consumed inbox does not.
- Counts respect the join boundary, latest valid acknowledgement, stale daemon cursor and log fallback, and exclude sent messages, signals and other recipients' traffic. The count describes the captured batch.
- Successful unfiltered reads still acknowledge only printed inbox messages; output failure and messages arriving after capture remain protected.
- The generated worker, team, lead and agent-skill guidance is consistent with the command, CLI help and user guide. Receipt-only replies are distinct from consuming the inbox.
- Stdout and exit behavior remain compatible. No JSON response format, repeated-query tracking or native UI behavior is added.

## Validation

The implementer runs the message-read unit tests, CLI read roundtrip tests, agent-skill tests, router prompt tests, spawn goldens, CLI help goldens, generated CLI reference, format checks and Clippy for runner-cli, runner-core and runner-daemon. Review the coverage and run focused checks if a concrete concern requires them. Reuse `--profile ci`, keep this worktree's own target directory and cap build jobs at 12.

Do not launch the app or live test agents. Native delivery and Windows testing are outside this CLI-feedback change.

## Authorization and handoff

Jason explicitly requested inline implementation followed by one Runner review mission using Claude Opus 5.5 with xhigh effort. This is a review-only mission: do not edit product code or golden expectations, commit, push, create a PR, merge, release, start additional agents or clean up branches/worktrees. The implementation and brief remain uncommitted pending Jason's separate git-write request. This review scope overrides the default implementation-mission PR workflow.

Write the review to `/tmp/runner-965-review.md`, then post the verdict and any must-fix findings through `runner msg post`. Use unfiltered `runner msg read` for inbox notifications. End your turn after reporting; do not send standby or receipt-only messages.

## Outcome

Claude Opus 5.5 at xhigh reviewed the candidate in mission `01M5BZE5JRPSM6DZ0SASB2N2KW` on 2026-10-20. The sole must-fix was an unchanged help-text unit test asserting the old wording. The implementer updated those assertions; the subsequent full runner-cli suite passed 217 tests with two existing manual benchmarks ignored, and formatting and diff checks passed. The reviewer found no other blocking issues in counting, acknowledgement behavior or guidance.

Jason subsequently authorized committing, opening the PR, merging after CI and cleaning up the review mission and merged worktree. This follow-up authorization applies to the implementer; the review-only role's boundaries above remain unchanged.
