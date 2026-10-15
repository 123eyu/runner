# 562 m4 — start a mission empty

Implement Phase 3 of [#562](https://github.com/yicheng47/runner/issues/562) with [#849](https://github.com/yicheng47/runner/issues/849)'s Start mission redesign: a mission can start with no crew and no roles, roles join one at a time through Phase 2's add-role, and its members work with each other without a lead. Jason asked for a codex trio mission ending in an open PR; starting it is his go for everything here, `qa`'s live checks included. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-562-start-empty`, on `feat/562-start-empty`, based on `main` `cf839e16`. No other branch or checkout, no other worktree or its target directory. If you need a newer main, rebase onto `origin/main`; never merge it in.

## What matters most

The main use case: a chat or the person starts a mission with no crew, adds a role with a task, and the role boots with the goal and its task; a second role joins, the two reach each other with `--to` and broadcasts, the person's composer message reaches both, and all of it survives a `runnerd` restart. Missions started from a crew behave as before, apart from the lines decision 16 changes. The database never deletes: the app removes dependent rows in code, and the foreign keys only refuse. Edge cases get the plan's handling and no more.

## Read first

- `AGENTS.md`: Worktrees, Test Scope, Crew Missions.
- **`docs/impls/562-mission-spawn/plan.md`, Phase 3: the contract.** Decisions 1–16, where each change lands (line numbers as of `cf839e16`), tests, live checks and risks. It wins over this brief and the specs on any detail. Also that directory's `README.md`, the spec `docs/features/562-mission-spawn.md`, and `docs/features/849-start-mission-modal.md`.
- `crates/runner-daemon/migrations/README.md`, before touching the schema, and commit `da4333ac` for how deletes clean up in code.
- **The design**, as PNGs in this worktree's ignored `target/design-ref/`: `562-no-roles-yet.png` (`PhtPw`), `562-start-from-crew.png` (`AVJ15`), `562-start-no-crew.png` (`m2Noaa`), `562-crew-picker-open.png` (`iJuGG`), `562-first-role-joined.png` (`BsErp`), `562-info-no-crew-row.png` (`blE2u`). Match layout, hierarchy and copy with the theme tokens; sample data is illustrative. Where a frame and shipped code disagree on a shared element, shipped code wins (say so in the handoff). Do not open `.pen` files.

## Deliverables

One PR against `main` in five commits, each building and passing its tests, with the program record, the specs and this brief folded into the first:

1. **The migration runner's foreign-keys-off mode** (decision 5), with its test that rebuilding a referenced table keeps its children.
2. **Migration 0028** (decisions 4 and 6): `missions.crew_id` nullable with no reference; `slots`, `sessions` and `session_attention` with `RESTRICT` instead of `CASCADE`; `schema.sql` regenerated; the version 27 test; the pinned foreign-key list in `ops/tests.rs`; the DDL rules in `migrations/README.md`.
3. **Core, daemon and CLI**: the optional crew in the protocol, the start, the directory, the envelope and the environment; the no-lead branches; `mission_status` with an optional crew; crew deletion that keeps missions; `ask_lead` removed; `runner mission start` without `--crew`; `docs/arch/arch.md` as the plan lists.
4. **#849's Start mission redesign**, crew mode only, closing #849.
5. **The No crew entry, the empty workspace and the info panel**, as the plan's `runner-app` section lists them.

The last commit adds `docs/tests/562-start-empty.md` (selected checks with reasons, results with exit codes, `qa`'s matrix, and the new regression cases written out for the release sweep), rewrites MIS-ASK-01 in `docs/tests/regression/missions.md`, and brings the program record's status and `impl_log.md` up to date.

Easy to get wrong:

- 0028 rebuilds `sessions` and `missions`, which other tables reference. Copy every column, run it only in the foreign-keys-off mode, and prove with the version 27 test that every `sessions.mission_id` survives.
- `mission_dir` is fixed at start: `crews/<crew_id>/missions/<id>/` for a crew start, `missions/<id>/` for an empty one. Never move an existing mission's files.
- A crew mission's prompts change only in decision 16's two lines; update the literal tests for exactly those.
- `human_said` is the only path that loses its lead fallback; the composer already broadcasts (decision 8).
- Old `ask_lead` events keep parsing and rendering; only sending one is refused.
- `runner mission show` must work for an empty mission and for one whose crew was deleted.

## QA

After a clean review the coder hands the frozen candidate to `qa`: one native macOS pass in the **development** app with computer use, following `docs/tests/full-smoke-test.md` for setup, candidate identity, configuration isolation (Codex trust entries for its own folders are pre-approved there), evidence and cleanup. Bounded missions on real agents, disposable roles and crews, a fresh `/tmp` folder, tiny prompts. Read the plan's two lessons from #881's run in Phase 2's Live checks first. Checks:

- **UPG-1.** Before launching the candidate, copy the development database aside. Count crews, slots, roles, missions, sessions, sessions with a mission and attention rows with read-only queries on a copy, never the live file; launch the candidate, which migrates; count again on a fresh copy. Every count matches, and existing development missions and chats resume.
- **EMPTY-1.** Start a mission from the dialog with **No crew**: the empty state shows, with no paused overlay, and the composer is disabled. Add a role with a task: it boots with the goal, reads the task and posts. Add a second: a composer message without a recipient reaches both, and the two coordinate through `--to`.
- **EMPTY-2.** Restart `runnerd` (quit the development app, `runner-dev daemon stop`, launch the same candidate) with one empty mission and one with a single added role: both come back, and the second resumes.
- **DELEG-1.** From a direct chat, `runner mission start --title … -q` then `runner mission add-role … --task-file -`: the mission appears in the sidebar without taking focus.
- **CREW-1.** Delete a disposable crew while a mission started from it runs: the mission keeps running, posting and adding roles, `runner mission show` works, and its info panel has no Crew row.
- **ASK-1.** `runner ask "<question>"` is refused with its hint, and `runner ask --human` reaches the person's card. An existing mission whose log holds `ask_lead` events still opens and renders them.
- **Regression:** MIS-CREW-01 through the redesigned dialog, MIS-LIFE-01, MIS-MSG-02, MIS-ASK-01 as rewritten, MIS-ROSTER-01 and MIS-FEED-01.

Compare each changed surface with its frame and attach screenshots. `qa` archives only its own roles, crews and missions, never touches the production app or pre-existing development sessions, and reports Blocked if someone else's development app or daemon is running. Native Windows is unverified.

## Validation

Log each to a file and read the exit code from the command itself, never through `tail` or `grep`: `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`; `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`; `cargo fmt --all --check`; `git diff --check`; `RUNNER_UPDATE_GOLDEN=1` only to regenerate `schema.sql` and goldens you changed on purpose. Gate imports and helpers used only by `#[cfg(unix)]` tests with `#[cfg(unix)]`.

## Boundaries

The coder and reviewer never run the development app. No slot operates Jason's production Runner, its chats or missions; this mission runs there. Never open a production Runner database; `qa` reads only copies of the development database, for UPG-1. No agents, crews or subagents beyond `qa`'s bounded test missions.

## Review and authorization

The reviewer waits for an explicit Runner handoff, then reviews the whole diff against the plan, the PNGs and this brief, must-fix first with file:line pointers. Must-fix: anything that loses or detaches data in the migration, breaks the main use case, changes a crew mission's behaviour beyond decision 16, or leaves an agent running that the person cannot see or stop; other edge cases go in the handoff as follow-ups. Check especially 0028 and the foreign-keys-off mode; every `mission_dir` caller; the envelope's serde; the no-lead branches and that crew missions route as before; `mission_status` without a crew; and `ask_lead`'s removal with old events still rendering. Iterate until `NO REMAINING MUST-FIX ISSUES`, then QA.

After clean review and QA, Jason authorizes: the five commits on current `origin/main` with imperative subjects and no co-author trailers; `git push --force-with-lease` of `feat/562-start-empty`; `gh pr create --base main` with `Part of #562` and `Closes #849`, the summary, checks with exit codes, the QA matrix, review follow-ups, what waits for Jason (Windows) and no agent session links; and `gh pr checks <n> --watch` until macOS and Windows are green, folding each fix into the commit it corrects after the reviewer checks it. Do not merge, delete the branch or worktree, or cut a nightly or release.

Final handoff to everyone: PR URL and CI, changes, checks with exit codes, the QA matrix, the review verdict, follow-ups and what is left for Jason. Then all slots stand by.
