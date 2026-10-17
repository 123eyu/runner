# 748 — Runner delivers mission updates to the chat that started the mission

> Tracking issue: [#748](https://github.com/yicheng47/runner/issues/748). Priority: P1, milestone 0.16 (moved from 0.12 to 0.13 on 2026-10-03, to 0.15 on 2026-10-07 and to 0.14 on 2026-10-08, renumbered 0.16 on 2026-10-10). Platforms: macOS and Windows.
> Status: on hold since 2026-10-17, after [826](./826-targeted-messaging.md). Jason does not want Runner typing mission updates into a normal chat, which is this design's core mechanism, so the return path needs a new design. The proposal below, refreshed that day against the delivery model [#921](https://github.com/yicheng47/runner/pull/921) shipped for #907, is kept for that discussion. No design: nothing new is drawn, and the notices are text typed into the chat's own pane.
> Part of delegating work to other agents ([#562](https://github.com/yicheng47/runner/issues/562), closed 2026-10-16): this is the path that brings a delegated worker's result back to the chat that started it.
> Related: the watch contract from #686 ([test record](../tests/archive/686-watch-cli-started-missions.md)), which this replaces inside Runner chats; [704](https://github.com/yicheng47/runner/issues/704), which first specified `RUNNER_SESSION_ID` and closed unbuilt on 2026-10-11, so this spec carries it; #907's [delivery model](../arch/arch.md#85-delivery-model) and [plan](../impls/907-reliable-delivery.md), which this builds on; [826](./826-targeted-messaging.md), [#916](https://github.com/yicheng47/runner/issues/916) and [#873](https://github.com/yicheng47/runner/issues/873), see [their relation](#relation-to-826-916-and-873).

## Motivation

Codex is Jason's main production tool, and he runs it as a direct chat inside Runner. When such a chat starts a mission, the generated skill requires it to watch the mission. Codex exposes no tool that brings a background process's output into an idle conversation, so the chat reports "automatic watching is unavailable" and Jason has to come back and ask. The chat that started #724's mission did exactly that on 2026-09-28.

Runner does not need Codex to have such a tool. Runner hosts the chat's PTY, already types wake-up lines into mission slots through a delivery gate that respects drafts, and already sees every event the mission writes. So Runner delivers the mission's important events to the chat that started it.

## Findings (2026-09-29)

These findings predate `runnerd` and reliable delivery; the [reconciliation](#reconciled-with-reliable-delivery-2026-10-17) below records what has changed since.

| Checked | Evidence | Consequence |
| --- | --- | --- |
| Jason's production Codex | `codex-tui` 0.158.0 (npm `@openai/codex`) on macOS 26.6.2. Runner's production database has 26 Codex direct chats since 2026-09-20, such as "Investigate issue #730". The #724 initiator (thread `01a0e6c9…`, 0.157.1, root checkout) was a TUI session; its row cannot be matched because a chat's row keeps only its latest thread ID. | Design for a Codex chat that Runner hosts. Codex elsewhere keeps today's limitation path. |
| Tools in the #724 initiator | All 27 calls were Codex code mode's `exec`, which wraps `exec_command`, `write_stdin`, `apply_patch` and web search. `write_stdin` returns output only when the model calls it during a turn. | No Codex tool meets the skill's test: every output line and the exit must reach an idle agent. |
| Codex Stop hook | `hooks` is stable and on. A Stop hook can block the end of a turn and continue it with a prompt (`core/src/session/turn.rs`), with a ten-minute default timeout (`hooks/src/engine/discovery.rs`). | A watch built on it keeps one turn open for the whole mission: the chat shows Working for an hour, and every wait costs a model call. Rejected. |
| Codex app server | Every command Codex runs gets `CODEX_THREAD_ID` (`protocol/src/shell_environment.rs`). The TUI shares a daemon on `~/.codex/app-server-control/app-server-control.sock` only when launched without `-c` overrides (`tui/src/lib.rs`, `can_reuse_implicit_local_daemon`). Runner launches Codex with `-c` hook overrides, so a Runner chat runs an embedded server with no socket. | Starting a turn through the app server cannot reach a Runner chat. It stays a possible route for Codex outside Runner. |
| Runner's input path | Codex slots already answer Runner's typed lines as new turns: the codex pair's rollouts show `[inbox] new message from @reviewer — run runner msg read to view.` as user turns. The gate (`SessionManager::reserve_delivery` and the router's outbox) waits for idle, no draft and no approval prompt, and queues for a stopped session (arch §8.5). | The mechanism exists. It needs a recipient that is not a roster slot. |
| What missions write | Across 85 mission logs in the production data: 912 slot-to-slot messages, 161 to `human`, 129 broadcasts. In #724 the three updates Jason needed (PR opened, final handoff, merged) were all broadcasts, among eight crew messages. | Wake the chat on messages to the person and broadcasts, not on handoffs inside the crew. |

Codex source was read in the local clone at `7f01a84` (2026-09-15), which is older than 0.158.0. Those findings only rule out the alternatives. The chosen design relies on two things already proven in production: Codex starts a turn for a typed line, and the environment of the commands Codex runs carries Runner's variables, as `RUNNER_HANDLE` does for slots today.

## Reconciled with reliable delivery (2026-10-17)

The spec above was written before `runnerd` (0.13.0), the empty mission start (0.16.2) and reliable delivery ([#921](https://github.com/yicheng47/runner/pull/921), after 0.16.4). Checked against `main` at `5de4c249`:

| Changed since 2026-09-29 | Where | Consequence for this spec |
| --- | --- | --- |
| Mission messaging is a durable queue: one dispatcher per mission, readiness owned by the session manager, every attempt Confirmed, Unsubmitted or Unverified, idle redelivery, soft dispatcher state rebuilt from the log | arch [§8.5](../arch/arch.md#85-delivery-model), `router/dispatch.rs`, `SessionManager::delivery_readiness` | The watching chat becomes one more recipient of that dispatcher. The old outbox, its folding and the "never on replay" rule are gone, and so are the parts of this spec built on them. |
| Readiness events go to a list of listeners per session ID | `notify_delivery_event` in `session/manager/mod.rs` | A mission's router can listen on a direct chat that is not in its roster, and two missions can listen on one chat. |
| `runnerd` owns sessions and missions; the app is a client | arch §1, 0.13.0 | Quitting the app no longer stops delivery. A daemon restart replaces the old "Runner quits" case. |
| The person's answers and composer messages are messages from `human`, carrying `source_event`, and answers carry `question_id` | arch §8.1 and §8.5 | The watching chat acts for the person, so its own posts and answers come back as `human` messages. They must neither wake nor be counted. |
| A member cannot post `--to human`; it broadcasts instead | `runner-cli/src/msg.rs`, arch §8.4 | "A message from a slot to `human`" no longer happens. A member reports to the person, and so to the watching chat, by broadcast or `ask --human`. |
| The router cursor is a byte offset; acks still compare IDs, because IDs can arrive out of order (#906's F5) | arch §8.5 | `watch_delivered.up_to` stays a byte offset, as the notice's `feed --since` already expects. |
| A chat delegates by `mission start` with no crew plus `mission add-role` | 0.16.2, [562](./archive/562-mission-spawn.md) | The binding attaches at `mission start`; `add-role` needs nothing. |
| `RUNNER_SESSION_ID` still does not exist; direct chats export only `RUNNER_HANDLE` as a label | `spawn_direct_inner`, arch §9.4 | Unchanged: this spec still adds it. |
| The skill moved to `runner-core/src/protocol/agent_skill.rs`; the backend crate is `runner-daemon`; the guide documents watching | | Paths and test commands below follow. |

## Decision

Runner watches the mission, not the agent. When `runner mission start` or `mission resume` runs in a Runner agent chat, that chat becomes the mission's **watching chat**. The mission's dispatcher types a one-line notice into it for the events that matter, with the same readiness holds, attempt and confirmation as a member's inbox notification (arch §8.5), but only when the chat is idle. The agent answers the notice in a new turn, reads the details with `runner mission feed` and tells the person.

This works for every runtime Runner hosts, not only Codex. It needs no follower process in the agent, no host facility and no change to Codex. A notice waits for the current turn to end, so nothing arrives in the middle of a turn.

Alternatives, in the order the issue asked for:

1. **An existing Codex facility.** None qualifies (findings).
2. **A foreground watch**, either a Stop hook or a `write_stdin` polling loop. Either holds the chat's turn open for the life of the mission and pays a model call per wait. Rejected as the default; the limitation path still offers `feed --follow` in the foreground.
3. **A Codex host integration**: `turn/start` through the app server for `CODEX_THREAD_ID`. It cannot reach a Runner chat, which has no shared socket, and it ties Runner to a protocol that changes weekly. Deferred until Jason starts missions from Codex outside Runner.

## Scope

### Identity: `RUNNER_SESSION_ID`

Every agent session and terminal gets its own session ID in its environment on spawn, resume and fork, as 704 first specified. The CLI reads it only to name the watching chat. It is a delivery address, not a caller: a chat still acts for the person (648 decision 7 and arch §9.3 unchanged), and a direct chat stays off the bus (arch §9.4).

### The watch binding

- `mission_start` and `mission_resume` take an optional `watch_session_id`, which the CLI fills from `RUNNER_SESSION_ID`. Runner accepts it only for a running agent direct chat. It ignores it for a terminal pane, because typing into a shell would run the text, and for a mission slot, whose dispatcher already owns its input. Without an accepted ID there is no watching chat, and the start still succeeds.
- One watching chat per mission. Attaching from another chat moves the watch, and the old chat gets one notice saying where it went.
- The binding lives in the mission log. A `mission_watch` signal from `system` carries `{session_id}`, or null to detach. A confirmed notice appends `watch_delivered` with `{up_to}`, the `next_offset` after the last event it covered. Unlike `inbox_read`, `up_to` is a byte offset, matching the router cursor and `feed --since`. The router rebuilds both on mount, so the watch survives a daemon restart. Both types join the feed's default-hidden noise beside `session_status`, `runner_status` and `inbox_read`, and the app's feed renders neither.
- `runner mission watch <mission> [--off]` attaches the calling chat, or with `--off` detaches whichever chat is watching, through a `mission_watch` client-protocol request. This is how a chat watches a mission it did not start (one Jason started in the app, or one whose watch moved), and how an agent stops watching.

### Delivery through the dispatcher

The watching chat is a recipient of the mission's dispatcher, kept apart from the roster. In arch §8.5's terms:

- **Subscription:** the wake and counted events below, from the binding's `up_to` on. It is keyed by the chat's session ID, never by a handle, so it stays out of routing, the roster, unread counts and the rail's pills.
- **Readiness:** the router registers as a delivery listener on the chat's session while the binding lasts, and reads `delivery_readiness` like any member. `Pending`, `Starting`, `Drafting`, `RecentlyTyping`, `NeedsYou`, `InFlight` and `Unavailable` hold the notice. Unlike a member's notification, a watch attempt also waits for `Ready { idle: true }`, because the chat is the person's own conversation and a notice typed during a turn could join that turn.
- **Notification:** composed when the attempt starts, by reading the log from `up_to`, so a busy chat gets one notice covering everything since the last confirmed one.
- **Ack:** the dispatcher writes `watch_delivered` only after a Confirmed outcome. Unverified leaves the backlog for idle redelivery, which may repeat a notice with the same `--since`. Unsubmitted pauses the watch with the notice still in the chat's input, where the person sees it, until they press Return or clear it. No pill is added to direct chats.
- **Redelivery:** the dispatcher's existing 30-second check and two-minute backoff, idle only.

### What wakes the chat

The router classifies each event past its cursor:

- **Wake:** a broadcast from a member; a `human_question`; a member crash, or a session status with `lifecycle: error` or outcome `failed` (the feed's existing exceptions to noise); a `mission_warning`; and the end of the mission: `mission_stopped`, every session exited, or completed or aborted status.
- **Counted, never waking:** directed messages between members. The next notice reports how many arrived since the previous one.
- **Ignored:** everything from `human`, including the watching chat's own posts and answers, the messages derived from them and their signals; `mission_goal`; and everything the feed hides by default.

### The notice

A notice is one line, typed and submitted by the dispatcher's attempt exactly like an inbox notification: paste, Enter after `SUBMIT_DELAY`, then confirmation. #753's Windows submit delay applies unchanged.

The line leads with the most important pending event: a question, then a failure or the end, then a broadcast. It quotes at most 200 characters of that event with newlines flattened, counts the rest, and ends with the exact command that reads everything undelivered, from the previous `up_to`. Commands use `runner` for the installed app and the quoted absolute sidecar for a development build, as the generated skill does.

```text
[Runner] Mission 01M3KD0A "Fix #724" · @coder to everyone: "Final handoff for #724: PR https://github.com/yicheng47/runner/pull/745 is OPEN, one commit 67620b8…" · +4 crew events · Read: runner mission feed 01M3KD0AK0BR736Q7ZB5G0QJX6 --since 170083 --oldest-first --json
[Runner] Mission 01M3KD0A "Fix #724" · @coder asks the user: "Merge PR #745 now?" (yes / no) · Ask the user, then: runner mission answer 01M3KD0AK0BR736Q7ZB5G0QJX6 <question_id> <choice>
[Runner] Mission 01M3KD0A "Fix #724" ended: every session exited. No more updates. Read: runner mission feed 01M3KD0AK0BR736Q7ZB5G0QJX6 --since 172742 --oldest-first --json
```

### Lifecycle

- **End.** The end notice is the last one, and the binding ends once it is confirmed. `mission resume` from a chat attaches that chat again. A resume from the app leaves the mission unwatched until a chat runs `mission watch`.
- **Partial failure.** A member crash wakes the chat while the other members keep running, and the binding stays.
- **Watching chat stopped or resuming.** `Unavailable` holds the notice and the backlog stays, with the dispatcher's "queued until the session resumes" warning once per stop. It is delivered once the chat is running and idle again. If the chat is archived or deleted, the binding ends with a `mission_warning`.
- **Archive.** Archiving the mission ends the binding without a notice. The router unmounts at archive (`mission_archive_impl`), and archiving is the person's or the agent's own act.
- **The app quits.** Nothing changes: `runnerd` keeps the mission, the chat and the dispatcher running, so notices keep arriving in the chat while no window is open.
- **The daemon restarts.** The mount rebuilds the binding and `up_to` from the log, and wake events after `up_to` go out as one catch-up notice once the chat is live and idle. This is ordinary redelivery under arch §8.5's at-least-once guarantee. It cannot repeat a confirmed notice, because `up_to` moves only on Confirmed.
- **Two missions, one chat.** Each mission's router owns its binding, listens on the chat's session and labels its notices; the session's input gate (`InFlight`) serializes their attempts. Attaching a chat that already watches is a no-op.

### CLI output

The changes are additive only. The JSON result of `mission start`, `mission resume` and `mission show` gains `watch: {session_id, delivery: "runner"}` or `watch: null`. Plain output and `-q` add one stderr line when a chat is attached: `watch: Runner will send this mission's updates to this chat.` stdout, exit codes and `feed --follow` are unchanged, and `mission start` still returns at once.

### Skill, guide and architecture

`crates/runner-core/src/protocol/agent_skill.rs` (both the `runner` and `runner-dev` renders) and `crates/runner-cli/src/help.rs` change together:

- **Rule 5** becomes: after `mission start` or `resume`, check `watch`, or the stderr line with `-q`. If Runner delivers, tell the person the mission is running and its updates will arrive in this chat, and arm nothing. Otherwise use the host's watch facility or the limitation path, as today.
- **A new rule:** a line starting `[Runner] Mission` is Runner delivering a mission this chat watches. Run the command it gives and tell the person what happened in a sentence or two. Relay a question to the person and answer it only with their decision. Do not start a watcher.
- **WATCH FACILITIES and rule 7** now apply only outside a Runner chat. Claude Code's Monitor stays the worked example there.

Guide pages: [Drive Runner from agents](../guide/guides/drive-runner-from-agents.md) says when Runner delivers and when an agent still arms a watch, [Troubleshooting](../guide/guides/troubleshooting.md) covers a watching chat that receives nothing, and `make guide-cli` regenerates the [CLI reference](../guide/reference/cli.md) for `mission watch` and the help text. `docs/arch/arch.md`: §8.5 gains the watching chat as a recipient, §9.4 `RUNNER_SESSION_ID` and the watch, and §9.6 the skill's two paths.

## Relation to 826, 916 and 873

- **[826](./826-targeted-messaging.md).** Either can land first. Its several recipients leave the classifier unchanged, because any message with a recipient counts and only a broadcast wakes. Its Phase 3 also acks for the recipient after a Confirmed attempt: whichever lands first adds that step to the dispatcher, and the other reuses it.
- **[#916](https://github.com/yicheng47/runner/issues/916).** When the lead's verbs add `runner done`, a member's done report becomes a wake event. `wait` stays a blocking call for callers that want one; the watch is the push path.
- **[#873](https://github.com/yicheng47/runner/issues/873).** Nothing here depends on a lead. In a delegated mission without one, the watching chat is the coordinator, and members reach it by broadcast or `ask --human`.

## Out of scope

- **Codex outside Runner**, in another terminal or the Codex app. The limitation path stands, and the app-server route in the findings is the follow-up if that becomes a production path.
- **Delivery in the middle of a turn.** A watch notice waits for idle.
- **A mission slot as the watching chat.** A lead that starts another mission keeps today's rules until #916's outside seats.
- **More than one watching chat per mission**, and any UI for the watch, including a "watched by" caption on the rail.
- **Jason's personal `mission-watch` skill** in the memory repo. It has to learn to skip Monitor when Runner delivers, and is updated alongside this work, outside the repo.

## Decisions to confirm

1. **Claude Code in a Runner chat also uses Runner delivery** instead of Monitor, so there is one rule inside Runner and no duplicate notices. Monitor stays for Claude Code outside Runner. Recommended. The alternative keeps Monitor in Runner chats and has Runner skip Claude Code.
2. **Crew messages are counted but never wake the chat.** Waking on every handoff costs a Codex turn each time; #724 had eight crew messages and three updates that mattered.
3. **One watching chat per mission**, moved by `mission watch`.
4. **Only a Confirmed notice advances the watch.** An Unverified attempt can then repeat a notice after the idle backoff, costing one extra turn, rather than risk losing one. Recommended, as at-least-once matches arch §8.5.
5. **The chat's own traffic is silent.** Messages from `human`, which include the watching chat's posts and answers, neither wake nor count.

## Implementation phases

1. **Backend.**
   - `RUNNER_SESSION_ID`, and `watch_session_id` on start and resume with its validation.
   - The `mission_watch` and `watch_delivered` signals, their rebuild on mount, and their noise filtering.
   - The watch recipient in `router/dispatch.rs`: the delivery listener on the chat's session, the idle-only hold, the classifier, the notice composed from `up_to`, `watch_delivered` on Confirmed, and the catch-up after a restart.
   - The `mission_watch` client-protocol request.
   - Tests, driving the dispatcher on its manual clock with the fake port where possible:
     - A chat attaches; a terminal pane and a slot are refused.
     - A wake event gives one attempt when the chat is idle, and waits while it is busy, drafting or showing an approval.
     - A burst gives one notice, crew messages are counted but not delivered, and messages from `human` are ignored.
     - Confirmed writes `watch_delivered`; Unverified leaves the backlog for redelivery; Unsubmitted pauses until the input clears.
     - Crash, end and archive each behave as specified.
     - A stopped chat keeps its backlog, and an archived chat ends the binding.
     - The restart catch-up delivers only events after `up_to`, once.
     - Two missions deliver into one chat, and a moved watch sends the move notice.
2. **CLI, skill and docs.** The `watch` field and stderr line, `mission watch`, the skill and help rules, the guide pages and the arch sections. Skill and help tests pin the new rules for both renders.
3. **Live checks.** In the development app under the standing authorization in `AGENTS.md`, by the crew's QA slot or Jason; see Verification.

After sign-off, the three phases are one crew mission.

## Verification

The checklist follows the issue's acceptance list. Local live tests use only Codex, Grok or pi since 2026-10-16 ([procedure](../tests/full-smoke-test.md#computer-use-test-runtimes)), so Codex carries them; the Claude Code rows stay Blocked until that runtime is available again. Select [MIS-WATCH-01 and MIS-WATCH-02](../tests/regression/missions.md) and replace their expectations for Runner chats in the test record.

- [ ] Give a fresh Codex chat in Runner the installed skill only, with no instruction to watch, and ask it to start a mission that posts a handoff, asks a yes/no question and finishes. `mission start` returns at once, the chat says updates will arrive here, and no follower runs.
- [ ] After that turn ends, the crew's broadcast and question each arrive as a `[Runner] Mission` user turn in the Codex rollout, followed by the agent's reply, with Jason sending nothing. Record those rollout lines as the proof of delivery.
- [ ] A notice waits while Jason types a draft in the chat, and while the chat works on something else, then arrives once. The agent's answer to the question, sent through `mission answer`, wakes nothing.
- [ ] Killing one member sends a crash notice while the other keeps running. Stopping every session sends the end notice and nothing after it. Archiving ends the watch silently.
- [ ] Quitting the app with sessions kept running, then reopening, shows a notice delivered while no window was open. Restarting `runnerd` with an undelivered broadcast delivers it once after the restart and never repeats a confirmed notice.
- [ ] Two missions from one chat stay labeled and separate. `mission watch` from a second chat moves the watch, with one notice to the first.
- [ ] Claude Code in a Runner chat gets the same notices and arms no Monitor. Claude Code outside Runner still arms Monitor (the 686 recipe).
- [ ] The release and development skills and `runner help agents` agree. `cargo test -p runner-daemon -p runner-cli -p runner-core`, workspace clippy and `make guide-cli` pass.
- [ ] Record the Codex version, macOS version and app build, and every attempt's outcome from the daemon log. Windows (JASONPC) and the other runtimes (pi, Grok, Copilot, TRAE, Antigravity, OpenCode, Cursor Agent) use the same dispatcher and stay marked unverified until each is checked.
