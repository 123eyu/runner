# 748 — Runner delivers mission updates to the chat that started the mission

> Tracking issue: [#748](https://github.com/yicheng47/runner/issues/748). Priority: P1, milestone 0.16 (moved from 0.12 to 0.13 on 2026-10-03, to 0.15 on 2026-10-07 and to 0.14 on 2026-10-08, renumbered 0.16 on 2026-10-10). Platforms: macOS and Windows.
> Status: spec, waiting for Jason's comments. No design: nothing new is drawn, and the notices are text typed into the chat's own pane.
> Part of [#562](https://github.com/yicheng47/runner/issues/562), delegating work to other agents, since 2026-10-11: this is the path that brings a delegated worker's result back to the chat that started it.
> Related: the watch contract from #686 ([test record](../tests/archive/686-watch-cli-started-missions.md)), which this replaces inside Runner chats; [704](./archive/704-session-send.md), which first specified `RUNNER_SESSION_ID` and closed unbuilt on 2026-10-11, so this spec carries it; [562](./562-mission-spawn.md), whose outside seats can later use the same delivery.

## Motivation

Codex is Jason's main production tool, and he runs it as a direct chat inside Runner. When such a chat starts a mission, the generated skill requires it to watch the mission. Codex exposes no tool that brings a background process's output into an idle conversation, so the chat reports "automatic watching is unavailable" and Jason has to come back and ask. The chat that started #724's mission did exactly that on 2026-09-28.

Runner does not need Codex to have such a tool. Runner hosts the chat's PTY, already types wake-up lines into mission slots through a delivery gate that respects drafts, and already sees every event the mission writes. So Runner delivers the mission's important events to the chat that started it.

## Findings (2026-09-29)

| Checked | Evidence | Consequence |
| --- | --- | --- |
| Jason's production Codex | `codex-tui` 0.158.0 (npm `@openai/codex`) on macOS 26.6.2. Runner's production database has 26 Codex direct chats since 2026-09-20, such as "Investigate issue #730". The #724 initiator (thread `01a0e6c9…`, 0.157.1, root checkout) was a TUI session; its row cannot be matched because a chat's row keeps only its latest thread ID. | Design for a Codex chat that Runner hosts. Codex elsewhere keeps today's limitation path. |
| Tools in the #724 initiator | All 27 calls were Codex code mode's `exec`, which wraps `exec_command`, `write_stdin`, `apply_patch` and web search. `write_stdin` returns output only when the model calls it during a turn. | No Codex tool meets the skill's test: every output line and the exit must reach an idle agent. |
| Codex Stop hook | `hooks` is stable and on. A Stop hook can block the end of a turn and continue it with a prompt (`core/src/session/turn.rs`), with a ten-minute default timeout (`hooks/src/engine/discovery.rs`). | A watch built on it keeps one turn open for the whole mission: the chat shows Working for an hour, and every wait costs a model call. Rejected. |
| Codex app server | Every command Codex runs gets `CODEX_THREAD_ID` (`protocol/src/shell_environment.rs`). The TUI shares a daemon on `~/.codex/app-server-control/app-server-control.sock` only when launched without `-c` overrides (`tui/src/lib.rs`, `can_reuse_implicit_local_daemon`). Runner launches Codex with `-c` hook overrides, so a Runner chat runs an embedded server with no socket. | Starting a turn through the app server cannot reach a Runner chat. It stays a possible route for Codex outside Runner. |
| Runner's input path | Codex slots already answer Runner's typed lines as new turns: the codex pair's rollouts show `[inbox] new message from @reviewer — run runner msg read to view.` as user turns. The gate (`SessionManager::reserve_delivery` and the router's outbox) waits for idle, no draft and no approval prompt, and queues for a stopped session (arch §8.5). | The mechanism exists. It needs a recipient that is not a roster slot. |
| What missions write | Across 85 mission logs in the production data: 912 slot-to-slot messages, 161 to `human`, 129 broadcasts. In #724 the three updates Jason needed (PR opened, final handoff, merged) were all broadcasts, among eight crew messages. | Wake the chat on messages to the person and broadcasts, not on handoffs inside the crew. |

Codex source was read in the local clone at `7f01a84` (2026-09-15), which is older than 0.158.0. Those findings only rule out the alternatives. The chosen design relies on two things already proven in production: Codex starts a turn for a typed line, and the environment of the commands Codex runs carries Runner's variables, as `RUNNER_HANDLE` does for slots today.

## Decision

Runner watches the mission, not the agent. When `runner mission start` or `mission resume` runs in a Runner agent chat, that chat becomes the mission's **watching chat**. The mission's router types a one-line notice into it for the events that matter, through the same gate as slot nudges: only when the chat is idle, has no unsent draft and has no approval prompt waiting. The agent answers the notice in a new turn, reads the details with `runner mission feed` and tells the person.

This works for every runtime Runner hosts, not only Codex. It needs no follower process in the agent, no host facility and no change to Codex. A notice waits for the current turn to end, so nothing arrives in the middle of a turn.

Alternatives, in the order the issue asked for:

1. **An existing Codex facility.** None qualifies (findings).
2. **A foreground watch**, either a Stop hook or a `write_stdin` polling loop. Either holds the chat's turn open for the life of the mission and pays a model call per wait. Rejected as the default; the limitation path still offers `feed --follow` in the foreground.
3. **A Codex host integration**: `turn/start` through the app server for `CODEX_THREAD_ID`. It cannot reach a Runner chat, which has no shared socket, and it ties Runner to a protocol that changes weekly. Deferred until Jason starts missions from Codex outside Runner.

## Scope

### Identity: `RUNNER_SESSION_ID`

Every agent session and terminal gets its own session ID in its environment on spawn and resume, as 704 first specified. The CLI reads it only to name the watching chat. It is a delivery address, not a caller: a chat still acts for the person on the bus (648 decision 7 and arch §9.3 unchanged).

### The watch binding

- `mission_start` and `mission_resume` take an optional `watch_session_id`, which the CLI fills from `RUNNER_SESSION_ID`. Runner accepts it only for a running agent direct chat. It ignores it for a terminal pane, because typing into a shell would run the text, and for a mission slot, whose router owns its input and its record. Without an accepted ID there is no watching chat, and the start still succeeds.
- One watching chat per mission. Attaching from another chat moves the watch, and the old chat gets one notice saying where it went.
- The binding lives in the mission log. A `mission_watch` signal from `system` carries `{session_id}`, or null to detach. Each delivery appends `watch_delivered` with `{up_to}`, the `next_offset` after the last event it covered, the way `inbox_read` records the inbox. The router rebuilds both on replay, so the watch survives an app restart. Both types join the feed's default-hidden noise beside `session_status`, `runner_status` and `inbox_read`, and the app's feed renders neither.
- `runner mission watch <mission> [--off]` attaches the calling chat, or with `--off` detaches whichever chat is watching, through a `mission_watch` socket tool. This is how a chat watches a mission it did not start (one Jason started in the app, or one whose watch moved), and how an agent stops watching.

### What wakes the chat

The router classifies each event it appends or receives:

- **Wake:** a message from a slot to `human` or to everyone; a `human_question`; a slot crash, or a session status with `lifecycle: error` or outcome `failed` (the feed's existing exceptions to noise); a `mission_warning`; and the end of the mission: `mission_stopped`, every session exited, or completed or aborted status.
- **Counted, never waking:** slot-to-slot messages, `human_response`, `human_said` and `mission_goal`. The next notice reports how many arrived since the previous one.
- **Ignored:** everything the feed hides by default.

### The notice

A notice is one line, pasted and submitted through the gate exactly like an inbox nudge: a bracketed paste, then Enter. The #754 paste-burst handling on Windows applies. While a notice waits at the gate, later events fold into it (latest wins), so a busy chat gets one notice covering everything since the previous one.

The line leads with the most important pending event: a question, then a failure or the end, then a message to the person, then a broadcast. It quotes at most 200 characters of that event with newlines flattened, counts the rest, and ends with the exact command that reads everything undelivered. Commands use `runner` for the installed app and the quoted absolute sidecar for a development build, as the generated skill does.

```text
[Runner] Mission 01M3KD0A "Fix #724" · @coder to everyone: "Final handoff for #724: PR https://github.com/yicheng47/runner/pull/745 is OPEN, one commit 67620b8…" · +4 crew events · Read: runner mission feed 01M3KD0AK0BR736Q7ZB5G0QJX6 --since 170083 --oldest-first --json
[Runner] Mission 01M3KD0A "Fix #724" · @coder asks the user: "Merge PR #745 now?" (yes / no) · Ask the user, then: runner mission answer 01M3KD0AK0BR736Q7ZB5G0QJX6 <question_id> <choice>
[Runner] Mission 01M3KD0A "Fix #724" ended: every session exited. No more updates. Read: runner mission feed 01M3KD0AK0BR736Q7ZB5G0QJX6 --since 172742 --oldest-first --json
```

### Lifecycle

- **End.** The end notice is the last one, and the binding ends with it. `mission resume` from a chat attaches that chat again. A resume from the app leaves the mission unwatched until a chat runs `mission watch`.
- **Partial failure.** A slot crash wakes the chat while the other slots keep running, and the binding stays.
- **Watching chat stopped or resuming.** The notice waits and is delivered once the chat is running and idle again, with the router's existing "queued until the session resumes" warning in the mission log. If the chat is archived or deleted, the binding ends with a `mission_warning`.
- **Archive.** Archiving the mission ends the binding without a notice. The router unmounts at archive (`mission_archive_impl`), and archiving is the person's or the agent's own act.
- **Runner quits.** Nothing is delivered. The chat cannot run either, since sessions live in Runner's process until #645. On relaunch the router restores the binding and its `up_to`, and wake events after `up_to` go out as one catch-up notice once the chat is live and idle. This is the one exception to "never on replay" (arch §8.5). It cannot repeat a notice, because `up_to` moves only when a notice is typed. With no host timeout, there is no watch expiry to recover from.
- **Two missions, one chat.** Each mission's router owns its binding and labels its notices, and the session's gate serializes them. Attaching a chat that already watches is a no-op.

### CLI output

The changes are additive only. The JSON result of `mission start`, `mission resume` and `mission show` gains `watch: {session_id, delivery: "runner"}` or `watch: null`. Plain output and `-q` add one stderr line when a chat is attached: `watch: Runner will send this mission's updates to this chat.` stdout, exit codes and `feed --follow` are unchanged, and `mission start` still returns at once.

### Skill and guide

`agent_skill.rs` (both `runner` and `runner-dev`) and `help.rs` change together:

- **Rule 5** becomes: after `mission start` or `resume`, check `watch`, or the stderr line with `-q`. If Runner delivers, tell the person the mission is running and its updates will arrive in this chat, and arm nothing. Otherwise use the host's watch facility or the limitation path, as today.
- **A new rule:** a line starting `[Runner] Mission` is Runner delivering a mission this chat watches. Run the command it gives and tell the person what happened in a sentence or two. Relay a question to the person and answer it only with their decision. Do not start a watcher.
- **WATCH FACILITIES and rule 7** now apply only outside a Runner chat. Claude Code's Monitor stays the worked example there.
- **`docs/arch/arch.md`:** §8.5 gains a watching chat as a delivery recipient, §9.4 `RUNNER_SESSION_ID` and the watch, and §9.6 the skill's two paths.

## Out of scope

- **Codex outside Runner**, in another terminal or the Codex app. The limitation path stands, and the app-server route in the findings is the follow-up if that becomes a production path.
- **Delivery in the middle of a turn.** The gate never interrupts; a notice waits for idle.
- **A mission slot as the watching chat.** A lead that starts another mission keeps today's rules until 562's seats.
- **More than one watching chat per mission**, and any UI for the watch. A "watched by" caption can come with 562's rail work.
- **Jason's personal `mission-watch` skill** in the memory repo. It has to learn to skip Monitor when Runner delivers, and is updated alongside this work, outside the repo.

## Decisions to confirm

1. **Claude Code in a Runner chat also uses Runner delivery** instead of Monitor, so there is one rule inside Runner and no duplicate notices. Monitor stays for Claude Code outside Runner. Recommended. The alternative keeps Monitor in Runner chats and has Runner skip Claude Code.
2. **Crew messages are counted but never wake the chat.** Waking on every handoff costs a Codex turn each time; #724 had eight crew messages and three updates that mattered.
3. **One watching chat per mission**, moved by `mission watch`.

## Implementation phases

1. **Backend.**
   - `RUNNER_SESSION_ID`, and `watch_session_id` on start and resume with its validation.
   - The `mission_watch` and `watch_delivered` signals, their replay, and their noise filtering.
   - The router's watching-chat recipient on the existing outbox and gate: the classifier, folding, the notice text and the catch-up.
   - The `mission_watch` tool.
   - Tests:
     - A chat attaches; a terminal pane and a slot are refused.
     - A wake event types one notice when the chat is idle, and waits while it is busy, drafting or showing an approval.
     - A burst folds into one notice, and crew messages are counted but not delivered.
     - Crash, end and archive each behave as specified.
     - A stopped chat queues its notice, and an archived chat ends the binding.
     - The restart catch-up delivers only undelivered events, once.
     - Two missions deliver into one chat, and a moved watch sends the move notice.
2. **CLI, skill and docs.** The `watch` field and stderr line, `mission watch`, the skill and guide rules, and the arch sections. Skill and help tests pin the new rules for both renders.
3. **Smoke test.** Jason runs it, since crews do not run the app; see Verification.

After sign-off, phases 1 and 2 are one crew mission.

## Verification

The checklist follows the issue's acceptance list.

- [ ] Give a fresh Codex chat in Runner the installed skill only, with no instruction to watch, and ask it to start a mission that posts a handoff, asks a yes/no question and finishes. `mission start` returns at once, the chat says updates will arrive here, and no follower runs.
- [ ] After that turn ends, the crew's broadcast and question each arrive as a `[Runner] Mission` user turn in the Codex rollout, followed by the agent's reply, with Jason sending nothing. Record those rollout lines as the proof of delivery.
- [ ] A notice waits while Jason types a draft in the chat, and while the chat works on something else, then arrives once.
- [ ] Killing one slot sends a crash notice while the other keeps running. Stopping every session sends the end notice and nothing after it. Archiving ends the watch silently.
- [ ] Quitting and relaunching Runner with an undelivered broadcast delivers it once after relaunch and never repeats earlier notices.
- [ ] Two missions from one chat stay labeled and separate. `mission watch` from a second chat moves the watch, with one notice to the first.
- [ ] Claude Code in a Runner chat gets the same notices and arms no Monitor. Claude Code outside Runner still arms Monitor (the 686 recipe).
- [ ] The release and development skills and `runner help agents` agree. `cargo test -p runner-backend -p runner-cli` and workspace clippy pass.
- [ ] Record the Codex version, macOS version and app build. Windows (JASONPC) and the other runtimes (pi, Copilot, TRAE, Antigravity, OpenCode) use the same gate and stay marked unverified until each is smoke-tested.
