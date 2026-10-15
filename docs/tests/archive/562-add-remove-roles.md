# 562 — add and remove roles while a mission runs

Archived in the 0.16.2 docs sweep on 2026-10-15. Lasting live checks are maintained as MIS-ADD-01 and MIS-REM-01 in [Missions](../regression/missions.md); the draft cases below are the origin, and historical verdicts and platform gaps are preserved.

Branch: `feat/562-add-remove-roles`, worktree `/Users/jason/repos/yicheng47/runner/.worktrees/feat-562-add-remove-roles`. Phase 2 follows [the plan](../../impls/562-mission-spawn/plan.md), decisions 1–15. Native Windows remains unverified.

## Selected checks and reasons

| Check | Reason |
| --- | --- |
| Shared handle rule and unchanged crew tests | Both forms and the daemon must use the same validation while crew copy stays unchanged. |
| Daemon add/remove, cancellation and rollback tests | Roster publication, session lifecycle and routing change together; failures must leave a visible, stoppable team. |
| Bus membership live/replay tests | Broadcast membership and newcomer unread counts must survive remount; directed messages must be retained. |
| CLI parsing, protocol round trips and goldens | Person-only verbs, `--handle`, task input and human/JSON/quiet output are new. |
| App and unchanged Start chat tests | Shared role controls, join/leave feed dividers and mission refresh affect the app. |
| Workspace nextest, clippy with/without updater, fmt and diff checks | Required mission gates cover cross-crate and platform-conditioned integration. |
| ADD-1, ADD-2, REM-1, CLI-1 | Prove the new flows and restarts in the native development app with real bounded test agents. |
| MIS-CREW-01, MIS-LIFE-01, MIS-MSG-01, MIS-MSG-02, MIS-FEED-01, MIS-ROSTER-01 | Roster changes affect crew starts, lifecycle, addressing, broadcasts, feed and roster persistence. |

## Automated results

Logs are under the worktree's ignored `target/validation/`; exit codes below come from the commands themselves. This is the automated feature gate; native QA is recorded separately below.

| Command | Exit | Result / log |
| --- | --- | --- |
| `NEXTEST_TEST_THREADS=4 cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci` | 0 | 2,274 passed, 6 skipped; `workspace-nextest-r1.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings` | 0 | `workspace-clippy-r1.log` |
| `cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings` | 0 | `workspace-clippy-updater-r1.log` |
| `cargo fmt --all --check` | 0 | `fmt-check-r1.log` |
| `git diff --check` | 0 | `diff-check-r1.log` |

The first workspace run exited 100: the newly enforced roster filter exposed mission sessions without slots in the app counting fixture, and the new worker control required compact header spacing at narrow rail widths. The fixture now seeds valid slots, and the layout keeps the visible handle prefix and every fixed control at all tested widths. Two unchanged model/usage timing tests also failed under default concurrency; both passed in an isolated rerun, and the entire final workspace run passed at four test threads. A subsequent output-view change caused the two new human-output goldens to print session ids; the dedicated role-change view now prints handles in human and quiet modes while session-show quiet output remains a session id. Failure logs (`workspace-nextest.log`, `focused-rerun.log`, `workspace-nextest-reviewed.log`) remain alongside the passing logs. Initial compile and clippy failures are also retained.

Focused new tests cover handle suggestions/refusals, log-ordered membership in live and replay, directed messages before joining and after leaving, add/remove event payloads and order, pending-spawn refusals and cancellation, failed roster-write rollback, remount persistence, launch definitions and overrides across Resume/Restart, live removal through a fake runtime, deterministic concurrent Remove/Resume/Restart claims, Resume all excluding a removed worker, role deletion remaining refused, form validation, feed dividers, refresh signals and CLI flags/output. The crew page's tests and all existing Start chat tests pass unchanged. CLI golden additions are 145 lines.

## Commit validation after rebase

All three code commits built and passed the five required gates serially on `origin/main` `780be080` before push. Logs and direct exit-code files use `target/validation/commit-<revision>-<check>.log/.exit`; subsequent amendments change documentation only.

| Commit / logged revision | Nextest | Clippy / updater | Fmt / diff |
| --- | --- | --- | --- |
| Shared handle rule, `ebb65672dd58` | 2,254 passed, 6 skipped; exit 0 | Both exit 0 | Both exit 0 |
| Daemon/protocol/CLI, `f36a8a1d00ae` | 2,271 passed, 6 skipped; exit 0 | Both exit 0 | Both exit 0 |
| App, `bc92e2c70edf` | 2,274 passed, 6 skipped; exit 0 | Both exit 0 | Both exit 0 |

