# 907 — reliable mission message delivery

Implement [#907](https://github.com/yicheng47/runner/issues/907): mission messaging becomes a small durable queue with one dispatcher per mission, readiness owned by the session manager, and confirmed delivery attempts. Jason asked for this codex trio mission on 2026-10-16, ending in an open PR; starting it authorizes the implementation, review and open-PR workflow below. Jason updated the scope on 2026-10-16: QA is removed, this mission runs no live checks, and Jason runs them in a separate session. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-907-reliable-delivery`, on `feat/907-reliable-delivery`, based on `main` `19d438d1`. No other branch, worktree or target directory. If you need a newer main, rebase onto `origin/main`; never merge it in.

## What matters most

No message is lost and no agent is left with Runner's text stuck in its input. A role added with a task gets it without a manual nudge, on Codex with real MCP startup and on a runtime without hooks. The person's answer reaches the asker even across a `runnerd` restart. A person typing in a pane is never interrupted. Everything else follows the plan; edge cases get the plan's handling and no more.

## Read first

- `AGENTS.md`: Worktrees, Test Scope, Crew Missions.
- **`docs/impls/907-reliable-delivery.md`: the contract.** Its Delivery model section defines every term this mission uses (stream, subscription, ack, backlog, dispatcher, notification, attempt, readiness, redelivery, router cursor) and names the code each maps to. Use those terms in code, comments, tests and commit messages. Decisions 1–8, where each change lands (line numbers as of `19d438d1`), commits, tests, live checks and risks. It wins over this brief on any detail. Decisions 2 and 8 are confirmed.
- Issue #907, and its comment linking #873: add no new rule that depends on the lead.
- `docs/arch/arch.md` §4.3, §7 and §8.

## Deliverables

One PR against `main` in the plan's six commits, each building and passing its tests, with the plan and this brief folded into the first:

1. The router cursor as a byte offset, and the cleanups (decision 6).
2. The session manager as the readiness owner (decision 3).
3. One `Dispatcher` per mission in `router/dispatch.rs`, replacing the attempt, retry, flush, cooldown and reconciliation threads and `retry_generation` (decisions 1 and 4); arch §8.5 becomes the plan's Delivery model section.
4. Confirmed attempts with the input probe and the three outcomes, and the pill's `reason` (decisions 5 and 8).
5. One router mount, one pending-ask reconstruction, one member prompt composer (decision 7).
6. The person's answers and `human_said` as messages, and the feed hiding answer messages (decision 2).

Do not add `docs/tests/907-reliable-delivery.md` or a QA matrix. The PR records automated checks with exit codes and states that live checks are pending with Jason. Update the regression rows whose expected result changes (MIS-ASK-02's answer now arrives as an inbox message).

Easy to get wrong:

- The `Dispatcher` takes `now` and returns its next deadline; tests never sleep. Keep all timing out of the session manager's locks.
- The synthetic busy status must not count as confirmation; publish it after a Confirmed outcome.
- The probe must find the input by row diff, not by the cursor: Claude Code and Codex echo the submitted prompt in the transcript above the input.
- A second Enter only when the exact typed text is still in the same rows; when unsure, the outcome is Unverified.
- `inbox_read` and `up_to` keep their names and ID comparison; only the router's own cursor moves to byte offsets.
- Old logs with typed `[human_response]` answers and `human_said` signals still replay and render.
- `#753`'s `SUBMIT_DELAY` (300 ms on Windows) stays.

## QA

Jason removed the QA slot and all mission live checks on 2026-10-16. After `NO REMAINING MUST-FIX ISSUES`, coder proceeds directly to the authorized six commits, runs the Validation commands below with exit codes, pushes, opens the PR and drives macOS and Windows CI green. There is no QA handoff, live pass, QA record or QA matrix in this mission.

Live checks remain pending with Jason in a separate session: ADD-1 on Codex with real MCP startup and OpenCode or Cursor Agent, DRAFT-1, ANSWER-1, RESTART-1, LOG-1, MIS-MSG-01/04/06, MIS-DRAFT-01, MIS-WAIT-01, updated MIS-ASK-02, MIS-ADD-01 and MIS-EMPTY-01. Native Windows MIS-MSG-03 is also pending with Jason. The plan retains their procedures; this mission does not claim their results. Recorded probe tests cover Claude Code, Codex and zsh; other runtimes’ probe behavior remains unverified pending Jason’s live checks.

## Validation

Log each to a file and read the exit code from the command itself, never through `tail` or `grep`: `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`; `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`; `cargo fmt --all --check`; `git diff --check`. Gate imports and helpers used only by `#[cfg(unix)]` tests with `#[cfg(unix)]`.

## Boundaries

The coder and reviewer never run the development app. No slot operates Jason's production Runner, its chats or missions; this mission runs there. No additional agents, crews or subagents; this mission runs no live test missions.

## Review and authorization

The reviewer waits for an explicit Runner handoff, then reviews the whole diff against the plan and this brief, must-fix first with file:line pointers. Must-fix: anything that can lose a message or an answer, leave Runner's text in an agent's input without a pause and a pill, type into a pane while the person drafts, or sleep in a test. Check especially the dispatcher's inputs and deadlines, the probe and its second Enter, readiness's `Pending` and `Idle`, the cursor's offsets, and old logs replaying. Iterate until `NO REMAINING MUST-FIX ISSUES`, then proceed to commits and automated validation.

After clean review, Jason authorizes: the six commits on current `origin/main` with imperative subjects and no co-author trailers; `git push --force-with-lease` of `feat/907-reliable-delivery`; `gh pr create --base main` with `Closes #907`, the summary, checks with exit codes, review follow-ups and the live checks pending with Jason (including Windows) and no agent session links; and `gh pr checks <n> --watch` until macOS and Windows are green, folding each fix into the commit it corrects after the reviewer checks it. Do not merge, delete the branch or worktree, or cut a nightly or release.

Final handoff to everyone: PR URL and CI, changes, checks with exit codes, the live checks pending with Jason, the review verdict, follow-ups and what is left for Jason. Then all slots stand by.
