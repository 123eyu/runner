# 878 — live QA for PR #881

Run the live QA that was waived for [PR #881](https://github.com/yicheng47/runner/pull/881) ([#878](https://github.com/yicheng47/runner/issues/878)), fix only what it finds and the human approves, and update the PR. Jason asked on 2026-10-13 for a codex trio mission in which `qa` runs the mission regression on this change and any finding goes back to the coder. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-878-session-launch-definition` on `fix/878-session-launch-definition`. Its tip is this brief on top of the PR's commit `899f2478`, one commit on `origin/main` `47a555f5`, already reviewed clean and green on both platforms. Do not create another branch or checkout except QA's baseline worktree below, touch the root checkout or another worktree, or share another worktree's target directory. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## Read first

- `AGENTS.md`: Test Scope, Full Smoke Tests, Crew Missions.
- `docs/impls/878-session-launch-definition.md`, the plan, and `docs/impls/briefs/878-session-launch-definition.md`, whose QA section defines LD-1 to LD-3. This brief supersedes that section's waiver.
- `docs/tests/878-session-launch-definition.md`, the test record QA completes.
- `docs/tests/full-smoke-test.md`, `docs/tests/regression/missions.md` and `docs/tests/regression/README.md` (run-record format). `docs/tests/runs/2026-10-11-macos.md` is the last run of this shape, with a baseline build and an upgrade.

## Start

The coder checks that the worktree is clean at the expected HEAD and hands the frozen candidate to `qa` at once. The coder and reviewer then stand by; nothing in the worktree changes while QA runs.

## QA

One native macOS pass in the development app with computer use, following `full-smoke-test.md` for setup, candidate identity, agent configuration isolation, evidence and cleanup. Codex only, on Jason's existing sign-in, with an inexpensive model, explicit low effort and tiny prompts, in a fresh scratch root under `/tmp` with one child folder per chat or mission. Never install or sign in to anything.

1. **Back up the development database.** With the development app and daemon stopped (verify it; if one owned by someone else is running, report Blocked and stop), copy `runner.db`, `runner.db-wal` and `runner.db-shm` from `~/Library/Application Support/com.wycstudios.runner-dev/` into the evidence directory without opening them, and record the path. The candidate migrates that database to 0027, which builds from before this change cannot read; the copy is Jason's way back.
2. **Baseline.** Create a detached worktree `.worktrees/qa-878-baseline` at `47a555f5`, with its own `target/`, and run its development app. Create the legacy fixtures: LD-3's direct chat from a disposable role with env `RUNNER_878_MARKER=before`, and for MIS-ROSTER-02 a running and an archived mission from a disposable crew, then edit that crew and role. Quit the app and run `runner-dev daemon stop`.
3. **Candidate.** Run `env -u NO_COLOR make run` in this worktree, which migrates. Then, in order:
   - LD-3 and MIS-ROSTER-02 on this upgrade: the baseline chat and the running mission resume with the conversations and values the baseline gave them, and ignore role edits made after the upgrade; the archived mission still opens; a second daemon start is clean.
   - LD-1 and LD-2 as the 878 brief defines them, including Restart slot taking the edited role.
   - Final mission selection, narrowed by Jason on 2026-10-13: the recorded launch/roster preservation variant of `MIS-ROSTER-01`, `MIS-CONFIG-01`, and `MIS-FEED-01` plus native owned-mission stop/archive divider evidence. The initial request for every mission row is superseded; already completed `MIS-CREW-01` and `MIS-LIFE-01` remain supplementary, and unrelated remaining delivery, draft, route, crash, IME and watch checks are outside scope. Native Windows and non-Codex runtimes remain unverified.

Jason's request authorizes the cases' bounded chats and missions, `MIS-INDEP-02`'s force-stop of QA's own worker by its read-back PID, the daemon restarts the cases need (quit the app, `runner-dev daemon stop`, relaunch the same build), and for `MIS-WATCH-01` and `MIS-WATCH-02` a fresh Codex host chat in the development app. Codex trust entries for disposable test folders follow the standing pre-approval in [Agent configuration isolation](../../tests/full-smoke-test.md#agent-configuration-isolation); at cleanup remove each entry and record a before-and-after diff of `~/.codex/config.toml`.

**Findings.** Report each failure to the coder and the human as soon as it is established: case, steps, expected and actual, evidence paths, and whether the baseline build shows it too. Continue with the cases it does not block. The coder investigates read-only, posts the root cause and a proposed fix to the human, and waits: code changes only after the human's go. The fix goes to the reviewer as a working-tree diff; after `NO REMAINING MUST-FIX ISSUES`, QA rebuilds and reruns the failed case and every case the fix touches. A failure the baseline shows too is recorded as pre-existing and is not fixed here unless the human says so.

QA creates and archives only its own roles, crews, chats and missions, never touches Jason's production app or its sessions, never operates pre-existing development sessions, and never opens a Runner database. Clean up as `full-smoke-test.md` says and stop the development daemon at the end. Keep the baseline worktree and the evidence; removing them is Jason's call.

QA writes the run record `docs/tests/runs/<run date>-macos.md` in the suite's format, and replaces the test record's Live QA section with the LD rows, a link to the run record, the backup path, and native Windows as unverified.

## Validation

If code changes, run each of these, log it to a file, and read the exit code from the command itself, never through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

Gate any import or helper that only `cfg(unix)` tests use with `#[cfg(unix)]`.

## Boundaries

The coder and reviewer never run the development app. No slot starts, stops, restarts or types into Jason's production Runner, its chats or missions; this mission itself runs there. No extra agents, crews or subagents beyond QA's own bounded test chats and missions in the development app.

## Authorization

After QA, and after review of any fix, Jason authorizes folding this brief, the run record, the test record and #878 fixes into the PR's launch-definition commit on top of current `origin/main` (`git commit --amend`, no co-author trailers); pushing with `git push --force-with-lease`; updating PR #881's body with the QA matrix and evidence, findings and fixes, and what waits for Jason; and watching CI with `gh pr checks 881 --watch` until macOS and Windows are green, folding each CI fix into the commit it corrects after the reviewer checks it. Do not merge, delete the branch or either worktree, or cut a nightly or release.

Jason separately authorized fixing the pre-existing `mission_stopped` feed card on 2026-10-13. Implement only after QA releases the candidate, review the working-tree diff, and have QA rebuild and verify `MIS-FEED-01` plus owned-mission stop/archive screenshots. Commit this feed fix separately after the #878 launch-definition commit, and record its pre-existing attribution in the test record and PR body.

Final Runner handoff to everyone: the PR URL and CI result, the QA matrix, findings and what was fixed, checks with exit codes, and what is left for Jason. Then all slots stand by.
