# 878 — a session records its launch definition: plan

Implementation plan for [#878](https://github.com/yicheng47/runner/issues/878) (bug, P2, milestone 0.16), the prerequisite for [#562](https://github.com/yicheng47/runner/issues/562)'s Phase 2. The rule it implements is "What is copied, and when" in [`docs/features/562-mission-spawn.md`](../../features/562-mission-spawn.md): roles and crews are templates, and a session records its whole launch definition when it spawns, so later role edits never change it. The mission brief is [`briefs/878-session-launch-definition.md`](https://github.com/yicheng47/runner/blob/v0.16.2/docs/impls/briefs/878-session-launch-definition.md). Line numbers are as of `7fd40b9c`.

## What happens today

| Field | Recorded at spawn | What Resume uses |
| --- | --- | --- |
| Runtime | `agent_runtime`, always | The recorded value |
| Command | `agent_command`, after `resolve_role_executable`, so the resolved path | The role's current command; the recorded one is read only for sessions without a role |
| Model, effort, speed | `agent_model`, `agent_effort`, `agent_speed` (stored in `runtime_options_json`), only when overridden | The recorded value, else the role's current one |
| Args, env | Not recorded | The role's current values |
| System prompt | Not recorded | The role's current prompt, whenever a prompt is composed: a fresh conversation, Restart, and every pi launch |

Resume and Restart share `resume_with_fresh_fallback` (`session/manager/spawn.rs` 2422; Restart passes `fresh = true` and is refused for rows that are not mission slots). Fork (about 1911–2040) also reads the current role. Sessions without a role rebuild from the `agent_*` columns through `resolve_runtime_only_resume_role` (376).

The screen shows the same mixture. `get_direct` (`ops/session.rs` 456) gives the chat header and side panel the current role's model, effort and command wherever the row does not override them. `runner session show` gets `session_get_with_status` (425), which for a mission session is the whole serialized `SessionRowDb`, every column included, and the CLI labels speed `fast (session)` when the row overrides it and `inherit (role or slot)` when it does not (`runner-cli/src/output.rs` 577).

## Decisions

1. **A column for what SQL reads, one JSON record for what only a launch reads.** `sessions.launch_definition_json`, nullable `TEXT`, holds the launch fields as one versioned record. `agent_runtime` stays a column and is the only record of the runtime: lists, shell checks, live counts and the orphan sweep filter on it in SQL, so the definition does not repeat it. A row with a definition always has `agent_runtime`. A separate column per launch field could not tell "recorded as the agent's default" from "not recorded".
2. **It records the effective values, before the machine is applied.** Effective means after a slot's or chat's overrides, as `resolve_runtime_override` and the speed rule compose them today. Before the machine means the command as configured (before `resolve_role_executable`) and configured args, including permission flags; the live permission layer strips or replaces them at launch. With a definition present, a `null` model or effort means the agent's default, never "follow the role".
3. **It records the role's system prompt text, not a composed prompt.** The crew addendum comes from the roster, and the lead's goal and team from the log and roster; those are mission state and keep composing at every launch that composes a prompt.
4. **Resume, fork and the screen read it.** Resume and fork build the launch from the definition and `agent_runtime`. Fields outside it come from where they do today: the handle and display name from the role row (or `runtime_direct_role` without a role), and the cwd from the row. The chat header, the side panel and `runner session show` show the definition's model, effort, speed and command, so what they show is what Resume will run.
5. **Restart rebuilds it.** The `fresh` path composes a new definition from the current role and the roster slot's overrides, exactly as `register_mission_session` does at spawn, launches from it, and writes it and `agent_runtime`. Restart is the only path that reads the role's launch fields. This changes #870's behaviour for Restart only.
6. **The machine stays live at every launch:** `resolve_role_executable` (including Settings → Agents executable overrides), the mission permission mode (`apply_mission`), stripping permission flags for direct chats on resume, and the shell environment the Cursor and Grok conversation probes read.
7. **Existing rows are converted once, inside the migration, and there is no fallback.** Migration `0026` adds the column and fills it in a Rust step in the same transaction, as `backfill_0014_nodes` does (`db/migrations.rs` 177): the conversion needs only the database, and `migrations/README.md` puts such a step in the migration. Every row gets the definition its next resume would use today. After that, Resume reads only the definition, and a row still without one fails with "session … has no launch definition". Nothing runs at `runnerd` start, and there is no converter to delete later.
8. **Every session column the definition makes unnecessary is dropped, in migration `0027`.** Jason asked on 2026-10-12 to drop them in this change rather than in later migrations. They are the nine marked dropped under Columns afterwards. `0027` is separate from `0026` only because the runner applies a migration's SQL before its Rust step and the conversion reads three of the dropped columns; both apply in the same upgrade.
9. **`agent_command` becomes the executable the recorded `pid` runs.** Today spawn writes the resolved path once. From now on the write after every launch (`update_runtime_metadata`, `repo/session.rs` 303, called at `spawn.rs` 1144, 1720, 2153 and 2892) records the pid and the resolved executable together and no longer writes the two dropped process-host columns. The startup orphan sweep (`list_orphan_process_candidates`, `repo/session.rs` 1030) matches a leftover pid against that pair. It is not a launch input: a launch resolves the definition's `command`.
10. **Storage stays in the repository layer**, as `repo/README.md` already requires: SQL lives only in `repo/` and `db/`, and `ops` and the session manager call repository functions. Only `repo/session.rs` knows the column's JSON, its version and the `runtime_options` shape; everything else works with a typed `LaunchDefinition` and calls the functions under Where it lands. No new code outside `repo/` and `db/` names a session column.
11. **Out of scope:** role deletion (still refused while an unarchived mission uses the role), the roster format, the UI (the protocol's `DirectSessionEntry` keeps its fields, now filled from the definition), renaming the remaining columns, and the argv of any session whose role was not edited, which must stay byte-identical.

## Columns afterwards

| Column | Meaning | Written by | Read by |
| --- | --- | --- | --- |
| `launch_definition_json` | What to launch, below | Spawn, fork, Restart, migration `0026` | Resume, fork, the screen, Phase 1's roster converter |
| `agent_runtime` | Which agent; the only record of the runtime | Spawn, fork, Restart, `0026` where empty | SQL filters, the screen, Resume, the orphan sweep |
| `agent_command` | The resolved executable the recorded `pid` runs | Every launch, with `pid` | The orphan sweep |
| `agent_session_key` | The agent's own conversation id | Unchanged | Resume |
| `runtime_socket`, `runtime_window`, `runtime_pane`, `runtime_cursor` | Dropped. tmux-era (`0003`), mapped into `SessionRowDb` and written `NULL` | | Nothing |
| `runtime`, `runtime_session` | Dropped. The process host (`pty`) and its id for the live PTY (`0003`), written after every launch | | Nothing; the live id is in memory |
| `agent_model`, `agent_effort` (`0019`), `runtime_options_json` (`0024`) | Dropped. The session's overrides, replaced by the definition's effective values | | Resume, fork, the screen, `runner session show` and Phase 1's roster converter (`ops/mission/roster.rs` 156), which move to the definition |

Dropping the three override columns loses one fact the definition does not keep: whether a value was overridden or followed the role. After this change nothing follows the role, so nothing needs it; the CLI's speed label is the one place that showed it, and it changes below. The values themselves move into every converted row's definition as effective values. A row the conversion cannot compose (an unknown runtime, or a runtime-only row without one) fails today's resume the same way, so the drop loses no session that can run today.

`ALTER TABLE … DROP COLUMN` rewrites the rows without dropping the table, so `session_attention`'s rows keep their `ON DELETE CASCADE` parent, and none of the nine is indexed or named by a constraint, view or trigger. Checked with SQLite 3.51 on the #879 schema snapshot on 2026-10-12: all nine dropped in one transaction, the attention row stayed, and `PRAGMA foreign_key_check` was clean.

**Permission decision (2026-10-12, mission human response):** keep permission args in the recorded definition to preserve legacy `RoleDefault` argv. Direct chats strip them at every launch; mission Auto/Bypass strip and replace them; mission RoleDefault keeps the recorded flags.

## The definition

Version 1, as stored:

```json
{
  "version": 1,
  "command": "codex",
  "args": ["--search"],
  "env": { "RUNNER_878_MARKER": "before" },
  "model": "gpt-6.1-sol",
  "effort": "xhigh",
  "runtime_options": { "codex": { "speed": "fast" } },
  "system_prompt": "You are the reviewer…"
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | integer | `1`. A reader refuses any other version. |
| `command` | string | As configured, for the row's `agent_runtime`. With a runtime override it is that runtime's catalog command, as `resolve_runtime_override` gives today. A shell keeps its recorded shell path. |
| `args` | array of strings | Effective args, permission flags included; the live layer strips or replaces them at launch. |
| `env` | object of strings | `{}` when empty; serialized from a sorted map so the JSON is stable. |
| `model`, `effort` | string or `null` | `null` is the agent's default. |
| `runtime_options` | object | The shape `runtime_options_json` uses on roles and slots, `{}` when there are none. Today it carries only Codex speed. |
| `system_prompt` | string or `null` | The role's own prompt text; `null` for sessions without a role. |

Not in it: the runtime (the row's `agent_runtime`, decision 1), the working directory (the row's `cwd`), the handle and display name, the resolved executable and app-wide permission mode (decision 6), the mission's prompt parts (decision 3), the native session key and terminal size (their own columns).

In Rust it splits the way `Role` and `RoleRow` do (`repo/README.md`):

- **The value**, `LaunchDefinition` in `crates/runner-daemon/src/model.rs`: `command`, `args`, `env` (a `BTreeMap`), `model`, `effort`, `codex_speed` and `system_prompt`, shaped like the matching `Role` fields. It has no version and no JSON shape.
- **The storage**, in `repo/session.rs`: a private version 1 row type with the stored JSON's fields, `From` conversions both ways, and the column adapter. It writes `codex_speed` as `runtime_options`, reusing `repo/serde.rs`'s `runtime_speed_json` shape, and a read of any other version fails as a mapping error. `repo/README.md`'s storage table gains a row for it.
- **The composition**, in a new `session/launch_definition.rs`: `from_effective(role: &Role) -> LaunchDefinition` takes the role after overrides and before `resolve_role_executable` and copies configured args, including permission flags; `to_role(def, runtime, identity) -> Role` copies the runtime and the launch fields onto the identity role (the role row, or the one `runtime_direct_role` builds), which the existing launch code then takes as it takes a role today; and the per-row conversion `0026` uses.

## Where it lands

- **Migrations.** `0026_session_launch_definition.sql` is `ALTER TABLE sessions ADD COLUMN launch_definition_json TEXT;`, with a `backfill_0026_launch_definitions` step beside the 0014 and 0015 ones in `db/migrations.rs` (177–181). The step reads the old columns with its own `SELECT`, since they exist only until `0027` and never enter `SessionRowDb`; composes each row through `session/launch_definition.rs`; and writes through `repo::session::set_launch`. `0027_drop_unused_session_columns.sql` is the nine `ALTER TABLE sessions DROP COLUMN` statements. Both are registered in `MIGRATIONS` (72) with comments like the others, and the comments on 0003, 0019 and 0024 say which of their columns 0027 dropped. `migrations/schema.sql` is regenerated in the same commit, as `migrations/README.md` says; both came with #879, merged as `7fd40b9c`.
- **`repo/session.rs`**, the only code that touches these columns:
  - `SessionRowDb` gains `launch_definition: Option<LaunchDefinition>`, stored in `launch_definition_json`, and loses the nine fields (66–82, and `new_running` and `COLUMNS` to about 153). `insert`, `get_row`, `get_direct` and `roster_rows_for_mission` (864) return it typed, so spawn, fork, resume, the screen and the roster converter read fields, never JSON.
  - `set_launch(conn, id, runtime, &LaunchDefinition)`: one `UPDATE` of `agent_runtime` and the definition together, so a row with a definition always has its runtime. Restart and `0026` use it. Restart commits the definition and runtime with the native key and launch generation in `resume_in_place`'s mandatory repository transaction before spawning; a failed spawn leaves a consistent stopped row for Resume.
  - `record_process(conn, id, pid, executable)` replaces `update_runtime_metadata` (303): `pid` and `agent_command` together, at every launch (decision 9).
  - `list_orphan_process_candidates` (1030) is unchanged.
  - Repo tests that set or assert the dropped fields (about 1125–1129, 1155–1156 and 1267–1280) change with them, and new ones cover the version 1 round trip, the refused version and `set_launch`.
- **`session/manager/spawn.rs`:**
  - `register_mission_session` (786): record `from_effective` of the role after slot overrides and speed (about 806–822), before `apply_mission` and `resolve_role_executable` (826); the row insert writes it and `agent_runtime` instead of the model, effort and speed fields (934–945).
  - `spawn_direct_inner` (1512): the same, recording before `resolve_role_executable` (1535); the insert at 1636–1646 changes the same way.
  - Fork (about 1911–2040): with a source definition, copy it and `agent_runtime` to the new row and launch from it instead of loading the role template (2036–2040).
  - `resume_with_fresh_fallback`: replace "Pull the role config fresh" (about 2517–2558) with `definition.to_role(runtime, identity)` when `fresh` is false, and with the spawn composition plus `set_launch` when `fresh` is true (decision 5). Everything after it (permissions at 2561–2566, the conversation probe, the prompt at about 2739, the launch) keeps working from that role, so pi and the fresh-conversation fallback read the recorded prompt.
  - The four `update_runtime_metadata` calls become `record_process` with the resolved executable the launch ran.
- **`ops/session.rs`.** `get_direct` (456) and `direct_entry_from_repo` (318) fill model, effort and command from the definition, with no role lookup; `session_get_with_status` (425) sets `agent_model`, `agent_effort` and `agent_speed` from it for every row. A mission row's JSON is still the serialized `SessionRowDb`, from which it removes the `launch_definition_json` key (its env can hold keys; `#[serde(skip_serializing)]` would also drop it from `insert`, which serializes the same struct), so `runner session show --json` keeps those three keys, now with recorded values, and loses only `runtime`, `runtime_session` and the tmux keys. A row without a definition shows its `agent_runtime` and `agent_command` and no model, effort or speed.
- **`runner-cli/src/output.rs`.** `render_session_show` (577) prints the recorded speed, or `default (Codex config)` when it is `null`; `(session)` and `inherit (role or slot)` go, because a session no longer inherits.
- **`ops/mission/roster.rs`.** In Phase 1's converter, a version 1 entry whose crew slot was deleted (156) takes its model, effort and speed from the session's definition; they become that roster slot's overrides, so Restart launches what the session last ran. The converter runs at `runnerd` start, after the migrations.
- **Other tests and fixtures** that write the dropped columns in SQL change with them.
- **`docs/arch/arch.md`:** §3.5 Session and §10.1 SQLite, for what a session records and what Resume, Restart, fork and the screen read.

## Conversion

`0026`'s Rust step, for every row, archived rows included, inside the migration's transaction. A row it cannot convert is logged and left empty rather than failing the migration; Resume then refuses it with the named error. It uses the composition today's resume uses (`resolve_runtime_override`, the speed rule, the runtime catalog, `runtime_direct_role`), so each row gets the definition its next resume would have used:

- **With a role:** the role row, then `resolve_runtime_override` with the row's `agent_runtime`, `agent_model` and `agent_effort`, then speed as the row's speed or the role's, then `from_effective`. `agent_runtime` is set to the effective runtime where it is empty. A row whose role is missing is logged and skipped; role deletion deletes its sessions, so none should exist.
- **Without a role:** model and effort from their columns, speed from `runtime_options_json`, args from `runtime_direct_role`, empty env, no prompt. The command: a shell keeps its recorded path; an absolute `agent_command` (the resolved path today's spawn stored) becomes the runtime's catalog command, so the executable resolves live from now on; any other recorded command is kept.

## Tests

Beside `session/manager/tests/resume.rs`, `slot_restart.rs`, `runtime_override.rs` and `fork.rs`, which already capture what a launch runs:

- Spawn records the definition for a mission slot with overrides and for a direct chat with chat overrides: the configured command, configured args including permission flags, env, effective model, effort and speed including `null`, and the prompt text; `agent_runtime` holds the runtime and the definition does not.
- After the role's command, args, env, prompt, model, effort, speed and runtime are edited, Resume launches from the definition, for a direct chat and a mission slot, including the fresh-conversation fallback and a pi session's prompt, and `session_get` and `session_get_with_status` still show the recorded model, effort, speed and command.
- Restart launches from the current role and the slot's overrides, and writes the new definition and `agent_runtime`.
- The live parts apply on Resume: an executable override set after spawn, a mission permission mode changed after spawn, and permission flags stripped for direct chats. After any launch, `pid` and `agent_command` name the process it started.
- Fork copies the source's definition and `agent_runtime`.
- Migrations, on a database at `0025`: a role-backed row with and without overrides, one with an empty `agent_runtime`, a runtime-only row with an absolute and a custom command, a shell row, an archived row, all nine legacy columns set, and a `session_attention` row. After migrating, each row has the definition its old resume would have used and an `agent_runtime`, the nine columns are gone, the attention row is intact, `PRAGMA foreign_key_check` is clean, and each row resumes with the argv today's code gives it. A row left empty is refused with the named error.
- Phase 1's roster converter, on a version 1 roster whose crew slot was deleted, takes the session's model, effort and speed from its definition.
- `runner session show` on a direct chat and a mission session after a role edit: text and `--json` show the recorded model, effort and speed, and the JSON has no `launch_definition`.
- The definition refuses an unknown version.
- The established argv goldens (`session/manager/tests/golden.rs`) are unchanged.

## Live checks

The brief's QA section: a direct chat and a mission slot whose role is edited after spawn, Resume keeping the old values on screen and in the agent and Restart taking the new ones, a session started by the previous build resumed after the upgrade, the macOS cases of the missions regression suite (Jason's request, 2026-10-12), and SET-AGENT-02, SET-AGENT-03 and RT-LIFE-07's Codex row.

## Risks

- **A direct chat cannot take up a role edit.** It has no Restart; a new chat uses the edited role. Accepted with the rule.
- **Env values can hold keys**, and each session row now carries a copy, in the same database the role already uses. A rotated key reaches a stopped session only through Restart.
- **Runtime-only chats resolve their executable live from now on** (conversion rule above). Today a recorded absolute path wins while it is still valid, so a chat could switch to a newly configured executable on its next resume. That is the rule's intent.
- **The migration step calls product code**, as `backfill_0014_nodes` does. A later change to the composition it calls changes what `0026` produces for a database that skips several versions, so the step calls only what this plan lists.
- **No way back to an older build.** After `0027`, a build from before this change fails on its first query that names a dropped column, as builds before `0023` already fail on its renames.
- **Prompt text per row.** A role's prompt is copied into every session spawned from it. Prompts are a few kilobytes, so the database grows little.

## Mission shape

One codex trio mission, ending in one PR against `main` with one commit, this plan and the brief folded in. `qa` runs the live checks in the development app on macOS; Windows is Jason's.
