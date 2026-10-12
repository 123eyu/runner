# 562 m2 — each mission session independent of the others

Make one mission session's exit or failed spawn never stop another. This is decision 9 of #562's Phase 2 plan, shipped ahead of Phase 2 at Jason's request on 2026-10-12, as a codex trio mission that ends in an open PR. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-562-independent-sessions`, on `fix/562-independent-sessions`, cut from `origin/main` at `a351e368`; its tip is this brief. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## Why

When a mission session exits without being killed, the exit epilogue kills every live sibling and cancels the queued spawns (`reap_live_mission_siblings`, `crates/runner-daemon/src/session/manager/lifecycle.rs` 230, called from `session/manager/output.rs` 384). Mission start's background spawn loop does the same when a slot fails to spawn (`ops/mission.rs` 860, comment at 803). This came from #346, fixing #338, when a dead slot showed one paused card over the whole mission while its siblings ran on unseen. #542's per-slot Stop, Resume and Restart replaced that card: the app already gives a stopped slot its own overlay with Resume slot and Restart slot while siblings are live (`runner-app/src/surfaces/mission_workspace/terminal_pane.rs` about 280–300), and shows the mission-paused overlay only when no session is live (`view.rs` 876–881). With #562 Phase 2, roles join running missions, and a newcomer that dies at boot must not take the team down. Jason's rule: each session is independent of the others.

## Deliverables

1. Remove `reap_live_mission_siblings` and both calls. A session that exits by itself is recorded stopped or crashed as today, and nothing else happens to the mission. At mission start, a slot that fails to spawn is still marked crashed and emits `session/exit` (`ops/mission.rs` 843–868), and the remaining queued slots still spawn.
2. Unchanged: the mission stays `running`; Stop all, mission stop, archive and role deletion still kill every session (`kill_all_for_mission`, `kill_all_for_role`); the per-mission cancel flag still stops queued spawns on Stop and Archive; `was_killed` handling; the synchronous register-phase rollback at mission start (the comment at 558 is about that phase and stays true).
3. Tests in `session/manager/tests/mission_lifecycle.rs`: invert `mission_slot_exit_reaps_live_siblings_and_keeps_mission_running` (297) into a slot exit leaving its siblings live and the mission running, and `mission_slot_exit_cancels_pending_sibling_spawns` (389) into the queue continuing after a slot exits. `intentional_mission_kill_does_not_reap_siblings_from_exit_epilogue` (433) loses its subject; delete it or fold its useful assertion into the inverted test. `spawn_failure_after_spawn_command_reaps_the_child` (608) is about the failed child process and stays. Add a test that a spawn failure during mission start leaves the later slots spawning, if the existing harness can drive that loop; say so in the handoff if it cannot.
4. Docs: in `docs/arch/arch.md` (§ about mission slot lifecycle, around 368) state the rule that one session's exit or failed spawn never stops another, and correct any other text that describes the old reap.
5. The test record `docs/tests/562-independent-sessions.md`: changed behaviours, selected checks with reasons, automated results with exit codes, and QA's matrix.

## QA

After a clean review, the coder hands the frozen candidate to `qa`. Jason authorized `qa` on 2026-10-12 to run mission regression checks for this change: one native pass on macOS in the **development** app with computer use, following `docs/tests/full-smoke-test.md` for setup, candidate identity, agent configuration isolation, evidence and cleanup. Bounded missions with real agents on Jason's existing sign-ins, in a fresh project folder under `/tmp`, with tiny prompts. Selected checks:

- **IND-1, the change.** A three-slot mission (lead and two workers, in a disposable crew of disposable roles). Once all are live, end one worker's agent from inside its terminal (`/exit` or the runtime's quit). Only that card shows stopped, with its own Resume slot and Restart slot; the other two stay live; a composer message to the lead gets a reply; the mission stays running; Resume slot brings the worker back with its conversation (a marker it was told earlier).
- **IND-2, a crash.** Send SIGKILL to the agent process of one of QA's own test sessions (its pid from `runner session show <id> --json`, never any other process). Only that card shows crashed; the siblings stay live.
- **IND-3, the last one out.** Stop the slots one by one. The mission-paused overlay with Resume and Archive appears only after the last; mission-wide Resume brings every slot back.
- **IND-4, spawn failure at start.** If QA can make one slot of a disposable crew fail to spawn (for example a role on a runtime that is not installed), the other slots start and the failed card shows crashed. If the app cannot express it, mark Not run; the automated test covers it.
- **Regression:** MIS-CREW-01 and MIS-LIFE-01 from `docs/tests/regression/missions.md`, because this change touches mission start's spawn loop and the slot exit path.

QA creates and archives only its own test roles, crews and missions, and never touches Jason's production app or its missions. If another dev app or a dev daemon owned by someone else is running, QA reports Blocked and does not stop it. Native Windows is unavailable; record it as unverified.

## Validation

Run each, log to a file, and read the exit code from the command itself, never through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

Gate any import or helper that only `cfg(unix)` tests use with `#[cfg(unix)]`.

## Boundaries

The coder and reviewer never run the dev app. No slot starts, stops, restarts or types into Jason's production Runner, its chats or missions; this mission itself runs there. No extra agents, crews or subagents beyond QA's own bounded test missions in the development app.

## Review, verification and authorization

The coder owns implementation, tests and fixes. The reviewer waits for an explicit Runner handoff, then reviews the whole working-tree diff against this brief, must-fix first with file:line pointers. It checks in particular: no caller of the reap remains; Stop all, mission stop, archive and role deletion still kill everything; Stop and Archive still cancel queued spawns; a start-time spawn failure still marks crashed and emits the exit event but no longer cancels the queue; mission status handling is unchanged; inverted tests assert the new behaviour rather than being deleted. Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`, then QA as above.

After clean review and QA, Jason authorizes one commit on top of current `origin/main` with this brief and the test record folded in, with a subject that names the change (for example `fix(session): keep a mission's sessions independent`); pushing `fix/562-independent-sessions`; and opening a PR against `main` whose body says `Part of #562`, with the summary, the checks and exit codes, the QA matrix, and what waits for Jason (Windows), with no agent session links. Review or CI fixes after the push are folded into that commit and pushed with `git push --force-with-lease`. Drive CI green on macOS and Windows. Do not merge, delete the branch or worktree, or cut a nightly or release.

Final Runner handoff to everyone: PR URL, CI result, what changed, checks with exit codes, the QA matrix, the reviewer's verdict, and anything left for Jason. Then all slots stand by.