The rebase included #888's mission-tab bounds fix. Its integration review had no remaining must-fix issues. Jason waived the post-#888 live recheck; the one-line note in the existing daily run block preserves that decision and claims no additional native result. [PR #889](https://github.com/yicheng47/runner/pull/889) carries the current CI status. Native Windows remains unverified.

## Working-tree review

The first review requested one must-fix: Remove could publish a smaller roster while Resume or Restart had passed roster validation and was still spawning, leaving an off-rail agent. Following Jason's direction, removal now takes the existing per-session Resume/Restart claim before stop and holds it through publication and rollback. A held claim refuses removal with the worker's handle. Two deterministic fake-runtime gate tests cover both race directions, including a successful removal followed by refused Resume and Restart with no additional spawn. The non-blocking duplicate launch-resolution finding was also addressed: `role_joined` uses the registration result's recorded launch definition, with payload assertions against the session row.

The revised focused run passed all three `mission_roles` tests (`mission-roles-r1-3.log`, exit 0); all required workspace gates above then passed. The initial focused compile run exited 101, and the next run exited 100 because the test assumed a single runtime stop; the forwarder also performs an idempotent stop. The test now checks that all stops target the removed worker and that no later spawn occurs. Those logs remain as `mission-roles-r1.log` and `mission-roles-r1-2.log`. Revised working-tree review reported no remaining must-fix issues before the frozen-candidate QA handoff.

## Native macOS QA

Final native verdict on 2026-10-13: **Passed for ADD-1, ADD-2, REM-1, CLI-1, MIS-CREW-01, MIS-LIFE-01, MIS-MSG-01, MIS-MSG-02, MIS-FEED-01 and MIS-ROSTER-01 on macOS/Codex**. Native Windows remains unverified, outside this run. This is the selected feature pass, not a complete regression pass. The [run record](../runs/2026-10-13-macos.md#562-addremove-roles--resumed-run-and-final-verdict) contains the final matrix, candidate/environment, exact refusal exits, runtime conversations, screenshot paths, first failed fixture attempts and successful retries.

QA matched the handoff HEAD `58878d641b7707e08a0471b13d3dc2baa7dab293` and initial full diff SHA-256 `adf6f60cf224cbb2621e8ad9fff98e7795bb2cb5a7bef4162d6145aa49b154ce`, built once, and kept source unchanged. Final source-only diff SHA-256 is `0e6cb014772b340bbd77b6eace7dc5fda4f0acb87619b0074272a178e4413d38`; app and development sidecar hashes remained identical across restarts. Evidence root: `/private/tmp/runner-562-qa.moi8by_j/evidence`; ledger: `/private/tmp/runner-562-qa.moi8by_j/ledger.json`. The native six-surface comparison passed with the documented shared-control precedence.

Cleanup is complete: only the disposable mission was archived, with zero live sessions, zero pending asks and its Recents row absent. Test role/crew metadata and conversation history were retained. All QA app/daemon/agent/helper processes stopped, and the Codex config returned to its exact baseline bytes after removing only the owned trust entry. The historical preflight blocker and authorization sequence remain in the run record: four automatically restored pre-existing chats were explicitly stopped after Jason's earlier “confirmed”; his later daemon-only instruction was honored thereafter, and those histories were retained. No product source/test/expectation edit, commit, push or PR was made by QA.

## Regression cases for the release sweep

### MIS-ADD-01 — add a worker to a running mission

Start a bounded development mission with a lead and worker. Add a configured role from the rail with a task and an unused handle. Verify its card/tab, join divider, system introduction and directed task match the signed-off frames; prove from its saved conversation that it received its worker brief, introduction and task. Verify directed messages and broadcasts reach it. Restart the development daemon and resume: the same roster and overrides persist, and the unread projection excludes broadcasts from before the join, while `runner msg read` includes them. Repeat through `runner mission add-role`, including `--handle`, task input and mission-session/`--as` refusals. Preserve screenshots and transcript paths.

### MIS-REM-01 — remove a worker from a running mission

Remove a worker through the rail confirmation. Verify its session stops, card/tab disappear, an active removed tab focuses Feed, the leave divider and system notice match the frames, and its history remains. Verify `--to` refuses it and lists the current team; broadcasts and Resume all skip it. Its handle remains reserved. Restart the development daemon and verify all these properties persist. Repeat through `runner mission remove-role`, including lead, unknown/already-removed handle and mission-session/`--as` refusals. Archive only the test's own roles, crews and missions and stop every process it started.
