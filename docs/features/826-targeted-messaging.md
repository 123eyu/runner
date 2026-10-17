# 826 — Targeted crew messaging that scales past three agents

> Tracking issue: [#826](https://github.com/yicheng47/runner/issues/826). Priority: P2, milestone 0.16. Platforms: macOS and Windows.
> Status: draft spec from Jason's 2026-10-08 design discussion, refreshed on 2026-10-17 against the delivery model that [#921](https://github.com/yicheng47/runner/pull/921) shipped for #907. No design: nothing new is drawn except the composer's multi-target picker, which follows its existing single-target chip.
> Related: arch §4.2–§4.3 and §8.1/§8.5 (messages, inbox, delivery model), #907's [plan](../impls/907-reliable-delivery.md#relation-to-826), [#753](../tests/archive/753-inbox-delivery.md) (Windows Codex input delivery), [748](./748-mission-watch-delivery.md) (the starting chat's return path, on hold).

## Motivation

Jason found that with more than three agents in a mission, crew messaging gets messy and slow. Everyone being able to see the crew's traffic is fine; the problem is that seeing a message today usually means being woken by it, and every wake-up costs a turn.

## Findings (2026-10-08)

These findings predate reliable delivery; the [reconciliation](#reconciled-with-reliable-delivery-2026-10-17) below records what has changed since.

| Checked | Evidence | Consequence |
| --- | --- | --- |
| Addressing | `runner msg post --to` takes one roster handle (`crates/runner-cli`), and `Event.to` is `Option<String>` (`crates/runner-core/src/model.rs:113`). The composer has one `target` (`crates/runner-app/src/surfaces/mission_composer.rs`). Local logs hold broadcasts whose text starts with `@reviewer` or `@coe`. | Writing to two members needs a broadcast, which wakes everyone. A typed `@handle` without `--to` is a broadcast too. |
| Wake-up | `message_nudge` (`crates/runner-daemon/src/router/handlers.rs:162`) types `[inbox] new message from @X — run runner msg read to view.`; a broadcast nudges every other roster member. | Each message costs the recipient one turn to fetch the body, and a broadcast costs that turn in every slot. |
| Reads | `runner msg read` (`crates/runner-cli/src/msg.rs:83`) parses the whole log from offset 0 and prints every inbox message since the mission began. It records `inbox_read`, and the daemon keeps per-handle watermarks and unread counts (`event_bus/mod.rs`), but the CLI ignores them. | Across 673 reads in the 57 local mission logs, at most 13% of 4,771 printed messages were new; one read printed 36 messages to deliver 1. Agents re-read old instructions and their context fills. |
| Log content | `session_status` is 85–98% of most mission logs, and every `msg read` parses it. | Reads get slower as missions run longer and crews grow. |
| Retry | The reconciliation tick (`router/mod.rs:828`, 30 s interval, 2 min backoff) wakes a live slot only when it is idle, has unread messages and nothing pending, with a generic `[inbox] unread messages` line. | The safety net exists and never interrupts a busy agent; it only needs to send the messages themselves. |
| Prompts | Workers get `WORKER_COORDINATION_PREAMBLE` and the lead gets a Coordination section (`router/prompt.rs`), both describing pull-based reads. Crew conventions repeat delivery mechanics ("Message delivery is push, not pull") and invite standby acknowledgements; in the #782 mission, reviewer and qa each messaged coder within 22 s of the start. | Messaging mechanics belong in the preamble, which ships with the code that implements them, not in each crew's conventions. |

No local mission had more than three agents, so latency above three agents is not measured yet; verification below covers it.

## Reconciled with reliable delivery (2026-10-17)

The findings and decisions were written on 2026-10-08, before reliable delivery ([#921](https://github.com/yicheng47/runner/pull/921), after 0.16.4) replaced the outbox and the reconciliation tick. Checked against `main` at `5de4c249`:

| Changed since 2026-10-08 | Where | Consequence for this spec |
| --- | --- | --- |
| One dispatcher per mission pushes notifications composed from each subscriber's backlog when an attempt starts; redelivery is its 30-second, idle-only deadline with a two-minute backoff | arch [§8.5](../arch/arch.md#85-delivery-model), `router/dispatch.rs`; `message_nudge` now only signals a backlog | Decisions 2 and 3 change what the dispatcher types, not a gate or outbox. The model's own note says this is #826's Phase 3. |
| Every attempt ends Confirmed, Unsubmitted or Unverified | arch §8.5 | "A successful submit advances the read marker" becomes a Confirmed outcome. An Unverified attempt may or may not have landed, so retyping its bodies could make an agent act twice (decision 3). |
| The person's composer messages and answers became messages from `human`, delivered by the pointer line | #907 decision 2, arch §8.1 | "`human_said` already delivers full text" is no longer true. Phase 3 gives the person's messages and answers their text back. |
| A member cannot post `--to human`; `human` is not a subscriber | `runner-cli/src/msg.rs`, arch §8.4 and §8.5 | Decision 1 drops `human` as a recipient. |
| Inside a mission, `msg post` and `msg read` use the event log directly, without the daemon | arch §9.3 | Addressing cannot be resolved only in the daemon; the CLI and the daemon share one resolver. |
| `msg read` still parses the log from offset 0, prints every inbox message since mission start and ignores the ack | `runner-cli/src/msg.rs` | Phase 1 is still needed. It also starts a later member's subscription at its `role_joined`, the CLI fix #907 left here. |
| Start, add-role and respawn compose prompts in one place, and #873 makes the lead optional | `member_prompt_channels`, [#873](https://github.com/yicheng47/runner/issues/873) | Phase 4 edits one composer, and the lead text applies only when there is a lead. |

The findings' line references predate this change and are kept as dated evidence.

## Decisions

1. **Addressing decides who is woken.** A message wakes exactly its recipients. `--to` is repeatable (`--to coder --to qa`). A message posted without `--to` whose text starts with one or more `@handle` tokens naming roster members is addressed to them; mentions later in the text do not address. `human` is not a recipient: `--to human` keeps its error, and a leading `@human` stays text. An unknown handle is an error that names the valid handles. One resolver in `runner-core`, used by the in-mission CLI before it appends, by the daemon's operations and by the composer, keeps the CLI, the composer and `--as` posts the same. The subscription becomes `from ≠ h` and (`to` is empty or contains `h`). The composer's `@` picker allows several targets, and a targeted `human_said` becomes a message to all of them. A message with no recipients is a broadcast and still wakes every other member, because a broadcast is meant for everyone; a broadcast from the person wakes every member.
2. **The notification carries the messages.** When an attempt starts, the dispatcher composes it from the backlog in stream order: `[@coder → you] <text>`, `[@coder → @reviewer, @qa] …` for several recipients, and `[human → you] Answer to "…": yes` for an answer. Readiness holds, attempts while the agent is busy, `SUBMIT_DELAY` and confirmation follow arch §8.5 unchanged. A backlog over the delivery limit is cut at a message or line boundary and ends with `… N more — run runner msg read`, which also gives the input probe a fixed last line to find. After a Confirmed outcome the dispatcher writes `inbox_read { up_to }` for the subscriber, covering only the messages it typed in full.
3. **A body is typed once; repeats are the pointer.** The dispatcher types a message's body at most once while it is mounted. After an Unverified attempt, and on every redelivery, it types today's pointer line (`[inbox] N new messages — run runner msg read to view.`) and leaves the ack to the read, so a body that landed without confirmation is never typed twice and the agent never acts on it twice. Redelivery keeps its interval, backoff and idle-only rule. After a daemon restart the backlog is typed in full once, so only bodies whose attempts were unconfirmed before the restart can repeat. Unsubmitted pauses the subscriber as today, with the bodies still in its input.
4. **Catch-up rides on a real wake-up.** A notification that carries bodies adds a digest of other members' directed messages above the subscriber's ack, which today are visible only to their recipients: one line each (`@coder → @qa: candidate ready …`), truncated, at most 10, then a pointer to `runner msg read --all`. Other members' messages never wake a member, the pointer line carries no digest, and redelivery never sends a digest alone.
5. **Reads return unread by default.** `runner msg read` prints the subscriber's messages after its ack, then the digest of others' traffic; `--all` prints the full history from the subscriber's `role_joined`, or mission start for an original member; `--since` and `--from` keep their meaning. Reading uses the daemon's projection rather than reparsing the whole log where the socket is reachable.
6. **The preamble owns messaging behavior.** `WORKER_COORDINATION_PREAMBLE`, and the lead's Coordination section when there is a lead, say that messages arrive in the terminal, how to address one or several members, to reply to the sender directly, to broadcast only when everyone must act, and not to send acknowledgements or standby messages. After this ships, the `codex duo`, `codex trio` and `claude pair` crew conventions drop their delivery-mechanics paragraphs.

## Non-goals

Agent-side polling; threads or per-message priority; waking a member for other members' traffic; changing the dispatcher's readiness holds, submit delay, confirmation window or redelivery timing; moving `session_status` out of the mission log (#797 covers status transport).

## Implementation phases

1. **Unread-only reads.** CLI default plus `--all`, from the ack the daemon already keeps, with subscriptions starting at `role_joined`. Independent of delivery; it can land at any time.
2. **Addressing.** `to` holds one or several handles, with readers accepting the old single-string form so existing logs stay valid; the shared resolver; repeatable `--to`; the multi-target composer and `human_said`; backlogs and unread counts only for recipients.
3. **Delivery.** Message-carrying notifications, the delivery limit and cut, the ack written after a Confirmed outcome, the pointer for repeats, the digest.
4. **Prompts and docs.** The preamble and lead Coordination text, arch §4.2, §4.3, §8.1 and §8.5 (its Notification and "Who acks" entries), `runner help agents` with `make guide-cli`, the guide pages, and the crew-convention cleanup. Guide pages: [Drive Runner from agents](../guide/guides/drive-runner-from-agents.md) for addressing and reads, and [Missions](../guide/concepts/missions.md) and [First mission](../guide/getting-started/first-mission.md) for the composer's several targets.

## Relation to 748

[748](./748-mission-watch-delivery.md) proposed the chat that started a mission as another recipient of the same dispatcher. It is on hold for a new design since 2026-10-17, so this spec lands first and adds the ack written after a Confirmed attempt itself.

## Verification

- Unit tests: addressing resolution in the CLI and daemon paths (repeatable `--to`, leading mentions, mid-text mentions, unknown handles, `human`); backlogs for direct, multi-recipient and broadcast messages; on the dispatcher's manual clock, bodies typed once, the ack written only after Confirmed, the pointer after Unverified and on redelivery, redelivery only to idle members, the cut and its last line; digest contents, cap and ordering; `msg read` default, `--all`, `--since`, `--from` and the `role_joined` start; old single-string `to` lines still parse.
- Live, in the development app under the standing authorization in `AGENTS.md`, with a four-member Codex crew: a scripted exchange where one member addresses another, then two, then broadcasts, and the person answers a question. Record per member the number of wake-ups and turns, and the time from post to the recipient's turn start, before and after. A member who is not addressed must stay idle, and the answer arrives with its text.
- Delivery limit: multi-line bodies of a few KB into Codex, Grok and pi on macOS, and Codex on Windows (#753's submit delay), confirming each submits as one turn and records Confirmed in the daemon log. Local live tests use only Codex, Grok or pi since 2026-10-16 ([procedure](../tests/full-smoke-test.md#computer-use-test-runtimes)); Claude Code, Copilot and Antigravity stay unverified until they can be run.

## Open questions

- The delivery limit per runtime, and whether it should be one shared value.
- Whether the lead's digest should be uncapped, since the lead coordinates the whole crew.
- The JSON shape of several recipients (`to` as an array versus a separate field) for `mission feed --json` consumers.
- Acks compare IDs while the stream is ordered by offset. A message appended later with a lower ID than a written ack would count as read without being typed (#906's F5); the plan decides whether written acks stop before such a message or acks move to offsets.
