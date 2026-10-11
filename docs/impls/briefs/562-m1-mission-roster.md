# 562 m1 — the mission roster (fixes #863)

Deliver Phase 1 of [562](../../features/562-mission-spawn.md), which fixes [#863](https://github.com/yicheng47/runner/issues/863): a running mission reads its roster from its own `roster.json` instead of its crew's current slots, so editing a crew never changes a mission that has already started. Jason requested this mission on 2026-10-11 as a 0.15.x patch. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-863-mission-crews`, on the branch `fix/863-mission-crews` (the name predates the roster design; keep it), cut from `origin/main` after the 0.15.1 release. The spec revision and this brief are committed on the branch and are folded into the mission's commit. The root checkout stays on main. Do not create another branch or worktree, and do not share a Cargo target directory.

The spec's Phase 1 section, with Model, The roster file and the Phase 1 bullets of Scope, is the contract. Phases 2 to 4 are out of this mission.

## Decisions already made (Jason, 2026-10-11)

- The roster lives in the mission's `roster.json`, not in database rows. No schema change, no `crews.kind`, no migration.
- The file is version 2 as the spec shows it: `crew_name`, `crew_addendum` and the slots with their crew ids, handles, lead flags, positions, role ids and overrides. Its type lives in `runner-core`.
- Version 1 files are converted once when `runnerd` starts, in one function that is the only code reading version 1. Every other reader, the CLI included, knows only version 2. A missing or unreadable file at load falls back to the crew's current slots and is written as version 2 with a warning.
- A role cannot be deleted while an unarchived mission has a session for it.
- Crew slot operations, crew deletion, crew lists and the app's rail Crew row are unchanged.
- A role edit still reaches a running mission at its next resume. Copying role configuration into the roster is a later follow-up.

## Where the roster is read today

- `crates/runner-daemon/src/ops/mission.rs`: `start` (about line 163) reads the crew's slots at 181 and again inside its transaction at 243 for `write_roster_sidecar` (344), which writes only handle and lead; `mission_start_impl_with_size` reads the crew row at 610 and the slots a third time at 616, after the commit, to compose prompts and spawn; `validate_roster_handle` (404); `post_message` (the `--to` check about line 500); `ensure_mission_router_mounted` (964: crew row at 995, slots at 1001, then `latest_unarchived_id_for_slot` per slot).
- `crates/runner-daemon/src/session/manager/spawn.rs`: the resume path's `MissionCtx` reads the slot row by id (about line 2489) and fails with "slot … no longer exists"; the lead's prompt on resume reads the crew row and `slot::list` (about 2730–2750).
- `crates/runner-daemon/src/daemon/boot.rs:94`: `mount_all_running_mission_routers` at startup. The conversion runs before it.
- `crates/runner-daemon/src/ops/role.rs`: `ensure_delete_allowed` (about 358) and `delete` (382), which deletes every session of the role.
- `crates/runner-cli/src/roster.rs`: parses `Vec<RosterEntry { handle, lead }>`; a parse failure allows `--to` with a warning.
- `crates/runner-app/src/surfaces/mission_workspace/attach.rs:89`: `slot_list(&mission.crew_id)`.
- `crates/runner-core/src/protocol/api.rs:118`: the `slot_list` declaration, the pattern for `mission_roster`.

## Deliverables

1. **The roster type** in `runner-core`: version 2, serialized as the spec shows.
2. **The roster module** in `runner-daemon` (for example `ops/mission/roster.rs`): write at start, atomically as today; load one mission's roster as `Vec<SlotWithRole>` with each role from the database, skipping a slot whose role is gone; the fallback for a missing or unreadable file.
3. **Start.** `start` reads the crew's slots once, inside its transaction, writes the roster, and `mission_start_impl_with_size` composes prompts, builds the router and spawns from that roster rather than reading the crew again.
4. **Readers.** `ensure_mission_router_mounted`, `validate_roster_handle`, `post_message` and the resume path in `spawn.rs` read the roster: slots, handle and lead flag, crew name and addendum. A session whose slot the crew deleted resumes.
5. **`mission_roster(mission_id)`** on the client protocol, returning `Vec<SlotWithRole>`; `attach.rs` uses it.
6. **The startup conversion**, called in `boot.rs` before the routers mount, over every mission, running or archived. Follow the spec's matching rules exactly: each handle to the slot its session ran under, first by current handle, then in the order the sessions first started; the session's own role, runtime, model, effort and speed stand in for a deleted slot; a handle without a session takes the crew's slot of that handle; a handle with neither is dropped with a warning; the file's handles, order and lead flags win. Version 2 files are skipped. One mission's failure is logged and does not stop the others or the boot.
7. **Role deletion** refused while an unarchived mission has a session for the role, naming the missions, beside today's two refusals.
8. **CLI.** `roster.rs` parses version 2 only, and its tests use version 2 files.
9. **Docs.** `docs/arch/arch.md`: the mission roster in §3 and `roster.json` in §10's directory layout. No other doc changes; the spec is already written.
10. **Test record** `docs/tests/562-mission-roster.md`: Jason's live checks on macOS and Windows, which are #863's reproduction (start a mission from a crew with a lead and two workers; while it runs remove one worker's slot, add a slot, rename a handle and edit the addendum on the crew page; restart `runnerd`; then the removed worker posts with `runner msg post --to lead`, the lead broadcasts with no warning about the added handle, and every slot resumes), an upgrade from 0.15.1 with a running and an archived mission whose crews were edited, and the regression cases MIS-CREW-01, MIS-LIFE-01, MIS-CONFIG-01 and MIS-MSG-01.

## Tests

Write the tests listed under the spec's Phase 1. Use roles with `runtime: "shell"` as fake agents, as the existing daemon tests do, and temporary app data directories. Simulate a `runnerd` restart by unmounting the router and bus and calling `ensure_mission_router_mounted` again. The conversion tests write version 1 files and database rows by hand for each case in the spec, run the conversion twice, and assert the resulting files. Keep fixtures inline in the tests; no golden files.

## Boundaries

Crews never run the dev app. Do not run `runner` or `runner-dev` commands that start, stop or kill sessions, missions or the daemon: this mission runs inside Jason's production Runner. Never read or write the real app data directories (`~/Library/Application Support/com.wycstudios.runner` and its `-dev` twin); tests use temporary directories. Do not start extra agents, crews or subagents. Imports, statics and helpers used only by `#[cfg(unix)]` tests get `#[cfg(unix)]` themselves; macOS Clippy cannot catch this and Windows CI will.

## Review, verification and authorization

The coder owns implementation and checks. The reviewer waits for an explicit Runner handoff, then reviews the full branch diff against the spec's Phase 1, must-fix findings first with file:line pointers. Focus on: no remaining read of `slot::list(mission.crew_id)` or the crew row for a mission's roster, name or addendum outside the conversion and the fallback; the conversion's matching against the spec's cases and its placement before the routers mount; atomic writes; the role-deletion guard; and no change to crew editing, crew deletion or the app's visible behavior. Iterate until the reviewer posts `NO REMAINING MUST-FIX ISSUES`.

Locally, run `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, the same with `--features updater`, `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`, `cargo fmt --all --check` and `git diff --check`. Record the exact commands and exit codes.

After a clean review, Jason authorizes the following:

- Squash all work on this branch into one commit on top of current `origin/main`, the spec revision and this brief included, with a subject that names the change (for example `fix(mission): keep each mission's roster in its own file`).
- Push `fix/863-mission-crews` and open a PR against main whose body says `Closes #863` and `Part of #562`.
- If main has moved, rebase; never merge main into the branch.
- Amend review or CI fixes after the push into the same commit and push with `git push --force-with-lease`.
- Drive CI green on macOS and Windows.

Do not merge, delete the branch or worktree, or cut a nightly or release.

The final Runner handoff covers the PR URL and what changed, the commands and exit codes, the CI result, the reviewer's verdict, anything in the conversion that the spec did not decide, and a pointer to the test record for Jason's live checks. Then stand by.
