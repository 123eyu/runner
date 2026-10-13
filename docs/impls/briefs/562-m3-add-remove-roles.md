# 562 m3 — add and remove roles while a mission runs

Implement Phase 2 of [#562](https://github.com/yicheng47/runner/issues/562): the person adds a role to a running mission or removes a worker from it, from the mission rail or the `runner` CLI. Jason asked for a codex trio mission ending in an open PR; starting it is his go for everything here, `qa`'s live checks included. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-562-add-remove-roles`, on `feat/562-add-remove-roles`, based on `main` `3a628e50`. No other branch or checkout, no other worktree or its target directory. If you need a newer main, rebase onto `origin/main`; never merge it in.

## What matters most

Get the architecture right for the main use case and keep edge cases simple (Jason). The main use case: the person adds a role with a task, it boots with the system introduction and its task, and the team reaches it with `--to` and broadcasts; the person removes a worker, its session stops, `--to` it is refused, broadcasts skip it and its history stays in the feed; both survive a `runnerd` restart. `roster.json` is the one roster, the log records every join and leave, the ops update the router directly, and the bus learns the team from the log, so live and replay are one code path. Edge cases get the plan's handling and no more; do not add machinery the plan does not ask for.

## Read first

- `AGENTS.md`: Worktrees, Test Scope, Crew Missions.
- **`docs/impls/562-mission-spawn/plan.md`, Phase 2: the contract.** Decisions 1–15, the event contract, where each change lands (line numbers as of `3a628e50`), tests, live checks and risks. It wins over this brief and the spec on any detail. Also that directory's `README.md` and the spec, `docs/features/562-mission-spawn.md`.
- **The design**, as PNGs in this worktree's ignored `target/design-ref/`: `562-add-role-rail.png` (`vfnFl`), `562-add-role-dialog.png` (`V61gij`), `562-handle-refused.png` (`E8wxa`), `562-role-joined.png` (`IPOAI`), `562-remove-confirm.png` (`E0HNqg`), `562-role-removed.png` (`zd1d4`). Match layout, hierarchy and copy with the theme tokens; sample data is illustrative. Where a frame and shipped code disagree on a shared element, shipped code wins (say so in the handoff). Do not open `.pen` files.

## Deliverables

One PR against `main` in three commits, each building and passing its tests, with the program record and this brief folded into the first:

1. **The handle rule in `runner-core`**, moved from `crews/logic.rs`, with the "already used" scope a parameter and the removed-handle error added. The crew page and its tests are unchanged.
2. **Daemon, protocol and CLI**, as the plan's `runner-core`, `runner-daemon` and `runner-cli` sections list them, with `docs/arch/arch.md` §3, §7 and §8 and the spec's Phase 2 bullets and Decisions to confirm revised to plan decisions 4, 6, 9, 10 and 15.
3. **The app**, as the plan's `runner-app` section lists it. Start chat's role and setup fields move into a module both forms use, and the Start chat tests pass unchanged.

The last commit adds `docs/tests/562-add-remove-roles.md` (selected checks with reasons, results with exit codes, `qa`'s matrix, and the two new regression cases written out for the release sweep) and brings the program record's status and `impl_log.md` up to date.

Easy to get wrong:

- `role_joined` is appended before `roster.json` is written, and the router is updated before the introduction is appended, so the introduction nudges the newcomer.
- Add and remove are refused while any spawn of the mission is pending; add registers its cancel flag right after its checks (decision 15).
- `runner msg read` is unchanged; only the bus's unread count starts at the join (decision 4).
- New code reads the lead from a slot's `lead` flag, never `Router::lead_handle()` or `LaunchInputs::lead()` (decision 14).
- The add's overrides are the new roster slot's overrides, so Restart rebuilds the newcomer with them (#878).
- The CLI flag is `--handle`; both commands refuse a caller inside a mission and `--as` (decisions 8, 10).
- CLI goldens: in `crates/runner-cli/tests/goldens/cli.txt`, both commands in human and `--json` output and their refusals, under about 150 lines.

## QA

After a clean review the coder hands the frozen candidate to `qa`: one native macOS pass in the **development** app with computer use, following `docs/tests/full-smoke-test.md` for setup, candidate identity, configuration isolation (Codex trust entries for its own folders are pre-approved there), evidence and cleanup. Bounded missions on real agents, disposable roles and crews, a fresh `/tmp` folder, tiny prompts. Checks (the plan's Live checks; read its two lessons from #881's run first):

- **ADD-1.** Add a role from the rail with a task: it boots, reads the introduction and the task, and posts to the lead; the lead reaches it with `--to`, and a broadcast reaches it.
- **ADD-2.** Restart `runnerd` (quit the development app, `runner-dev daemon stop`, launch the same candidate): the newcomer is still on the team and resumes, and its unread count excludes messages from before its join.
- **REM-1.** Remove a worker from the rail: its session stops, `--to` it is refused, broadcasts skip it, its history stays in the feed, Resume all leaves it out, and all of this holds after another restart.
- **CLI-1.** The same through `runner mission add-role` and `remove-role`, and both refusals from inside a mission session.
- **Regression:** MIS-CREW-01, MIS-LIFE-01, MIS-MSG-01, MIS-MSG-02, MIS-FEED-01 and MIS-ROSTER-01.

Compare each changed surface with its frame and attach screenshots. `qa` archives only its own roles, crews and missions, never touches the production app or pre-existing development sessions, and reports Blocked if someone else's development app or daemon is running. Native Windows is unverified.

## Validation

Log each to a file and read the exit code from the command itself, never through `tail` or `grep`: `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`; `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`; `cargo fmt --all --check`; `git diff --check`. Gate imports and helpers used only by `#[cfg(unix)]` tests with `#[cfg(unix)]`.

## Boundaries

The coder and reviewer never run the development app. No slot operates Jason's production Runner, its chats or missions; this mission runs there. Never open a Runner database. No agents, crews or subagents beyond `qa`'s bounded test missions.

## Review and authorization

The reviewer waits for an explicit Runner handoff, then reviews the whole diff against the plan, the PNGs and this brief, must-fix first with file:line pointers. Must-fix: anything that breaks the main use case, leaves `roster.json`, the log and the router disagreeing, or leaves an agent running that the person cannot see or stop; other edge cases go in the handoff as follow-ups. Check especially the add and remove step order, payloads and rollback; that a restart rebuilds the same team from the file and the log; that broadcasts reach exactly the team at that point and directed messages are never dropped; decision 15; the lead only through a slot's flag; and that `msg.rs`, the crew page and Start chat behave as before. Iterate until `NO REMAINING MUST-FIX ISSUES`, then QA.

After clean review and QA, Jason authorizes: the three commits on current `origin/main` with imperative subjects and no co-author trailers; `git push --force-with-lease` of `feat/562-add-remove-roles`; `gh pr create --base main` with `Part of #562`, the summary, checks with exit codes, the QA matrix, review follow-ups, what waits for Jason (Windows) and no agent session links; and `gh pr checks <n> --watch` until macOS and Windows are green, folding each fix into the commit it corrects after the reviewer checks it. Do not merge, delete the branch or worktree, or cut a nightly or release.

Final handoff to everyone: PR URL and CI, changes, checks with exit codes, the QA matrix, the review verdict, follow-ups and what is left for Jason. Then all slots stand by.
