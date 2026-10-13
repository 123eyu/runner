# 878 — a session records its launch definition at spawn

Implement [#878](https://github.com/yicheng47/runner/issues/878) (bug, P2, milestone 0.16): a spawned session must not change when its role is edited later. It is the prerequisite for #562's Phase 2 (mission 3, on `feat/562-add-remove-roles`). Jason asked on 2026-10-12 for a codex trio mission that ends in an open PR. Starting this mission is his go for everything in it, including `qa`'s live checks below.

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-878-session-launch-definition`, on `fix/878-session-launch-definition`, cut from `origin/main` and rebased onto it after #879 merged, so it has `crates/runner-daemon/migrations/schema.sql` and `migrations/README.md`. Its tip is this brief and the plan, `docs/impls/878-session-launch-definition.md`. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## The rule

From "What is copied, and when" in `docs/features/562-mission-spawn.md`, settled by Jason on 2026-10-12:

- **A session records its launch definition when it spawns.** Resume replays it exactly, so later role edits never change a session that exists.
- **Restart is the one explicit way to take up role edits.** It starts a fresh conversation rebuilt from the current role and the slot's roster entry, and records the new definition. Restart exists only for mission slots; a direct chat takes up role edits by starting a new chat.
- **The machine stays live.** Resolving the command to an executable, the app-wide mission permission mode (and stripping permission flags for direct chats), and the shell environment that Cursor and Grok probes read are applied at every launch, because they describe the machine, not the agent.

Keep the change to that rule and the column cleanup the plan lists. Role deletion, the roster format and the UI's layout are out of scope (the chat header and side panel change only where their values come from), and the argv for a session whose role was not edited must not change.

## Read first

- `AGENTS.md`, including Worktrees and Crew Missions.
- **`docs/impls/878-session-launch-definition.md`, the plan.** It is the contract: today's behaviour, decisions 1 to 11, the columns afterwards, the definition's JSON and Rust type, where each change lands, the conversion inside migration `0026`, tests and risks. It wins over this brief on any detail.
- `docs/features/562-mission-spawn.md`, "What is copied, and when", for the rule.
- The code the plan points at: `crates/runner-daemon/src/session/manager/spawn.rs` (`register_mission_session`, `spawn_direct_inner`, fork, `resume_with_fresh_fallback`), `session/manager/mod.rs` (`resolve_runtime_override`), `repo/README.md` and `repo/session.rs` (`update_runtime_metadata`, `list_orphan_process_candidates`), `repo/role.rs` (the `RoleRow` pattern the definition's storage follows), `repo/serde.rs` (`runtime_speed_json`), `ops/session.rs` (`get_direct`, `session_get_with_status`), `ops/mission/roster.rs` (`convert_all`), `db/migrations.rs` (`backfill_0014_nodes`) and `crates/runner-daemon/migrations/README.md`.

## Deliverables

One commit, as the plan's "Where it lands" lists it:

1. Migration `0026` adding `sessions.launch_definition_json` and converting every existing row in its Rust step, and migration `0027` dropping the nine session columns the plan lists (the four tmux-era ones, `runtime` and `runtime_session`, `agent_model`, `agent_effort` and `runtime_options_json`), with `schema.sql` regenerated; `LaunchDefinition` in `model.rs`, its storage and every query on the session columns in `repo/session.rs`, and the composition in `session/launch_definition.rs`.
2. Spawn (mission slot and direct chat) and fork recording it; Resume and fork launching from it; Restart rebuilding it from the current role and roster slot and writing it with `agent_runtime`; every launch writing `pid` and `agent_command` together.
3. The chat header, the side panel, `runner session show` (text and `--json`) and Phase 1's roster converter reading the definition instead of the dropped columns. After the migration there is no fallback: a session without a definition is refused with a named error.
4. `docs/arch/arch.md` §3.5 and §10.1, the test record `docs/tests/878-session-launch-definition.md` with the changed behaviours, selected checks and reasons, automated results with exit codes, and `qa`'s LD rows, and `qa`'s regression run record under `docs/tests/runs/`.

Tests are the plan's Tests section. Points that are easy to get wrong:

- The recorded command is the configured one, before `resolve_role_executable`, and the recorded args include configured permission flags; the live launch layer strips or replaces them.
- The recorded prompt is the role's own text; the crew addendum and the lead's launch inputs keep composing at launch.
- A `null` model or effort in a definition means the agent's default, never "follow the role".
- The runtime lives only in `agent_runtime`; the definition does not repeat it.
- Only `repo/session.rs` touches the session columns and the definition's JSON (`repo/README.md`); spawn, fork, resume, Restart, `ops` and the roster converter call its functions and read typed fields.
- Restart is the only path that reads the role's launch fields.
- The conversion runs inside migration `0026`, not at `runnerd` start, because `0027` drops columns it reads.
- The screen and `runner session show` show the definition's model, effort, speed and command, which is what Resume will run; `--json` never carries the definition itself, whose env can hold keys.

## QA

**Superseded by Jason through the mission on 2026-10-12:** live QA is waived; do not run the development app, LD checks or regression cases, and do not create a run record. After clean review, proceed to the one commit, push, PR and CI. Record the waived cases as not run in the test record and PR body.


After a clean review, the coder hands the frozen candidate to `qa`. `qa` runs one native macOS pass in the **development** app with computer use, following `docs/tests/full-smoke-test.md` for setup, candidate identity, agent configuration isolation, evidence and cleanup. Use disposable roles on Jason's existing sign-ins: Codex by default, and Claude Code or pi only where a case names them and they are already installed and signed in. Never install or sign in to anything. Use a fresh project folder under `/tmp` and tiny prompts. Before Runner adds its normal Codex trust entry for that folder, ask Jason through the mission and wait for the answer; remove the entry at cleanup and record a before-and-after diff.

- **LD-1, a direct chat.** A role with env `RUNNER_878_MARKER=before`, an explicit model and effort. Start a chat, have the agent print `$RUNNER_878_MARKER`, stop it. Change the marker to `after`, the model, the effort and the prompt on the role page. Resume: the agent prints `before`, the side panel shows the original model and effort, and the conversation continues.
- **LD-2, a mission slot.** The same edit on a slot's role in a disposable two-slot crew. Resume slot keeps `before` and the original model; Restart slot prints `after` and runs the new model.
- **LD-3, an older session.** If the procedure's candidate identity allows it, a chat started by a build from `origin/main` before this change, then resumed by the candidate, resumes as it would have before the upgrade, and from then on ignores edits to its role. Otherwise mark it Not run with the reason; the automated conversion tests cover it.
- **Mission regression.** Jason asked on 2026-10-12 for the missions regression to run on this change, because it changes how every mission slot spawns, resumes and restarts, and migrates every session row. Run every case in `docs/tests/regression/missions.md` in table order, `MIS-CREW-01` through `MIS-WATCH-02`. His request authorizes the cases' bounded missions and chats and, for `MIS-WATCH-01` and `MIS-WATCH-02`, a fresh Codex host chat in the development app. Record `MIS-MSG-03` (Windows only) as not applicable, and `MIS-MSG-05` and `MIS-DRAFT-06`, which need a human, as Not run, left for Jason. A case whose runtime is not installed or signed in is Blocked with the reason, never worked around.
- **Other regression:** SET-AGENT-02 and SET-AGENT-03 from `settings.md`, and RT-LIFE-07's Codex row from `runtimes.md`, because this change rewrites what executable overrides, Speed and resume read.

`qa` writes the regression results as a run record, `docs/tests/runs/<run date>-macos.md`, in the suite's run-record format (`docs/tests/regression/README.md`); the test record links it and carries the LD rows.

`qa` creates and archives only its own test roles, crews, chats and missions, and never touches Jason's production app or its sessions. If another development app or a development daemon owned by someone else is running, `qa` reports Blocked and does not stop it. Native Windows is unavailable; record it as unverified.

## Validation

Run each, log it to a file, and read the exit code from the command itself, never through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

Imports, statics and helpers used only by `#[cfg(unix)]` tests get `#[cfg(unix)]` themselves; macOS clippy cannot catch this, and Windows CI will.

## Boundaries

The coder and reviewer never run the development app. No slot starts, stops, restarts or types into Jason's production Runner, its chats or missions; this mission itself runs there. Never open Jason's real Runner database. No extra agents, crews or subagents beyond `qa`'s own bounded test chats and missions in the development app.

## Review, verification and authorization

The coder owns implementation, tests and fixes. The reviewer waits for an explicit Runner handoff, then reviews the whole diff against this brief and the spec's rule, must-fix first with file:line pointers. A finding is must-fix when a recorded session can still change with a later role edit, when the machine-level parts stop applying, when a converted session from before this change resumes with different argv than today's code gives it, when argv changes for an unedited role, when the screen or `runner session show` shows a model, effort, speed or command other than what Resume runs, when a session's env reaches CLI output, or when code still reads or writes a dropped column. Other edge cases go in the handoff as follow-ups. The reviewer checks in particular:

- that with a recorded definition, no Resume or fork path reads the role row's runtime, command, args, env, model, effort, speed or prompt;
- that the recorded command is the configured one and configured permission args are recorded, with direct-chat stripping and mission mode applied live;
- that Restart is the only path that reads the current role, and that it records what it launched;
- that migration `0026` gives every existing row, runtime-only chats and shells included, the definition its next resume would use today, and that no fallback to the role remains;
- that the runtime is recorded only in `agent_runtime`, nothing reads or writes the nine dropped columns, and `schema.sql` matches the migrations;
- that no new code outside `repo/` and `db/` prepares SQL on `sessions` or parses the definition's JSON.

Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`, then QA as above.

After the clean review and QA, Jason authorizes:

- **One commit** on top of current `origin/main`, the plan, the brief, the test record and the run record folded in, with a subject naming the change (for example `fix(session): record a session's launch definition at spawn`) and no co-author trailers.
- **Push** with `git push -u origin fix/878-session-launch-definition`.
- **Open the PR** with `gh pr create --base main`. The body says `Closes #878` and that it unblocks #562's mission 3, and carries the summary, the checks and exit codes, the QA matrix, follow-ups from review, what waits for Jason (Windows), and no agent session links.
- **Watch CI** with `gh pr checks <n> --watch` until nothing is pending, on macOS and Windows. Fold each fix into the commit (`git commit --amend`), have the reviewer check it, and push with `git push --force-with-lease`.

**Do not merge**: Jason said on 2026-10-12 that the mission ends at the open PR with CI green, and he merges. Do not delete the branch or worktree, or cut a nightly or release.

The final handoff goes to everyone through Runner: the PR URL and CI result, what changed, the checks with exit codes, the QA matrix, the reviewer's verdict, follow-ups, and anything left for Jason. Then all slots stand by.
