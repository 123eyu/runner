# 1046 — Mission inbox delivery

> Tracking issue: [#1046](https://github.com/yicheng47/runner/issues/1046).
> Status: implemented 2026-10-30. Runner keeps composer delivery and makes three changes to it: notices are typed only into idle members, a draft hold expires ten minutes after the person's last keystroke, and an input-method composition that the app can no longer finish stops holding delivery. Native channels that avoid the composer are later work.
> Related: [#940](https://github.com/yicheng47/runner/issues/940), [#1018](https://github.com/yicheng47/runner/issues/1018) and [#1075](https://github.com/yicheng47/runner/issues/1075) (holds and leftover notices), [#753](https://github.com/yicheng47/runner/issues/753) (unsubmitted notices in a busy Codex), [#1012](https://github.com/yicheng47/runner/issues/1012) (closed), [#861](https://github.com/yicheng47/runner/issues/861) and [#1051](https://github.com/yicheng47/runner/issues/1051) (status), [748](./748-mission-link.md).

## Motivation

The router types ``[inbox] N new messages — run `runner msg read` to view.`` into a mission member's composer and presses Enter. Two screen readers guard it: the draft hold keeps a notice out of a composer the person is typing in, and the submission probe checks that Enter took. The [delivery walkthrough](../tech/mission-message-delivery.md) follows the implemented path.

The delivery bugs fall into two classes:

- **Notices typed into a working agent.** A notice typed into Codex during a turn can sit unsubmitted in its composer. #753 accumulated unsent notices in a busy Codex on Windows; #1075 left Runner's own notice in a busy Codex composer, which then read as a draft and held delivery past the turn's end.
- **Draft holds that never expire.** A composer learned from the person's typing can keep reading as a draft after it is empty. #940 and #1018 held delivery indefinitely behind an empty Codex composer.

The 2026-10-29 draft of this spec replaced composer delivery with native channels for every runtime and allowed no typed path. #1012 was closed on 2026-10-30, so Codex has no channel that can wake an idle session, and that replacement cannot ship. Jason's decision on 2026-10-30: keep composer delivery, and remove the two bug classes above.

A review of the draft guard on 2026-10-30 found no timer behind any hold except the two-second recent-typing hold. The tracker rereads the composer only when the terminal prints output or the person types, and the router's 30-second reconciliation reuses its last verdict, so a wrong verdict on a quiet idle screen stays until something happens. The same review found the two further problems handled below: an input-method composition can hold delivery indefinitely, and any keystroke turns Runner's own leftover notice into what reads as the person's draft.

### Other products

Checked on 2026-10-30:

- **Orca** (`17828c78f1`) delivers its terminal mailbox pointer as a paste and a delayed Enter, only when the agent is settled (idle). Since its #33429 (2026-10-25) it holds for a draft only when the person typed into the pane within the last ten minutes and the screen shows a composer draft; the cap bounds "an abandoned or misread" draft. A hook-reported turn start confirms submission for Claude, Codex and Antigravity. Its structured chat sessions, including Codex on its own `codex app-server` child, take mailbox pointers as chat messages with no composer.
- **herdr** (`ee5bfb50`) pastes a prompt and presses Enter on an explicit `herdr agent prompt`. It refuses a blocked, undetected or starting agent, and has no draft check.
- **t3code** (`c8a606c7b7`) has no TUI. A send starts an idle thread with `turn/start`, steers an active Codex turn with `turn/steer`, or queues behind a turn it cannot steer.

## Changes

### 1. Deliver only to idle members

Today every attempt needs `DeliveryReadiness::Ready`, but only periodic redelivery requires its `idle` flag: a new message can start an attempt while the recipient is working ([`router/dispatch.rs`](../../crates/runner-daemon/src/router/dispatch.rs)). Every attempt now requires `Ready { idle: true }`. A message that arrives during a turn stays queued, and the session's existing Idle transition requests delivery when the turn ends.

Idle comes from the session's status as today: hooks, OSC 7501, or quiet output for a runtime without either. Startup, Needs you, in-flight and draft holds apply unchanged.

The cost is that a notice no longer reaches an agent in the middle of a long turn; it arrives when the turn ends. Mid-turn delivery through hooks is later work (below).

### 2. Expire the draft hold ten minutes after the person's last keystroke

A `Drafting` observation from the terminal composer holds delivery only while the person's last keystroke into that session is less than ten minutes old. After that the hold lifts, and the next eligible attempt types the notice. A draft that is misread, as in #940 and #1018, then delays delivery by at most ten minutes instead of blocking it.

- The keystroke time is the session's existing `last_local_input_at`. A `Drafting` observation with no keystroke since the session attached does not hold.
- An input-method composition writes no bytes until it is committed, so a composition update also counts as a keystroke. Otherwise a person who starts composing after ten quiet minutes would have the notice typed into the open composition.
- A runtime's own draft report, pi's `EditorDraft`, is not a misread and keeps holding without the cap.
- **Inbox waiting** shows during the hold, and a hold longer than two minutes adds its one feed warning, as today.

The tradeoff: a real draft left untouched for ten minutes receives the notice after its text, and Enter submits both. Orca accepts the same tradeoff for the same reason.

#### Runner's own notice

A notice Runner typed stays Runner's only until the person presses any key, arrows and Esc included: every observed input advances the tracker's input revision, and ownership is a match against the revision at the router's Enter (`InputTracker::delivery_owned` in [`input_state.rs`](../../crates/runner-terminal/src/input_state.rs)). From then on the leftover notice reads as the person's `Drafting`, the slot shows **Inbox waiting** instead of **Message not submitted — press Return**, and Runner no longer presses Return on it, because the resubmission guard requires an unchanged revision ([`terminal/model.rs`](../../crates/runner-terminal/src/terminal/model.rs)).

The revision answers two questions today, and this change separates them:

- **Whose text is in the composer** comes from the text. The composer holds Runner's notice while its row reads exactly as it did at Runner's Enter: the `submitted_form` the tracker already records and `unsent_notice` already compares. Moving the cursor leaves the notice Runner's, so the slot keeps **Message not submitted — press Return**. A key that changes the text, such as Up recalling an earlier prompt or Esc starting Codex's rollback, leaves the person's draft in the composer.
- **Whether Runner may press Return** keeps the revision guard. Once the person has pressed any key, Runner presses none on its own, and the person submits the notice when the label asks.
- **Submit / clear** keeps the revision as its reset guard.

The cap must not lift a hold while the composer still holds Runner's notice, or the next attempt types a second notice after the first and Enter submits both. The cap applies only to a `Drafting` observation whose text is not Runner's notice, and the dispatcher's existing pause on a seen leftover takes priority over it.

### 3. Clear an input-method composition the app can no longer finish

While an input method composes text, as in Chinese or Japanese input, the tracker reports `Drafting` whatever the screen shows (`InputTracker::current_reported_state`). Only the app's `Composing { composing: false }` clears it, sent when the composition is committed or cancelled ([`terminal_ime.rs`](../../crates/runner-app/src/terminal_ime.rs)). If the app quits, crashes or loses its daemon connection mid-composition, the message is never sent; if that one request fails, `set_composing` does not retry it ([`terminal.rs`](../../crates/runner-terminal/src/terminal.rs)). The member then shows **Inbox waiting** until **Submit / clear**, a respawn or a daemon restart.

- The daemon remembers which client connection started a composition and clears the flag when that connection closes or when control of the terminal moves to another client.
- The app retries `Composing { composing: false }` after a failed request.

The cap in change 2 would also end this hold, but only ten minutes after the person's last input; clearing the flag ends it when the app goes away.

### Unchanged

Submission confirmation (a new turn or the cleared notice confirms; one extra Enter on positive evidence that the notice remains), the `Unsent` hold and its resubmission, Needs you and startup holds, the two-minute redelivery backoff, `runner msg read` and its read acknowledgment.

## Later: native channels

These remain the direction for removing composer delivery, runtime by runtime:

- **While working:** Claude Code and Codex 0.165 hooks can add context after a tool (`PostToolUse`) and keep a turn going with a reason (`Stop`). Either could deliver a notice mid-turn without touching the composer.
- **While idle:** Claude Code's `asyncRewake` hook can wake an idle session. Codex needs a Runner client connection to an app-server, which needs the TUI attached with `--remote` to a server Runner starts ([#1012](https://github.com/yicheng47/runner/issues/1012), closed). The [#1012 hand test](https://github.com/yicheng47/runner/blob/b01059ac08b6a9df9d64645e4f9c9c08540583f3/docs/tests/1012-codex-app-server.md) showed `turn/start` from a second client leaving a TUI draft untouched on Codex 0.165.0.

The previous revision of this spec (`git show 12b25f0c:docs/features/1046-hook-inbox-delivery.md`) holds the delivery contract each channel must meet, the shared offer bookkeeping and the Claude idle-coverage questions.

## Verification

In-process router and session-state tests, on the manual clock:

- A new message to a working member starts no attempt; the Idle transition starts one.
- A `Drafting` observation with a keystroke under ten minutes old holds; at ten minutes the hold lifts and the attempt proceeds.
- A `Drafting` observation with no keystroke since attach does not hold.
- A composition update after ten quiet minutes holds again.
- pi's `EditorDraft` holds past ten minutes.
- Runner's leftover notice keeps new typing paused past ten minutes. After the person moves the cursor it is still labelled as Runner's, and Runner presses no Return on it; after a key replaces its text, the composer reads as the person's draft.
- A composition started over one client connection is cleared when that connection closes and when control moves to another client, and the router receives `InputCleared`.

Live checks in the development app, from the [missions regression suite](../tests/regression/missions.md):

- **MIS-MSG-01**, updated: a message posted while the recipient is Working is typed only after its turn ends, then read and acknowledged once. Its current expectation, "Working alone is not a hold", changes.
- **MIS-DRAFT-01**: a Claude and a Codex draft hold delivery and stay intact. Run the cap with a shortened value in a development build, rather than waiting ten minutes.
- **MIS-MSG-03** on Windows, updated: notices sent during Codex tool windows wait for idle and none accumulates in the composer.
- **MIS-MSG-06**: messages during a long first turn still wait for it to end and arrive once.
- **New:** with a Chinese input method, start a composition in an idle Claude member, quit the development app before committing it, and post a message to that member from the CLI. The notice is delivered without **Submit / clear**, and the composition text never reaches the composer.

Run the router and session crate tests and workspace Clippy.

## Affected documentation

In the implementing pull request: [Missions](../guide/concepts/missions.md) and its zh-CN copy, which describe the draft hold and must say that notices reach a member when it is idle, that a draft hold ends ten minutes after the last keystroke, and that a composition left open when the app quits no longer holds delivery; the [delivery walkthrough](../tech/mission-message-delivery.md) §5 and §8; [architecture §8.5](../arch/arch.md#85-delivery-model); and the regression rows above.

## Open questions

1. **A setting for the cap?** Recommendation: a fixed ten minutes. A row in Settings → Missions would need a design, translations and guide text for a value few people would change; it can be added if someone needs another value.
2. **Ten minutes** follows Orca. A shorter cap delivers sooner after a misread hold and submits abandoned drafts sooner.