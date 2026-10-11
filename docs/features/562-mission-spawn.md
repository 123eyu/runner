# 562 — Missions as containers: missions without a crew, roles added and removed while they run

> Tracking issue: [#562](https://github.com/yicheng47/runner/issues/562), "delegate work to other agents". Priority: P1, milestone 0.16 (moved from 0.17 on 2026-10-11, when #562 became the parent of [#863](https://github.com/yicheng47/runner/issues/863) and [#748](https://github.com/yicheng47/runner/issues/748) and [#704](https://github.com/yicheng47/runner/issues/704) closed into it). Delegation from a chat works end to end once Phase 1, Phase 3 and [748](./748-mission-watch-delivery.md) have landed.
> Status: Phase 1, the mission roster, is settled and is the fix for [#863](https://github.com/yicheng47/runner/issues/863); it ships on its own as a 0.15.x patch. Phases 2 to 4 are refined in this spec before their work starts, and the app phase is designed first.
> Design: `design/specs/562-mission-spawn.pen`, drawn on `main` before Phase 4, together with the Start mission dialog of [849](./849-start-mission-modal.md).
> Identity: a caller is the person at the app or a roster handle, never a location ([648](./archive/648-runner-cli.md) decision 7).
> Related: [748](./748-mission-watch-delivery.md), which types a mission's updates into the chat that started it, the return path for delegation. Direct chats stay off the bus. [704](./archive/704-session-send.md), `runner session send` into a chat that is already open, closed into this issue on 2026-10-11 and not scheduled. [#863](https://github.com/yicheng47/runner/issues/863), the crew-edit roster bug that Phase 1 fixes.
> Rewritten 2026-10-10 against the `runnerd` code, after Jason set the direction: a crew is not a prerequisite for a mission, a mission can start from a single role, and roles can be added to or removed from a running mission when needed. The 2026-09-18 draft's lead verbs (`runner spawn`, `ps`, `stop`) and outside seats move to Follow-ups; they build on this.
> Revised 2026-10-11: Jason replaced the 2026-10-10 choice of a crew row per mission (`crews.kind`, `template` or `mission`) with a roster file in the mission's directory. The database holds the definitions that are listed and searched (roles, crews and their slots); a mission's roster is the mission's own state, like its event log.

## Motivation

A mission can only start from a crew today. `ops::mission::start` refuses a crew with no slots or no lead, every session is spawned from one of the crew's slots, and the roster cannot change until the mission ends. Three problems follow:

- **One agent handing work to another.** Jason's daily pattern (a Claude Code session plans, a Codex agent implements) needs a one-slot crew ("codex solo") just to start a mission. A crew of one is setup in front of a single role.
- **A mission that needs another pair of hands.** When the task splits three ways, or a reviewer is needed that the crew never had, the only move is to stop the mission, edit the crew and start over, losing every conversation. The reverse, letting an agent go once its part is done, is not possible either.
- **A running mission does not own its roster** ([#863](https://github.com/yicheng47/runner/issues/863)). The daemon and the app read the crew's slots as they are now, so editing the crew while a mission runs changes that mission the next time its router is rebuilt, which happens on every `runnerd` restart, including an app update. The router, the bus, the CLI's `roster.json` and the live sessions then disagree.

Crews stay. A repeatable shape, such as coder plus reviewer, needs a fixed template so runs can be compared. The crew becomes one way to fill a mission, not the only way in.

## Model

- **Crew**: a named, reusable set of slots, each filled by a role, with exactly one lead, edited on the crew page. Unchanged. Roles, crews and their slots are the definitions the database holds, lists and searches.
- **Mission roster**: the slots one mission runs with, kept in `roster.json` in the mission's directory beside `events.ndjson`. It is written when the mission starts, changed only by that mission's own add-role and remove-role operations, and read by everything that needs the mission's roster: the router and bus when they mount, the handle checks on posts and signals, the lead's prompt, session resume, the app's mission view and the CLI.
- **Mission**: a container with one event log, a cwd, a goal, and one roster with exactly one lead.
- **Start from a crew**: copy the crew's name, prompt addendum and slots into the roster. Editing or deleting the crew afterwards never touches the mission. `missions.crew_id` keeps naming the crew the mission started from.
- **Start from roles**: build the roster from one or more picked roles, one marked as lead, with no crew behind it. A single role is a one-agent mission.
- **Add a role**: add a slot to the roster of a running mission and start its session. An added role is always a worker.
- **Remove a role**: stop the slot's session and drop the slot from the roster. Its messages stay in the feed and its handle stays taken in that mission. The lead cannot be removed.

**Why a file, not rows.** Nothing lists, searches or joins mission rosters across missions. The crew page, the pickers and `runner crew list` show crews, and "which missions use this role" is answered by `sessions`, which carries `role_id` and `mission_id`. Every reader of a roster reads one mission's. The file also exists already: `mission_start` writes `roster.json` for the CLI's `--to` check, and #863 is the daemon and the app reading the crew's current slots instead. A roster file needs no change to the `crews` or `slots` tables and no filter keeping per-mission copies out of crew lists. The 2026-10-10 draft gave each mission its own crew row instead; that put the same data in the database, needed a `kind` filter in every crew query, the role page's "Crews using this role", first-run seeding and the pickers, and needed an upgrade that moved every mission's directory.

**What stays linked.** A roster slot names its role by id, so a role edit reaches a running mission at its next resume, as it does today. Copying the role's own configuration into the roster, so that role edits and deletions never reach a mission either, is a Follow-up.

UI copy keeps "Crew" for crews, the only thing a person picks or edits by that name. In code and docs the mission's slots are *the mission roster*. The vision's "Mission: one live activation of a crew" becomes "one live run of its own roster, seeded from a crew or from roles".

### The roster file

Version 2, written by Phase 1:

```json
{
  "version": 2,
  "crew_name": "Codex pair",
  "crew_addendum": "Team conventions…",
  "slots": [
    {
      "id": "01K7…",
      "handle": "lead",
      "lead": true,
      "position": 0,
      "role_id": "01K6…",
      "runtime_override": null,
      "model_override": null,
      "effort_override": null,
      "codex_speed_override": null
    }
  ]
}
```

- A slot keeps the id it had in the crew, so `sessions.slot_id` names a roster slot. Slots added in Phase 2 get new ids that exist only in the file.
- `crew_name` and `crew_addendum` are the copies mission prompts read. Phase 1 display surfaces keep reading the crew's current name. A mission started from roles has an empty name and no addendum.
- Version 1 is the file written before Phase 1: a bare array of `{ "handle", "lead" }`. It is converted once when `runnerd` starts, as described under Existing missions, and nothing else reads it.
- The file is written atomically, through a temporary file renamed over it, as it is today. Its type lives in `runner-core`, so the daemon and the CLI parse one definition.

## How it works today

Most of the runtime machinery already works per session and per handle; Phase 1 changes where the roster is read from, and later phases make it change while the mission runs.

| Layer | Today | With the mission roster |
| --- | --- | --- |
| Database | `missions.crew_id` and `slots.crew_id` are `NOT NULL`; a mission uses its crew's slots directly, and `sessions.slot_id` points at one of them. | Phase 1: unchanged. `sessions.slot_id` names a roster slot, which keeps its id from the crew. Phase 3: a mission started from roles has no crew. |
| Start | `ops::mission::start` requires a crew with a lead, writes `roster.json` with each slot's handle and lead flag, and `mission_start_impl_with_size` reads the crew's slots again after the commit to spawn. | Builds the roster once in the start transaction, writes it, and spawns from it. |
| Re-mount | `ensure_mission_router_mounted` builds the router and mounts the bus from `slot::list(mission.crew_id)`, and reads the crew's name and addendum from its row. | Reads the roster file. |
| Posts and signals | `validate_roster_handle` and `post_message` check `--from` and `--to` against `slot::list(mission.crew_id)`. | Check against the roster file. |
| Session resume | `session/manager/spawn.rs` reads the slot row by `slot_id` for the handle and lead flag, and fails with "slot … no longer exists" once the crew dropped it; a lead's resume reads the crew's slots, name and addendum for its prompt. | Reads the slot, the name and the addendum from the roster file. |
| App | `mission_workspace/attach.rs` loads the roster with `slot_list(mission.crew_id)`. | Loads it with `mission_roster(mission_id)`. |
| CLI | Rereads `roster.json` on every call to validate `--to`. | Reads version 2. |
| Event log, events, env | `crews/<crew_id>/missions/<mission_id>/events.ndjson`; `crew_id` on every envelope; `RUNNER_CREW_ID` on every session. | Unchanged for a mission started from a crew. Phase 3 settles them for a mission without one. |
| Crew lists, missions of a crew | Every crew is listed; `runner mission list --crew` filters on `missions.crew_id`. | Unchanged. |
| Crew deletion | Refused while the crew has unarchived missions; deleting it deletes its archived missions' rows. | Phase 1: unchanged. |
| Session spawn | `register_mission_session` then `complete_mission_session_spawn`, once per slot; nudges queue until the PTY is live. | Ready for one new slot. |
| Router routing | `session_by_handle` is a map under the router's state lock; `register_sessions` adds entries at any time. | Ready. |
| Router roster | `LaunchInputs.roster` is fixed at mount; broadcast nudges walk it and the lead's Restart prompt lists it. | Needs add and remove (Phase 2). |
| Bus | `BusState.handles` is fixed at mount ("Adding handles after mount is not supported in MVP") and decides whose inbox each message lands in. | Needs add and remove (Phase 2). |
| Per-session lifecycle | #542's Stop, Resume and Restart work on one session. | Ready; Remove uses Stop. |

## Scope

### In scope

- **The roster file** (Phase 1), as above, owned by one module in `runner-daemon` that writes it at start and loads it for one mission.
- **Start from a crew** (Phase 1). `mission_start` reads the crew's slots once, inside its transaction, writes them as the roster with the crew's name and addendum, and the spawn loop, the prompts, the router and the bus use that roster. Today it reads the slots three times, the last after the commit, so an edit in between could spawn a different roster from the one the CLI checks.
- **Every reader uses the roster** (Phase 1): `ensure_mission_router_mounted` (the router, the bus's handles, the crew name and addendum), `validate_roster_handle`, `post_message`, the session resume in `session/manager/spawn.rs` (the slot's handle and lead flag, and the lead's prompt), and the app's mission view through a new `mission_roster(mission_id)` call on the client protocol. It returns the same `SlotWithRole` rows as `slot_list`, with the slots from the file and each role from the database, and skips a slot whose role has been deleted.
- **Existing missions** (Phase 1). Converted once when `runnerd` starts, before any router mounts or session resumes: every mission, running or archived, whose file is version 1 or missing gets a version 2 file, and a version 2 file is left alone, so later starts only scan. Sessions all restart with `runnerd`, so no agent runs the CLI before the conversion is done, and every reader, the CLI included, knows only version 2. The conversion is one function, the only code that reads version 1, and is deleted once two releases have shipped with it. A missing file or one that is not valid JSON at load falls back to the crew's current slots, today's behavior, written as version 2 with a warning in the daemon log. Other I/O errors, unsupported versions and valid JSON with an invalid roster shape return an error without rewriting the file. This also refuses a leftover version 1 file after the conversion is deleted. The conversion:
  - The crew's name and addendum come from the crew row.
  - Each handle in the file is matched to a slot the mission's sessions ran under: first the session slot whose current handle is the same; then, in the order the sessions first started, a session slot that no other handle matched. That covers a slot #863 renamed, and one it deleted, for which the session's own role, runtime, model, effort and speed stand in for the slot row.
  - A handle without a session (its spawn failed) takes the crew's slot of that handle; a handle with neither is dropped with a warning in the daemon log, since nothing ran under it.
  - The file's handles, order and lead flags always win over the crew's current ones.
  - A mission with no file at all takes the crew's current slots, which is what it runs with today.
- **Deleting a role** (Phase 1). Also refused while an unarchived mission has a session for the role: "role @x is used by unarchived missions; archive them before deleting this role: …", naming the missions. Today the crew check covers a mission's roles only while its crew still has them; once the slot is removed from the crew, deleting the role deletes the running mission's session with it, since role deletion deletes every session of the role (`ops/role.rs`, `delete`).
- **Crew edits and deletion** (Phase 1). The crew page's slot operations need no guard, since a crew edit no longer changes a mission's runtime roster or prompt copies. Display surfaces, including mission summaries and CLI list/status, keep reading the crew's current name as before Phase 1; the rail's Crew row still reads and opens the current crew. Deleting a crew is unchanged.
- **Start from roles** (Phase 3). `mission_start` takes exactly one of `crew_id` or `roles: [{ role, handle?, lead, runtime?, model?, effort? }]` with exactly one lead, and builds the roster from the roles with an empty crew name and no addendum; the mission header, summaries and the lead prompt leave an empty crew name out. A mission without a crew needs `missions.crew_id` to become nullable (a table rebuild) and a directory that is not under a crew. Settled before Phase 3 starts: where that directory lives (recommended `missions/<mission_id>/`, where its `runner` shims already live), what an event's `crew_id` and the session's `RUNNER_CREW_ID` carry without a crew, and whether deleting a crew then keeps the missions started from it.
- **Add a role** (Phase 2). `mission_add_role(mission_id, { role, handle?, runtime?, model?, effort?, task? })`. It refuses unless the mission is running and not archived. It picks the handle with the crew page's rule unless one is given, treating the handles of roles removed earlier in the mission, read from its `role_removed` events, as taken (`suggest_slot_handle` and `slot_handle_error` move from `runner-app`'s `surfaces/crews/logic.rs` into `runner-core` so the daemon and the modal share them). It adds the slot to the roster file, adds the handle to the router's roster and session map and to the bus with an inbox that starts at its join event, appends a `role_joined` signal and a broadcast `message` from `runner` that introduces the newcomer to the team and the team to the newcomer, adds `task` as a directed message, then spawns through the existing per-slot path with the worker first turn.
- **Remove a role** (Phase 2). `mission_remove_role(mission_id, handle)`. It refuses the lead and an unknown or already-removed handle. It stops the slot's session with #542's Stop, drops the slot from the roster file, removes the handle from the router's roster (broadcasts skip it, `--to` refuses it) and from the bus's projection, and appends a `role_removed` signal and a `runner` broadcast that the agent left the mission. A re-mount rebuilds the smaller roster from the file. The session's row stays as the mission's history.
- **Router and bus** (Phase 2). `LaunchInputs.roster` moves behind the router's state lock with add and remove; broadcast nudges and the lead's Restart prompt read the current roster. The bus gains add-handle and remove-handle control messages handled on its consumer thread, so a join or leave is ordered exactly against the log it projects. Roster file rewrites are serialized per mission, and on Windows a rename over a file the CLI is reading is retried briefly.
- **Protocol and CLI.** The client protocol carries `mission_roster` (Phase 1) and the start, add and remove operations (Phases 2 and 3). The `runner` CLI gains `runner mission start --role <role>` (repeatable, `--lead <role>` when more than one) beside `--crew`, `runner mission add-role <mission> <role> [--as <handle>] [--runtime …] [--model …] [--effort …] [--task …]` and `runner mission remove-role <mission> <handle>`. They act as the person, the identity of a caller without a handle, and are refused from inside a mission session until the lead's verbs exist (Follow-ups).
- **App** (Phase 4). The Start mission dialog offers a crew or a set of roles (with 849). The mission rail gains **+ Add role** under its sessions while the mission runs, opening a modal with the role picker from the crew page's add-slot form, runtime, model and effort, a handle pre-filled with the suggested unique handle, and an optional task. A worker card's menu gains **Remove from mission**, with a confirmation. The feed renders `role_joined` and `role_removed` rows. The rail's Crew row, which opens the crew the mission started from, is redesigned for a mission without a crew.
- **Docs.** `AGENTS.md` and `docs/product/vision.md` core vocabulary as in Model; `docs/arch/arch.md` §3 and §10 for the roster file (Phase 1), and §7 and §8 for `role_joined`, `role_removed` and the roster changing while a mission runs (Phase 2).

### Decisions to confirm

Each has a recommendation; the spec assumes it until Jason decides otherwise. Phase 1 needs none of them.

1. **A removed role's handle stays taken in the mission.** Reusing it would merge two agents' history under one name in the feed and in inbox projection. Adding the same role again gets `coder-2`. The add operation checks the mission's `role_removed` events.
2. **A mission cannot start empty.** It needs a lead to receive the goal. "Create a mission, then assign roles" is: pick the lead role at start, then add the rest.
3. **The lead is fixed for the mission's life.** Removing or changing the lead is not in this delivery.
4. **No roster cap for the person.** A cap arrives with the lead's spawn verb, where an agent could loop.
5. **Add and remove are for the person only** until the lead's verbs land; a mission agent calling them is refused.

### Out of scope

- **The lead's verbs** (Follow-ups): `runner spawn`, `runner ps` and `runner stop` from the lead, idle and crash notices to the lead (`slot_exited`), and outside seats, where an agent outside Runner holds the lead seat. A separate issue once this lands.
- **A copy of each role's configuration in the roster** (Follow-ups): the role's command, args, env, system prompt, runtime, model and effort as they were at start, read by resume instead of the role row, with the role link no longer cascading, so role edits and deletions never reach a mission. Today a resume reads the role fresh (`session/manager/spawn.rs`, "Pull the role config fresh").
- Saving a mission's roster as a crew.
- A direct chat joining a mission as a slot. A chat's `RUNNER_*` environment is fixed at spawn; it drives a mission from outside instead.
- Workers adding or removing roles.
- Adding to or removing from a completed, aborted or archived mission, or while the mission's cold-start spawn queue is still running.

## Implementation Phases

Phases 1 to 3 are daemon and CLI work and can land before any UI. Each phase is a commit that builds and passes its tests.

### Phase 1 — the mission roster (fixes #863)

One mission and one pull request against `main`, shipped as a 0.15.x patch that closes #863. No schema change and nothing visible: the rail's Crew row still opens the crew the mission started from.

- `runner-core`: the roster file type, version 2.
- `runner-daemon`: the roster module (writing at start, loading, and the fallback for a missing file or one that is not valid JSON), the startup conversion of version 1 and missing files, `mission_start` building and spawning from one roster, the readers listed under Scope switched to it, `mission_roster` on the client protocol, and the role-deletion refusal.
- `runner-cli`: `roster.rs` parses version 2.
- `runner-app`: `mission_workspace/attach.rs` loads the roster with `mission_roster`.
- `docs/arch/arch.md`: the mission roster in §3 and the roster file in §10's directory layout.
- Tests:
  - Starting a mission writes a version 2 file with the crew's name, addendum and slots, overrides included, and spawns those slots.
  - With a mission running, each crew edit (remove a slot, add one, rename a handle, move the lead, change an override, edit the addendum) followed by a re-mount, as at a `runnerd` restart: the router's roster and handle map, the bus's handles, the `--from` and `--to` checks, the lead's prompt and a worker's resume all follow the file, and the CLI accepts exactly the file's handles.
  - A session whose slot the crew deleted resumes instead of failing with "slot … no longer exists".
  - The startup conversion of a version 1 file for an untouched mission, a renamed slot, a deleted slot, a deleted slot whose handle a new crew slot reused, and a handle whose spawn failed, for running and archived missions; a mission without a file takes the crew's current slots; the conversion finishes before any router mounts, and a second start changes nothing.
  - The CLI reads version 2. At daemon load, a missing file or one that is not valid JSON falls back to the crew's current slots; other read errors and unsupported versions preserve the file and return an error.
  - `mission_roster` returns the file's slots with their roles and skips a slot whose role is gone.
  - Deleting a role is refused while an unarchived mission has a session for it, including after its slot left the crew, and allowed once the mission is archived.
  - Crew deletion behaves as before.
- Live checks, macOS and Windows: #863's reproduction (edit the crew while a mission runs, restart `runnerd`, then post from a removed worker, broadcast from the lead and resume a slot), and the affected cases of the mission regression suite: MIS-CREW-01, MIS-LIFE-01, MIS-CONFIG-01 and MIS-MSG-01.

### Phase 2 — add and remove while running

- `ops::mission::{mission_add_role, mission_remove_role}`, the shared handle rule in `runner-core`, the router's mutable roster, the bus's add-handle and remove-handle, the serialized roster file rewrite, and the `role_joined` and `role_removed` events.
- Client protocol operations and the `runner mission add-role` and `remove-role` commands.
- Tests: join events land in order with the documented payloads; the newcomer's inbox holds the broadcast and the task and nothing earlier; `--to` the newcomer validates right after the join and broadcasts reach it; `--to` a removed handle is refused and broadcasts skip it; removing the lead is refused; adding a removed role's handle again is suffixed or refused; a re-mount after a daemon restart rebuilds both changes from the file; two missions of one crew can each add `@coder-2`.

### Phase 3 — start from roles

- Settle the directory, envelope and crew-deletion questions under Scope, then `mission_start` with `roles`, the nullable `missions.crew_id`, the empty-name rule in the lead prompt and the mission header, and `runner mission start --role`.
- Tests: a one-role mission runs end to end and its lead's `runner msg post` works; a several-role start has exactly one lead and the picked roster; a start with zero or two leads is refused.

### Phase 4 — design, then app

- Draw the frames in `design/specs/562-mission-spawn.pen` on `main` and stop for sign-off: the Start mission dialog's crew-or-roles choice (with 849), **+ Add role** and its modal, **Remove from mission** and its confirmation, a card added mid-mission, the rail's Crew row for a mission without a crew, and the two feed rows.
- `mission_workspace`: the rail row, the modal (reusing the crew page's add-slot pieces and the shared handle rule), the card menu item, and the feed rows; `app_store.rs` refreshes on `role_joined` and `role_removed`; the Start mission dialog.
- Tests: the modal validates handles against the mission roster, including removed ones; the rail shows a slot added after `session/spawned` and hides a removed one; feed row copy.

## Verification

- [ ] A mission started from a crew runs as before on its own roster file, and missions started before Phase 1 have their files converted at the first `runnerd` start, then resume, restart and replay unchanged.
- [ ] Editing a crew (slots, handles, lead, overrides, addendum) never changes a running mission, including after a `runnerd` restart.
- [ ] A role used by an unarchived mission cannot be deleted.
- [ ] A mission started from one role runs end to end: the lead gets the goal and posts with `runner msg post`.
- [ ] A mission started from several roles has exactly one lead and the roster that was picked.
- [ ] An added role boots with the join broadcast and its task, is reachable with `--to` at once, receives later broadcasts, and survives a daemon restart.
- [ ] A removed role's session stops, `--to` it is refused, broadcasts skip it, its history stays in the feed, its handle is not reused, and the removal survives a daemon restart.
- [ ] The lead cannot be removed, and add and remove are refused on a mission that is not running.
- [ ] Every join and leave on the feed is attributed to the person.
- [ ] Live checks: #863's reproduction and the affected `MIS-*` cases in [`docs/tests/regression/missions.md`](../tests/regression/missions.md), plus new cases for a roles start, an add and a remove, on macOS and Windows.
