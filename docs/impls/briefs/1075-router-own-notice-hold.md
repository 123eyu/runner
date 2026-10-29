# #1075 — The router's own unsent notice must not hold delivery

Jason approved this mission on 2026-10-29: the `claude duo` crew (`coder`, `reviewer`) plus one `qa` member added to the mission. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-1075-router-own-notice-hold` on `fix/1075-router-own-notice-hold`, based on `origin/main` at `76a04518`. The root checkout stays on `main`. Use the scratch prefix `/tmp/1075-` for every temporary folder.

## Problem

Issue [#1075](https://github.com/yicheng47/runner/issues/1075) and its comment, on Windows with Codex 0.165.0: a mission slot stops receiving messages after a router notice whose Enter did not take, until `session stop` / `session resume`. It happened on a busy slot (held as `Drafting` for 26 minutes) and on idle slots (nothing logged at all). Investigation on `main` found one chain:

1. **The router's leftover text is tracked as a person's draft.** `track_delivery_input` feeds the notice to `InputTracker` like a paste. After the router's Enter, an unchanged composer row turns `Submitted` back into `Drafting` (`observe_composer`, the `submitted` branch, in `crates/runner-terminal/src/input_state.rs`). Readiness then reports `Drafting` (`SessionState::delivery_readiness` → `SessionModel::draft_hold`).
2. **`Unverified` ends the attempt.** `Dispatcher::advance` (`crates/runner-daemon/src/router/dispatch.rs`) clears `wanted` when it types; only `Unsubmitted` pauses. Afterwards the only retry is the reconcile, which needs `Ready { idle: true }`, which a `Drafting` slot never reaches.
3. **A turn end changes nothing.** The `Idle` event only triggers that reconcile; only detach/attach resets the tracker.
4. **Invisible.** Held/released lines are logged only while `wanted` is set; the blocked reason goes only to the app's `router/delivery-blocked` event; `mission_warning` (what `mission show` RECENT WARNINGS reads) is written only for unavailable sessions.

Both were reproduced with throwaway in-process tests: the tracker reports `Drafting` for 26 simulated minutes after an ignored delivery Enter with no person input, and the dispatcher after `Unverified` + `Drafting` makes no attempt and emits no blocked event through 60 idle events, then only "draft" to the app for new messages, with zero `mission_warning`. The trigger (why the Enter did not take, and why the outcome was `Unverified` rather than `Unsubmitted`) is the Windows Enter-suppression class of #753 and is not in scope.

## Outcome

- Text that only the router typed, with no person input since (the tracker's input revision already tells), is never treated as a person's draft. The router knows it as its own unsent notice.
- No outcome other than `Confirmed` ends delivery. While its own unchanged notice is still in the composer, the router presses Enter again at the next opportunity: a turn end, the reconcile or a readiness change, through the existing `resubmit` guard. It never types a second notice on top of its own leftover (#753's accumulation). If the notice is gone and messages remain unread, the normal backoff retry runs.
- A person's draft, including a person's Return that the agent rejected, still holds exactly as today.
- A hold with unread messages that outlasts the redelivery backoff appends one `mission_warning` per hold, naming the `@handle` and the reason in words, so `mission show` shows it. Log the reason whenever the router declines to attempt, so a stall is visible in `runnerd.log`.
- Do not clear composer text with editing keys.

The coder owns the design inside these lines. Update `docs/guide/concepts/missions.md` and its zh-CN copy where the Inbox waiting behaviour or warnings change.

**Non-goals:** hook delivery (#1046, draft PR #1061), Codex app-server (#1012, draft PR #1063), Windows submit timing, and the #1018 rollback path. PR #1045 (#1018) is open and edits the same tracker: stay out of its `Edit`/`Cancel` relocation and rebase onto `main` if it merges first.

## Validation

Name the changed behaviours and the reason for each check before running it.

- **Regression tests:** an ignored delivery Enter leaves a router-owned notice, not a person draft; person input after it makes it a person draft; the dispatcher resubmits its own leftover after `Unverified` and `Unsubmitted` on turn end and reconcile, and never types a second notice over it; a person draft stays held; the warning is written once per hold. The existing `input_state_replay` fixtures, #940 replays and router tests keep passing.
- **Gates:** the CI commands in `AGENTS.md` (Clippy with `--profile ci`, nextest, fmt), with this worktree's own target.

## Live checks (qa)

`qa` uses the development app only, under the standing authorization in AGENTS.md Test Scope: cheap model, low effort, tiny prompts, owned test missions in fresh `/tmp/1075-*` folders, and it archives only what it created. The mission itself runs in the installed Runner: never touch that app, its daemon, its chats or its missions. No Antigravity.

- **Phase A, now, on the baseline:** try to reproduce the stall on `main` (`76a04518`), built from the root checkout without editing it. One approach is a stub agent that echoes typed input after ~200 ms and ignores Enter, set as the dev app's executable for one runtime and restored afterwards; real Codex attempts are fine too. Spend up to 45 minutes. Report to `coder` and the human: Reproduced (exact recipe, the `runnerd.log` lines, `mission show` output) or Not reproduced (what was tried). Phase A blocks nothing.
- **Phase B, on the frozen candidate after the clean review:** the Phase A recipe if it reproduced (it must now deliver or warn); `MIS-MSG-01` with one Codex and one Claude Code slot; `MIS-DRAFT-01` (Codex); the rejected-Return half of `MIS-DRAFT-07` (Codex). Write `docs/tests/1075-router-own-notice-hold.md`.
- Windows `MIS-MSG-03` is left to Jason; list it as not run.

## Handoffs and authorization

`coder` implements and hands the working-tree diff to `reviewer`. After the clean review, `coder` commits locally and hands `qa` the branch, `git rev-parse HEAD` and the checks; `coder` does not edit or build while `qa` holds the candidate. `qa` does not edit source, tests or expectations and does not commit. Failures go back through `reviewer`, and the affected checks are rerun.

After the QA verdict, `coder` adds the QA record, pushes the branch, opens one PR against `main` and drives CI green on every platform. The PR carries the QA verdict as reported, the Windows check as not run, and no `frontend-only` label. Fold this brief and later fixes into the commits they belong to; use `git push --force-with-lease`, never plain force; add no tool co-author trailers. Stop at an open PR with green CI and report the PR URL and final SHA. Do not merge, release, cut a nightly, archive the mission, or delete branches or worktrees. When waiting for a teammate, finish the turn.
