# 826 — Targeted crew messaging that scales past three agents

> Tracking issue: [#826](https://github.com/yicheng47/runner/issues/826). Priority: P2, no milestone yet. Platforms: macOS and Windows.
> Status: draft spec from Jason's 2026-10-08 design discussion. No design: nothing new is drawn except the composer's multi-target picker, which follows its existing single-target chip.
> Related: arch §4.2–§4.3 and §8.1/§8.5 (messages, inbox, delivery gate), [#753](../tests/archive/753-inbox-delivery.md) (Windows Codex input delivery), [748](./748-mission-watch-delivery.md) (notices typed into a watching chat through the same gate).

## Motivation

Jason found that with more than three agents in a mission, crew messaging gets messy and slow. Everyone being able to see the crew's traffic is fine; the problem is that seeing a message today usually means being woken by it, and every wake-up costs a turn.

## Findings (2026-10-08)

| Checked | Evidence | Consequence |
| --- | --- | --- |
| Addressing | `runner msg post --to` takes one roster handle (`crates/runner-cli`), and `Event.to` is `Option<String>` (`crates/runner-core/src/model.rs:113`). The composer has one `target` (`crates/runner-app/src/surfaces/mission_composer.rs`). Local logs hold broadcasts whose text starts with `@reviewer` or `@coe`. | Writing to two members needs a broadcast, which wakes everyone. A typed `@handle` without `--to` is a broadcast too. |
| Wake-up | `message_nudge` (`crates/runner-daemon/src/router/handlers.rs:162`) types `[inbox] new message from @X — run runner msg read to view.`; a broadcast nudges every other roster member. | Each message costs the recipient one turn to fetch the body, and a broadcast costs that turn in every slot. |
| Reads | `runner msg read` (`crates/runner-cli/src/msg.rs:83`) parses the whole log from offset 0 and prints every inbox message since the mission began. It records `inbox_read`, and the daemon keeps per-handle watermarks and unread counts (`event_bus/mod.rs`), but the CLI ignores them. | Across 673 reads in the 57 local mission logs, at most 13% of 4,771 printed messages were new; one read printed 36 messages to deliver 1. Agents re-read old instructions and their context fills. |
| Log content | `session_status` is 85–98% of most mission logs, and every `msg read` parses it. | Reads get slower as missions run longer and crews grow. |
| Retry | The reconciliation tick (`router/mod.rs:828`, 30 s interval, 2 min backoff) wakes a live slot only when it is idle, has unread messages and nothing pending, with a generic `[inbox] unread messages` line. | The safety net exists and never interrupts a busy agent; it only needs to send the messages themselves. |
| Prompts | Workers get `WORKER_COORDINATION_PREAMBLE` and the lead gets a Coordination section (`router/prompt.rs`), both describing pull-based reads. Crew conventions repeat delivery mechanics ("Message delivery is push, not pull") and invite standby acknowledgements; in the #782 mission, reviewer and qa each messaged coder within 22 s of the start. | Messaging mechanics belong in the preamble, which ships with the code that implements them, not in each crew's conventions. |

No local mission had more than three agents, so latency above three agents is not measured yet; verification below covers it.

## Decisions

1. **Addressing decides who is woken.** A message wakes exactly its recipients. `--to` is repeatable (`--to coder --to qa`). A message posted without `--to` whose text starts with one or more `@handle` tokens naming roster members (or `human`) is addressed to them; mentions later in the text do not address. An unknown handle is an error that names the valid handles. The daemon resolves addressing, so the CLI, the composer and `--as` posts behave the same. The composer's `@` picker allows several targets. A message with no recipients is a broadcast and still wakes every other member, because a broadcast is meant for everyone; a broadcast from the person wakes every member.
2. **The wake-up is the message.** Delivery types `[@coder → you] <text>` (or `[@coder → @reviewer, @qa] …` for several recipients) through the existing gate and outbox, immediately, with no wait for the recipient's turn to end. A body over the delivery limit is cut at a line boundary and ends with a pointer to `runner msg read`. A successful submit advances the slot's read marker to that message. `human_said` already delivers full text this way.
3. **The tick re-sends the messages.** The reconciliation tick keeps its interval, backoff and idle-only rule, but delivers the slot's unread addressed messages and broadcasts with the same format instead of the generic line.
4. **Catch-up rides on a real wake-up.** The read marker covers every mission message, including direct messages between other members, which today are visible only to their recipients. A delivery adds a digest of other members' messages since the slot's last read: one line each (`@coder → @qa: candidate ready …`), truncated, at most 10, then a pointer to `runner msg read --all`. Other members' messages never wake a slot, and the tick never sends a digest alone.
5. **Reads return unread by default.** `runner msg read` prints messages after the slot's read marker, addressed ones first, then the digest of others' traffic; `--all` prints the full history; `--since` and `--from` keep their meaning. Reading uses the daemon's projection rather than reparsing the whole log where the socket is reachable.
6. **The preamble owns messaging behavior.** `WORKER_COORDINATION_PREAMBLE` and the lead's Coordination section say that messages arrive in the terminal, how to address one or several members, to reply to the sender directly, to broadcast only when everyone must act, and not to send acknowledgements or standby messages. After this ships, the duo, pair and trio crew conventions drop their delivery-mechanics paragraphs.

## Non-goals

Agent-side polling; threads or per-message priority; waking a slot for other members' traffic; changing the delivery gate's draft, submit and cooldown timings; moving `session_status` out of the mission log (#797 covers status transport).

## Implementation phases

1. **Unread-only reads.** CLI default plus `--all`, from the watermark the daemon already keeps.
2. **Addressing.** `to` holds one or several handles, with readers accepting the old single-string form so existing logs stay valid; leading-`@` resolution in the daemon; repeatable `--to`; multi-target composer; nudge only recipients.
3. **Delivery.** Message-carrying wake-ups, delivery limit and fallback, read-marker advance on submit, tick re-delivery, catch-up digest.
4. **Prompts and docs.** Preamble and lead Coordination text, arch §4.2/§4.3/§8.1/§8.5, `runner help agents`, and the crew-convention cleanup.

## Verification

- Unit tests: addressing resolution (repeatable `--to`, leading mentions, mid-text mentions, unknown handles, `human`); nudge targets for direct, multi-recipient and broadcast messages; read-marker advance only after a successful submit; tick re-delivery only to idle slots with unread addressed mail; digest contents, cap and ordering; `msg read` default, `--all`, `--since`, `--from`; old single-string `to` lines still parse.
- Live, on the development build with a four-slot crew: a scripted exchange where the lead addresses one member, then two, then broadcasts. Record per slot the number of wake-ups and turns, and the time from post to the recipient's turn start, before and after. A member who is not addressed must stay idle.
- Delivery limit: multi-line bodies of a few KB into Codex, Claude Code, Copilot, pi and Antigravity on macOS, and Codex on Windows (#753's paste detection), confirming each submits as one turn.

## Open questions

- The delivery limit per runtime, and whether it should be one shared value.
- Whether the lead's digest should be uncapped, since the lead coordinates the whole crew.
- The JSON shape of several recipients (`to` as an array versus a separate field) for `mission feed --json` consumers.
