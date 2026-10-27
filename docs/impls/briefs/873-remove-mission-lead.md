# #873 — Remove the mission lead

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-873-remove-mission-lead` on `feat/873-remove-mission-lead`, based on `origin/main` at `2c4015ba18a7320b5a5d65fb782b2241913e7f12`. The root checkout stays on `main`; this worktree uses its own `target/`.

## Problem and outcome

A crew needs exactly one lead, and a mission started from a crew gives it a separate prompt and the only copy of the goal. Since #562 the no-lead paths exist for **No crew** missions, `ask_lead` is gone and untargeted messages already reach every member, so the lead now only gates the goal and restart notices. Jason decided on 2026-10-27 to remove it: every mission runs the way a **No crew** mission does today.

The spec, [`docs/features/873-remove-mission-lead.md`](../../features/873-remove-mission-lead.md), is authoritative: its Decisions, Scope and Verification sections list every change, the guide pages and the selected live checks. The UI change is removal only and has no new design; the spec's App section is the source.

## Read first

- `crates/runner-daemon/src/router/prompt.rs` (`member_prompt_channels`, both preambles, the lead prompt) and `crates/runner-daemon/src/runtimes/mod.rs` (`PromptChannels`, pi's lead path).
- `crates/runner-daemon/src/router/mod.rs` (`LaunchInputs`, `LeadRow`, `record_slot_restart`) and `crates/runner-daemon/src/session/manager/spawn.rs` (prompt kind on spawn and resume).
- `crates/runner-daemon/src/ops/mission.rs` (start), `ops/mission/roles.rs` (remove), `ops/slot.rs` and `repo/slot.rs`, `db/seed.rs`.
- `crates/runner-core/src/mission_roster.rs`, `protocol/`, `companion/`, `model.rs` (`ASK_LEAD_REMOVED_HINT`), `protocol/agent_skill.rs`.
- `crates/runner-cli/src/command.rs`, `output.rs`, `help.rs`.
- `crates/runner-app/src/surfaces/crews/`, `start_mission.rs`, `mission_workspace/` (rail, feed, composer, actions, remote, `mod.rs`), `i18n/en.toml` and `i18n/zh-hans.toml`.

## Deliverables

- Everything in the spec's Scope: one member prompt with the `mission show` and own-part lines, the goal for every member, restart notices to every other member, any crew with a slot can start, any member can be removed, no lead in the CLI or the app, `runner ask "<q>"` asking the person, ROLE in `mission show`.
- Stored lead flags still load and are ignored; the companion protocol keeps `lead: false`. No schema migration.
- The guide pages, product and architecture docs, `AGENTS.md`'s vocabulary, both READMEs and the regression rows the spec names, with `make guide-cli` regenerated.
- In-process tests for every Verification bullet; existing lead tests are deleted or rewritten, not skipped.

Out of scope: dropping the `lead` column or roster key, #916's verbs, #748, and any change to `ask_human`, the inbox or delivery.

## Validation and handoffs

`@coder` implements, runs the relevant crate tests, `make guide-cli`, `cargo fmt --all --check`, `git diff --check` and `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, then hands the diff to `@reviewer`. `@reviewer` reviews the code only. This brief settles the live checks: they are required, so a clean review says so and names any check the diff adds beyond the spec's table.

After the clean review, the coder commits, pushes, opens the PR and drives the macOS, Windows and Linux CI green. The person then adds `@qa` to this mission with `runner mission add-role`. `@qa` runs the spec's live macOS selection on the PR head in the development app under Test Scope's standing authorization (inexpensive model, low effort, tiny prompts, fresh folders under `/tmp`, only its own chats and missions), writes `docs/tests/873-remove-mission-lead.md` in the suite's evidence format, and reports a verdict naming the checks that ran and the SHA. Failures go to `@coder`; fixes are folded, re-reviewed, and `@qa` reruns the affected checks. `@coder` commits the test record. Windows and Linux live checks are recorded as gaps.

## Authorization

Jason approved on 2026-10-27 a `codex duo` mission (`@coder`, `@reviewer`) and, after the clean review, one `qa` role added by the person. No other agents, missions or internal subagents. Authorized Git actions: commits on this branch, pushing it, and one PR against `main`. The history is clean on top of `main`, shaped as the crew judges best; this brief and the spec are folded into an implementation commit, and review or CI fixes are folded into the commits they correct and pushed with `git push --force-with-lease`. No tool-specific co-author trailers.

Stop at an open PR with green CI and a qa verdict. Report the PR URL, final SHA, a short summary and the validation results. Do not merge, cut a nightly or release, archive the mission, or remove the branch or worktree. When waiting for a teammate or the person, finish your turn and rely on the inbox notification.
