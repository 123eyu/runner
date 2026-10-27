# 873 — Remove the mission lead

> Tracking issue: [#873](https://github.com/yicheng47/runner/issues/873). Background: [#562's spec](archive/562-mission-spawn.md), whose Phase 3 built the no-lead paths for missions started with **No crew** and removed `ask_lead`.

## Motivation

A crew needs exactly one lead, and a mission started from a crew gives that slot its own prompt. Since #562 a mission is a container with a changing roster: it can start empty, roles join and leave, and the chat that delegates work coordinates from outside it. The lead has become a vague role, and for a one-slot crew such as `codex qa` a wrong one: the only agent is told it leads a team that workers will escalate to, and that team does not exist.

Most of the lead's work is already gone. `ask_lead` was removed on 2026-10-15, the person's messages without a recipient go to every member in every mission, and a mission with no lead shares its goal and restart notices with every member. What remains in a crew mission is that only the lead receives the goal and restart notices; the rest is a second prompt, labels and refusals.

## Decisions

1. **The lead is removed, not made optional** (Jason, 2026-10-27). Every mission runs the way a **No crew** mission does today. Keeping an optional lead would keep two prompt shapes and every lead branch, and add a crew with no lead as a third state.
2. **The goal is shared with every member.** Who acts first is written in the role prompts and the team conventions, and the shared preamble tells each member to act only on its own part.
3. **Members look up the team.** The preamble tells each member to run `runner mission show` before messaging; `mission show` gains a ROLE column in place of LEAD. No roster is written into a prompt, because it goes stale after the first Add role or Remove.
4. **`runner ask "<question>"` asks the person**, the same as `runner ask --human`, which stays as an alias.
5. **Stored lead flags stay and are ignored.** Old databases and roster files load unchanged, and there is no migration.
6. **Roster changes stay with the person.** If agents later get #916's verbs, they are granted per role or slot, not through a lead seat.

## Scope

### Daemon

- One member prompt: `TEAM_COORDINATION_PREAMBLE`, the goal, the team conventions and the role's brief, for every runtime. The lead prompt goes: `compose_launch_prompt_sections`, `compose_launch_prompt`, `LaunchPromptInput`, `LeadView`, `RosterEntry`, `SessionPromptKind::Lead`, `PromptChannels::lead` and pi's lead path. `WORKER_COORDINATION_PREAMBLE` merges into the team preamble, which also stops saying "This mission has no lead".
- The preamble's team line becomes an instruction: run `runner mission show` to see the current team and their handles before messaging; the inbox announces every join and leave. A new line: act only on your own part of the goal; when your part waits on a teammate's handoff, finish your turn.
- The `ask_human` line drops `on_behalf_of`, which existed so a lead could pass on a worker's question. The signal still accepts it.
- `Router::record_slot_restart` notifies every other member. `LeadRow`, `LaunchInputs.lead`, `RosterRow.lead` and `Router::lead_handle` go.
- `ops::mission::start` accepts any crew with at least one slot. `mission_remove_role` removes any member.
- `ops/slot.rs` drops `set_lead`, first-slot auto-lead and promotion on remove; `repo::slot` drops `promote_to_lead` and `clear_crew_lead`. New slots are written with `lead = 0`, and the seed stops setting it.
- Session spawn and resume choose the member prompt for every slot. Fresh starts send it as the first turn; a genuine Pi resume reapplies the same context through its system-prompt file without submitting another goal turn, including for legacy crew conversations.

### CLI and protocol

- `runner crew lead` is removed. `crew list` drops the `*`, `crew show` drops LEAD, and `mission show` replaces LEAD with ROLE.
- `runner ask "<question>"` posts `ask_human`; `--human` stays as an alias and `--context` goes. `ASK_LEAD_REMOVED_HINT` becomes "message a member with `runner msg post --to <handle>`, or ask the person with `runner ask`", and `runner signal ask_lead` is still refused with it.
- `runner help`, the generated agent skill (`runner-core/src/protocol/agent_skill.rs`) and `docs/guide/reference/cli.md` (`make guide-cli`) drop the lead wording.
- `slot_set_lead` leaves the client protocol. The local protocol may drop `lead` from its slot, crew and session types. The companion protocol (`runner-core/src/companion`) keeps its `lead` fields, always `false`, so remote clients of other versions still parse.
- `runner-core/src/mission_roster.rs` still reads the `lead` flag in version 1 and 2 roster files and writes `false`.

### App

- Crew editor: no LEAD badge, no **Set as lead** (`CREW_SLOT_SET_LEAD`), no lead-specific remove confirmation, no "lead" in the crew summary.
- Start mission dialog: no LEAD badge; the goal hint is the shared one in every case ("Shared with every member").
- Mission workspace: no LEAD badge on cards, every member has Remove, one restart confirmation (`format.restart_slot_conversation`), the feed's goal row addressed to everyone, and the composer always reads "Message everyone — @handle to address one", locally and remotely.
- Unused keys leave `i18n/en.toml` and `i18n/zh-hans.toml` together.

### Docs

- Guide: `concepts/crews.md` (drop "Slots and the lead"), `concepts/missions.md`, `getting-started/first-mission.md`, `guides/build-a-crew.md`, `guides/drive-runner-from-agents.md`, `reference/cli.md`.
- `docs/product/vision.md`, `docs/arch/arch.md`, `docs/product/agent-environment.md`, and the Crew entry in `AGENTS.md`'s vocabulary.
- `README.md` and `README.zh-CN.md` together: the crews paragraph and the alt text of `assets/crews.png`. The screenshot still shows a lead badge until it is retaken, which is out of scope.
- `docs/tests/regression/missions.md`: the rows that name a lead (MIS-CREW-01, MIS-ADD-01, MIS-REM-01, MIS-ROSTER-01, MIS-INDEP-01, MIS-ASK-01, MIS-MSG-06) are reworded for missions without one.
- `examples/pair-coding/goal.md` drops "(lead)", and the seed's description of the crew matches.

## Design

The UI change is removal only: the LEAD badge, **Set as lead** and the lead-specific copy come out of the app, and nothing new is drawn, so no design pass precedes implementation. The `cmp/` components and screens in `runner/desktop/runner.pen` that still show the badge are updated separately.

## Out of scope

- Dropping the `slots.lead` column or the roster file's `lead` key.
- #916's verbs, #748's starting-chat link, and agents adding or removing roles.
- Changing `ask_human`, the inbox or delivery.

## Verification

### In-process

- Prompt composition for every runtime contains the goal, the team preamble with the `mission show` and own-part lines, the conventions and the brief, and never "lead". Genuine Pi mission resumes with either legacy flag keep their conversation key and reapply that context in the system-prompt file, with no first turn; cover manual and daemon-startup resume.
- A crew with one or three slots starts; every member's first turn carries the goal.
- A restart notifies every other member; Remove works on any member, including the first slot.
- Old databases with `lead = 1` and roster files of versions 1 and 2 with lead flags load, mount and resume.
- CLI output, help, golden tests and the generated reference; `runner ask "<q>"` posts `ask_human`; the companion payload keeps `lead: false`.
- App tests for the crew editor, Start mission dialog, rail, feed and composer.

### Live, macOS

Jason asked on 2026-10-27 for a live regression of the mission's main functionality, so the selection is wider than the diff alone. Run on the candidate in the development app, with inexpensive models, tiny prompts and fresh folders under `/tmp`.

| Check | Why |
| --- | --- |
| LEAD-01: a three-slot crew (coder, reviewer, qa shape) with a small real goal | Every member gets the goal; the reviewer and qa wait for the coder's handoff instead of acting on it. No terminal shows a lead prompt. |
| LEAD-02: a one-slot crew | Its member gets the goal and the team preamble and starts work without a team to wait on. |
| LEAD-03: crew editor, Start mission dialog, rail, feed and composer | No LEAD badge, **Set as lead** or lead copy; the goal row is addressed to everyone; every member has Remove. |
| LEAD-04: `crew list`, `crew show`, `mission show`, `crew lead`, `runner ask "<q>"` | Output without lead, ROLE in `mission show`, `crew lead` unknown, and the ask becomes a card for the person. |
| MIS-CREW-01, MIS-MSG-01, MIS-ASK-01 | The smoke-tier mission rows; each named a lead. |
| MIS-LIFE-01 | Explicitly Restart a slot and observe its notices at both other members, as well as Stop/Resume. |
| RT-LIFE-07 (Pi mission member) | Stop/Resume retains role and team conventions through the system channel, with no extra first turn. |
| MIS-ADD-01, MIS-REM-01 | Adding to a crew mission and removing any member, including the first slot. |
| MIS-ROSTER-01 | Crew edits still leave a running mission alone, without "move the lead". |
| MIS-EMPTY-01 | **No crew** missions are now the only model and must be unchanged. |
| MIS-INDEP-01, MIS-MSG-02, MIS-MSG-06, MIS-MSG-07 | Composer, addressing and startup delivery, which named or routed through the lead. |
| MIS-FEED-01 | The goal row and cards in the feed. |

Not selected: MIS-ROSTER-02 (needs a human-held legacy fixture; old roster files are covered in-process), Windows-only rows, the draft and wait rows, watching and long-feed rows: this change does not touch them. Windows and Linux live checks are recorded as gaps; their CI runs.

Results go in `docs/tests/873-remove-mission-lead.md` with the suite's case IDs and evidence format.
