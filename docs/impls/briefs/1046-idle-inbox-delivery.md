# #1046 — Deliver inbox notices only to idle members and cap draft holds

Jason requested a `codex trio` mission for [#1046](https://github.com/yicheng47/runner/issues/1046) on 2026-10-30. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-1046-idle-inbox-delivery` on `feat/1046-idle-inbox-delivery`, based on `origin/main` at `2ce6c3a404a0698006a04ff3be72130fbe089304`. The root checkout stays on `main`. Use this worktree's own `target/`; do not set a shared `CARGO_TARGET_DIR`. Leave the older `feat-1046-hook-inbox-delivery` worktree (merged #1061) alone.

## Problem

Notices typed into a working agent sit unsubmitted (#753, #1075); screen-read draft holds never expire (#940, #1018); an input-method composition the app never finishes holds delivery until Submit / clear.

The spec is [`docs/features/1046-hook-inbox-delivery.md`](../../features/1046-hook-inbox-delivery.md), revised 2026-10-30 (c8feadc5); read it in full before starting. Where this brief and the spec disagree, the spec wins. Its "Later: native channels" section is out of scope.

## Read first

- [`docs/tech/mission-message-delivery.md`](../../tech/mission-message-delivery.md) — the implemented delivery path.
- `crates/runner-daemon/src/router/dispatch.rs` — `DeliveryReadiness::Ready { idle }`, the reconcile path and the new-message path that start attempts.
- `crates/runner-daemon/src/session/state/mod.rs` — `draft_hold`, `last_local_input_at`, `unsent_notice`, `RECENT_LOCAL_INPUT_WINDOW`, `native_input` (pi's `EditorDraft`) versus `observed_input`.
- `crates/runner-terminal/src/input_state.rs` — `InputTracker`: `submitted_form`, `unsent_notice`, `delivery_owned`, `current_reported_state` and the `Composing` handling.
- `crates/runner-terminal/src/terminal/model.rs` — the resubmission guard on the input revision.
- `set_composing` in `crates/runner-terminal/src/terminal.rs`, `crates/runner-app/src/terminal_ime.rs`, `terminal_observe_input` in `crates/runner-core/src/protocol/api.rs` (no connection identity today), and `TerminalController` / `take_control` in the daemon.

## Deliverables

1. **Idle only.** Every delivery attempt requires `DeliveryReadiness::Ready { idle: true }`, including the new-message path. A message arriving mid-turn stays queued; the existing Idle transition requests delivery. Startup, Needs you, in-flight and draft holds are unchanged.
2. **Ten-minute draft cap.** A screen-observed `Drafting` holds only while `last_local_input_at` is under ten minutes old; with no keystroke since attach it does not hold. A composition update counts as a keystroke. pi's native `EditorDraft` keeps holding uncapped. Inbox waiting and the two-minute feed warning stay as today. Use a fixed constant; no setting.
3. **Runner's own notice, by text.** Whether the composer holds Runner's notice comes from its text matching `submitted_form`, not from the input revision: moving the cursor keeps it Runner's (label stays **Message not submitted — press Return**); a key that changes the text makes it the person's draft. Runner presses Return only while the revision is unchanged, and Submit / clear keeps the revision as its reset guard. The cap never lifts a hold while the composer holds Runner's notice; the dispatcher's pause on a seen leftover wins.
4. **Orphaned compositions.** The daemon records which client connection started a composition and clears it, emitting `InputCleared` to the router, when that connection closes or terminal control moves to another client. The app retries `Composing { composing: false }` after a failed request.
5. **Tests** on the manual clock, covering every bullet in the spec's Verification list.
6. **Docs.** The Missions guide in English and zh-CN (`docs/guide/concepts/missions.md`, `docs/guide/zh-CN/concepts/missions.md`), the delivery walkthrough §5 and §8, [architecture §8.5](../../arch/arch.md#85-delivery-model), and `docs/tests/regression/missions.md`: update MIS-MSG-01 (drop "Working alone is not a hold"; a notice to a Working member waits for idle), MIS-MSG-03 and MIS-DRAFT-01 for the cap, and add a row for the orphaned-composition check. Update the spec's status line to say it is implemented. Leave archived docs alone.

Out of scope: hooks, `asyncRewake`, app-server or any native delivery channel; status detection; a Settings row for the cap; unrelated refactors.

## Validation

Before running checks, list the changed behaviors and the check that covers each. Expected selection: `runner-daemon` router and session-state tests, `runner-terminal` input-state and terminal-model tests, the daemon server/terminal-control tests for connection close and control changes, and `runner-app` tests for the IME retry. Then `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, `cargo fmt --all --check` and `git diff --check`. No full smoke or regression expansion.

Live checks after the clean code review, run by `qa` in the development app under AGENTS.md Test Scope (`make run`, inexpensive models, low effort, tiny prompts, fresh `/tmp` folders; archive only the chats and missions qa created):

- **MIS-MSG-01** (updated): a message to a Working member is typed only after its turn ends, then read and acknowledged once.
- **MIS-DRAFT-01**: a Claude and a Codex draft hold delivery and stay intact. For the cap, build the candidate with the constant temporarily shortened (for example 30 seconds), never committed; restore it, confirm `git status` is clean and record the value used.
- **MIS-MSG-06**: messages during a long first turn wait for it and arrive once.
- **New**: with a Chinese input method, start a composition in an idle Claude member, quit the development app before committing it, post a message to that member from the CLI; the notice arrives without Submit / clear and the composition text never reaches the composer.
- **MIS-MSG-03** needs a Windows host. Run it only if one is available; otherwise record it as not run and leave it to Jason.

`qa` records results in `docs/tests/1046-idle-inbox-delivery.md` with event IDs and evidence, marking any check it could not run and why. A failed check goes back to `coder` with the evidence.

## Roles and handoffs

`coder` implements and runs the checks, then hands the uncommitted diff to `reviewer` through Runner. `reviewer` posts must-fix findings first with file:line pointers; iterate until clean. The clean review confirms the live checks above or names changes. `coder` then commits and pushes, and hands the candidate SHA to `qa`. Coder and reviewer never run the app. After qa passes, `coder` folds the test record into the commits, opens the PR and drives CI.

## Authorization

Jason authorized this `codex trio` mission (one coder, one reviewer, one qa) with implementation, validation, commits on this branch, pushing it and opening one PR against `main`, then driving the required CI green. `qa` may use the development app as above. No other agents, missions or internal subagents.

Fold this brief into an implementation commit. Shape the history as you judge best; the three changes may stand as separate commits (idle-only delivery, the draft cap with text-based ownership, orphaned compositions), each building and passing its tests. Fold later review, qa or CI fixes into the commit they correct and push with `git push --force-with-lease`; never plain force. Rebase onto `origin/main` only when AGENTS.md requires it; never merge `main` into the branch. Do not add tool-specific co-author trailers. The PR closes #1046, carries `core-review` (the workflow adds it; never remove it) and one risk label chosen from the diff (expect `risk-high`: it changes when every mission notice arrives), and lists which live checks ran and which were left to Jason.

Stop at an open PR with green CI and report through Runner with the PR URL, final SHA, changes and validation results. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. When waiting for a teammate or Jason, finish the turn and rely on the pushed inbox notification; do not start a feed watcher or polling loop.
