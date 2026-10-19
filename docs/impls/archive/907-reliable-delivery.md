# 907 — reliable mission message delivery: plan

Implementation plan for [#907](https://github.com/yicheng47/runner/issues/907). The issue serves as the spec: this is an internal change whose only user-visible effects are the pill shown for an unsubmitted delivery and how the person's answers reach an agent (decisions 2 and 8). It comes before [#826](https://github.com/yicheng47/runner/issues/826), whose direct delivery builds on it (see [Relation to 826](#relation-to-826)). Related: [#873](https://github.com/yicheng47/runner/issues/873), which makes the lead optional; decision 2 removes one of the lead fallbacks it lists. Line numbers are as of `19d438d1`.

## Delivery model

Mission messaging is a small durable queue with push delivery. This section names its parts in standard message-queue terms; the decisions, the code and the tests below use these terms, and architecture §8.5 takes this section in the commit that adds the dispatcher. Names already in the log (`inbox_read`, `up_to`) do not change.

| Term | In Runner | Code or log name |
| --- | --- | --- |
| Stream | The mission's event log: append-only, with one total order, the order of the file, and each append under `flock`. | `events.ndjson`, `EventLog` |
| Producer | Anything that appends: the `runner` CLI, the daemon's mission operations, the router. | `EventLog::append` |
| Message | An event with `kind: "message"`. Signals are not messages and are not queued for subscribers. | `EventKind::Message` |
| Subscriber | One mission member, addressed by its handle. The person is not a subscriber; a message to `human` is in no subscription. | roster handle |
| Subscription | The messages a subscriber receives: `from ≠ h` and (`to` is null or `to = h`), starting at the subscriber's `role_joined`, or at mission start for an original member. | bus `RoleInbox` |
| Ack | A cumulative acknowledgement written by or for the subscriber: every message in its subscription with an ID at or below `up_to` is consumed. Acks only move forward; the bus ignores a stale one. | `inbox_read { up_to }` signal |
| Backlog | The subscription's messages after the latest ack. | bus unread count (`InboxUpdate.unread_count`) |
| Dispatcher | The per-mission loop that pushes notifications to subscribers with a backlog. | `router/dispatch.rs` (new) |
| Notification | The line that tells a subscriber it has a backlog. It carries no message, is composed at send time from the backlog, and is idempotent. | `[inbox] N new messages — run runner msg read to view.` |
| Delivery attempt | Typing one notification into the subscriber's terminal, submitting it, and confirming that it was taken. | `Attempt` |
| Readiness | Flow control: whether the subscriber's session can take input now. The session manager owns it. | `SessionManager::delivery_readiness` |
| In-flight limit | One attempt per subscriber at a time. | — |
| Redelivery | A new notification to an idle subscriber whose backlog is still there, after a backoff. | the reconciliation deadline |
| Router cursor | The router's own position in the stream, as a byte offset. The router acts only on events past it. | `AppendedEvent.next_offset` |

Guarantees:

- **Durability.** A message is durable once appended, and no message is held only in memory, so a session exit, a stop or a daemon restart loses none.
- **At-least-once notification.** A subscriber with a backlog is notified until it acks. A duplicate notification is harmless; a lost one is repaired by redelivery.
- **Order.** A subscriber reads its messages in stream order, and its attempts are serial.
- **Soft dispatcher state.** Everything the dispatcher holds is rebuilt from the stream on mount: backlogs from acks, the cursor from the log's end.
- **Who acks.** Today the subscriber acks by reading, through `runner msg read`. #826's Phase 3 makes a confirmed attempt that carries the messages themselves an ack written for the subscriber.

Not in this model: partitions, consumer groups, exactly-once delivery, per-message acks, dead-letter queues and priorities. A subscriber whose attempt is unsubmitted is paused, not dead-lettered: its backlog stays until a later attempt is confirmed or it acks.

## What happens today

| Problem | Where | Effect |
| --- | --- | --- |
| 1. Attempts are never confirmed | `start_reserved_delivery` (`router/mod.rs` 1063) types the line through `inject_reserved` (`session/manager/output.rs` 479); a detached thread (1105) sleeps `SUBMIT_DELAY` (80 ms, 300 ms on Windows for #753), presses Enter and ignores the result. | An Enter the agent drops leaves Runner's text in its input, where it blocks every later attempt. #906 hit this while Codex was starting its MCP servers. |
| 2. Readiness in three copies | The session manager's `SessionModel`, `handle.startup` and the gate's `in_flight` (`session/manager/mod.rs` 540, 1056); the router's `status` map rebuilt from `session_status` rows through the bus (up to 500 ms late), its `pending_sessions` and `live_sessions`, and the outbox's `submit_in_flight` (`RouterState`, 285). | Redelivery decides "idle" from the late copy (`inject_delivery_at`, 660), and the two in-flight flags must agree. |
| 3. Answers bypass the stream's guarantees | `human_said` and `human_response` (`router/handlers.rs` 49, 105) type text into the agent's terminal instead of appending a message. Replay never re-sends (`handle_event`, 547), and the `Exited` arm of `session_delivery_event` drops the outbox with a warning. | A lost answer is gone. A lost notification is not: the backlog survives and redelivery repairs it. |
| 4. Timing on ad-hoc threads | One thread per attempt (1105), per retry (1262), per flush (1295) and per cooldown (1323), all cancelled through one `retry_generation`; one reconciliation thread per mission (902). | Scheduling one silently cancels another, and the lock order between router state, the gate and session state is documented only in comments. |
| 5. Duplicated logic | `inject_delivery_at` (660) and `flush_outbox` (1126) each map the eight `DeliveryReservation` states, and both check for stale notifications. A fresh start (`ops/mission.rs` 687–807) and a reopen (`ensure_mission_router_mounted`, 933) build the router differently. `reconstruct_from_log` (459) counts a question only when it finds its `ask_human`; `count_pending_asks_from_log` (`ops/mission.rs` 1325) counts every unanswered `human_question`. The member prompt is composed at start (`ops/mission.rs` 640), add-role (`ops/mission/roles.rs` 179) and respawn (`session/manager/spawn.rs` 2734), each repeating the no-lead goal rule. | Each copy drifts from the others. |
| 6. The router cursor is an event ID | `handle_event` skips events whose ULID is at or below a high-water mark (547). | Wrong when a later append carries a lower ID, as #906's QA fixture showed (F5). The bus already gives every event its byte offset. |
| Cleanups | `remove_roster_member` removes `last_reconciliation_nudge` by session ID (397), but the map is keyed by handle. The module header and `StdinInjector` docs still describe `ask_lead`. `runner msg read` starts every subscription at mission start, not at `role_joined`. | A removed handle keeps its backoff entry; the comments mislead. The CLI fix belongs to #826's Phase 1. |

One layer has already moved: #906 put startup readiness into the session manager (`handle.startup`, `session/manager/startup.rs`), and `reserve_delivery` reports `Starting` from it. That is where decision 3 wants all readiness.

## Decisions

1. **The dispatcher pushes notifications, not messages.** Per subscriber it keeps only whether a notification is wanted (the backlog is above zero), the attempt in flight, a pause, and the redelivery backoff. The notification is composed when the attempt starts, from the current backlog, so the coalescing and the stale-notification checks go: an attempt for a subscriber whose backlog has dropped to zero is not made. #826's Phase 3 changes only what the attempt types.
2. **The person's answers become messages** (the issue's open question; Jason confirmed it on 2026-10-16). The `human_response` handler appends a message from `human` to the asker, `Answer to "<prompt, cut to 80 characters>": <choice>` with `payload.question_id`, instead of typing `[human_response] <choice>`; the signal stays the card's record. `human_said` becomes a message from `human` to `payload.target`, or to every subscriber when there is no target, which drops its fallback to the lead. The feed hides a message carrying `question_id`, because the card already shows the answer. Answers then have the stream's guarantees, and the router types nothing into an agent except notifications. The cost is one `runner msg read` for the agent to see an answer, until #826's Phase 3.
3. **The session manager owns readiness.** `SessionManager::delivery_readiness(session_id)` returns `Ready { idle }`, `Pending` (registered, not spawned yet), `Starting`, `Drafting`, `RecentlyTyping(remaining)`, `NeedsYou`, `InFlight` (the gate is held) or `Unavailable` (stopped or exited). It is `reserve_delivery`'s current logic plus `Pending`, for which `register_mission_session` records the session before its PTY spawns, and `idle` from `SessionModel::activity()`. Attempts stay allowed while the agent is busy, because the TUIs queue typed input; only redelivery waits for idle. Readiness changes reach the dispatcher as the existing `SessionDeliveryEvent`s plus `Idle`, sent when the model's activity turns idle. Approval and bridge releases and raw Enter/Ctrl-C writes use `ReadinessChanged` rather than claiming `InputCleared`, so they cannot lift an Unsubmitted pause. Injected notifications are tracked with their generated submissions by the terminal composer without advancing the local-input revision, with generated submission bookkeeping kept internal; only observed composer clears or native editor-clear events provide `InputCleared` evidence after a resize. The router drops its `status` map, `pending_sessions`, `live_sessions`, `submit_in_flight` and the `session_status` handler, and `synthesize_wake_busy` reads the model instead. Which runtimes have hooks stays with the runtime adapters (`startup_delivery_uses_hooks`).
4. **One dispatcher per mission.** `router/dispatch.rs` holds the `Dispatcher`, which owns every subscriber's dispatch state and handles its inputs in order: `Backlog { handle }` when a message lands in a subscription, `Session { session_id, event }` for readiness changes, attempt finished, exit and respawn, `RosterChanged`, `Stop`, and the deadlines it set itself. It is given `now`, calls the session manager through the existing `StdinInjector` port, and returns its next deadline. A small driver thread per mission feeds it from a channel with `recv_timeout` until that deadline. It replaces the attempt, retry, flush and cooldown threads, `retry_generation`, and the reconciliation thread. Redelivery is a deadline in the dispatcher with the current 30-second check, 2-minute backoff per subscriber and idle-only rule, and an `Idle` event runs the same check at once. Tests drive the `Dispatcher` directly with a fake port and a manual clock, without sleeps. Lock order: the dispatch state belongs to the driver thread; the router's remaining state (roster, session map, backlogs, pending asks) sits behind one mutex that is never held across a port call; the session manager keeps its gate-then-session-state order.
5. **Every attempt ends in one of three outcomes.** The driver types the notification, presses Enter after `SUBMIT_DELAY`, releases the gate as today, and then watches for `CONFIRM_WINDOW` (1.5 s to start, tuned in live checks).
   - **Confirmed:** the runtime reported `AgentEvent::TurnStarted` (Claude Code, Codex, Copilot, pi, Antigravity and Grok report it), or the input probe shows the typed text has left the input. The probe is new in `runner-terminal`, beside `InputTracker`, and uses the same row-diff technique: snapshot the visible rows before typing, find the rows where the typed text's last line appeared, and after Enter check whether those rows still show it.
   - **Unsubmitted:** the probe still shows the text after one more Enter and a second window. The dispatcher pauses the subscriber, makes no attempt into that session, reports it blocked with reason `unsubmitted`, and resumes when the input clears because the person pressed Return or cleared it.
   - **Unverified:** the probe could not find the typed text after typing, or local input arrived during the window. The attempt counts as made, as every attempt does today, and is logged. This is safe because notifications are idempotent: if it did not land, the backlog remains and redelivery repeats it.
   The synthetic busy status is published after a Confirmed outcome, so it never confirms an attempt itself.
6. **The router cursor is a byte offset.** `reconstruct_from_log` records the offset after the last entry it read; `RouterSubscriber::appended` passes `AppendedEvent.next_offset` to `handle_event`, which skips events at or below it. Acks keep comparing IDs, because `up_to` is the CLI's contract.
7. **One of each.** One `mount_mission_router(state, mission, Mount::Fresh(pending) | Mount::Reopen)` serves `mission_start_impl_with_size` and `ensure_mission_router_mounted`. One `pending_asks(entries)` serves `reconstruct_from_log` and `count_pending_asks_from_log`: it counts every unanswered `human_question`, as the feed shows them, and pairs an asker when its `ask_human` is found. One `member_prompt_channels(adapter, member, roster, crew, log)` serves start, add-role and respawn.
8. **What the person sees.** The delivery pill keeps "Inbox waiting (N)" while readiness holds an attempt for a draft or an approval, and reads "Message not submitted — press Return" for a paused subscriber; `DeliveryBlockedEvent` gains `reason`. The copy changes but no frame is needed; Jason confirmed the wording on 2026-10-16. The `mission_warning` "queued until the session resumes" stays, once per stop. "Dropped N deferred deliveries" goes, because the dispatcher holds no messages to drop.

The issue suggests items 1 and 2 first. This plan builds readiness (2), then the dispatcher (3), then confirmation (1), because confirmation needs timers, and adding them to today's thread-and-generation code is the pattern item 3 removes. #906's startup hold already covers the case that raised the issue, so nothing urgent waits on the reorder. The issue's question about runtimes without status is answered by decision 5: the probe confirms on every runtime, so the startup grace for output-only runtimes (10 s plus the 2 s idle window) stays as it is.

## Where it lands

| Area | Change |
| --- | --- |
| `router/dispatch.rs` (new) | `Dispatcher`, its inputs, `Attempt` and its outcomes, deadlines, the driver thread, `CONFIRM_WINDOW`. |
| `router/mod.rs` | `Router` keeps the roster, session map, backlogs and pending asks; loses the outbox, timers, status and session sets, and `reconcile_inbox_at`. `RouterRegistry` starts and stops the dispatcher instead of the reconciliation thread. Header and `StdinInjector` docs rewritten in the model's terms. |
| `router/handlers.rs` | `message_nudge` sends `Backlog` per subscriber; `human_said` and `human_response` append messages (decision 2); the `session_status` handler goes. |
| `session/manager/` | `delivery_readiness`, `Pending` for registered mission sessions, the `Idle` event, probe calls, busy published after a Confirmed outcome. |
| `runner-terminal` | The input probe next to `InputTracker`, exposed through `TerminalModel`. |
| `ops/mission.rs`, `ops/mission/roles.rs` | `mount_mission_router`, `pending_asks`, `member_prompt_channels`. |
| `runner-app` | The pill's `reason`; the feed hides answer messages that carry `question_id`. |
| `docs/arch/arch.md` | §8.5 becomes the Delivery model section above; §7's startup replay describes the router cursor; §8.1's handler table and §8.3 step 3 follow decision 2. No guide page names the pill or the warnings, so the guide does not change. |

## Commits

1. `fix(router): track the router cursor by log offset and clear removed handles' backoff` — decision 6 and the cleanups.
2. `refactor(session): make the session manager the readiness owner` — decision 3.
3. `refactor(router): dispatch notifications through one loop per mission` — decisions 1 and 4, and arch §8.5's model.
4. `fix(router): confirm delivery attempts and resubmit an ignored Enter` — decisions 5 and 8.
5. `refactor(mission): one router mount, ask reconstruction and member prompt` — decision 7.
6. `feat(mission): deliver the person's answers as messages` — decision 2.

Each commit builds and passes its tests, and carries the arch text for what it changes.

## Tests

- **Dispatcher, on a manual clock with a fake port.** A backlog for a ready subscriber produces one attempt whose Enter follows `SUBMIT_DELAY`. Backlog events that arrive while an attempt is in flight produce one later attempt with the current count, and no attempt is made once the backlog is zero. Each readiness hold (`Pending`, `Starting`, `Drafting`, `RecentlyTyping`, `NeedsYou`, `Unavailable`) defers the attempt and releases on its matching event. Redelivery reaches only idle subscribers with a backlog, at most once per backoff; `Idle` runs the same check; removing a subscriber clears its backoff. An exit keeps the backlog, and a respawn starts an attempt.
- **Outcomes.** `TurnStarted` gives Confirmed; a cleared probe gives Confirmed; a probe still showing the text gets exactly one more Enter and then gives Unsubmitted, which pauses the subscriber with its reason until the input clears; an unfound probe or local input during the window gives Unverified and logs it.
- **Probe.** `runner-terminal` tests on recorded screens for each runtime's input, including Claude Code and Codex, which echo the submitted prompt in the transcript above the input.
- **Readiness.** Session manager tests for each state and for the `Pending` → `Starting` → `Ready` events.
- **Router cursor.** An event appended later with a lower ID is still handled; events before the reconstruct offset are skipped.
- **Asks.** The router and the sidebar count the same open questions, including a question whose `ask_human` is missing.
- **Answers (decision 2).** `human_response` appends a message from `human` to the asker with `question_id`; `human_said` with and without a target; the feed hides the answer message; old logs with typed answers still render.

## Live checks

On the development app under the standing authorization, with inexpensive models and low effort in a fresh `/tmp` folder:

- Add a role with a task to an empty mission and to a crew mission on Codex with real MCP startup, and on one runtime without hooks (OpenCode or Cursor Agent). The task arrives with no manual nudge.
- Type into an agent's input while a message arrives; the attempt waits and lands after the draft is sent.
- Answer a question card, then restart `runnerd` before the asker reads it; the answer still reaches the asker (decision 2).
- Restart `runnerd` mid-mission with a backlog; nothing is lost, and nothing already acked is notified again.
- Codex on Windows for #753's paste detection on JASONPC, or record it as unverified.
- Read the daemon log for the run's attempt outcomes; an Unverified outcome names a runtime whose probe needs work.

An ignored Enter cannot be produced on demand, so the Unsubmitted path is covered by the dispatcher and probe tests.

## Relation to 826

#826's Phase 3 changes the notification into the messages themselves and writes an ack for the subscriber on a Confirmed attempt; it needs decisions 1, 4 and 5. Its Phase 1 (unread-only `runner msg read`, with subscriptions starting at `role_joined`) does not touch delivery and can run alongside this. Its Phase 2 (several recipients) changes `message_nudge`, which this plan rewrites, so it follows this one.

## Risks

- **The probe misreads an input**, for example a TUI that redraws its input elsewhere or echoes the submitted text into the same rows. Mitigation: recorded-screen tests for every runtime, a second Enter only when the exact typed text is still in the same rows, and an Unverified outcome when unsure, which behaves as today.
- **A second Enter after an accepted first one** submits an empty input. That is a no-op in the TUIs Runner hosts as far as is known; the live checks watch for it.
- **The rail's busy pill appears later**, by up to the confirmation window, because the synthetic busy waits for a Confirmed outcome.
- **Answers cost the agent one read** until #826's Phase 3 (decision 2).
- **Threads:** one driver thread per mission replaces the reconciliation thread one for one; the per-attempt and per-retry threads go.

## Mission shape

The codex crew (coder and reviewer; QA removed by Jason on 2026-10-16) in `.worktrees/feat-907-reliable-delivery` on `feat/907-reliable-delivery`, ending in one pull request against `main` with CI green on both platforms and no merge. This mission runs no live checks or QA record/matrix; Jason runs the plan’s live checks in a separate session. After clean working-tree review, coder makes the six commits, runs automated validation, opens the PR and drives CI green. The brief is [`briefs/907-reliable-delivery.md`](https://github.com/yicheng47/runner/blob/5e153e10771235eef40d17d558963286d76426ed/docs/impls/briefs/907-reliable-delivery.md).
