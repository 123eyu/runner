# Fix the stale Codex draft hold (#940)

Jason requested this mission on 2026-10-22 using codex duo, with permission for the operator to add one QA role if needed. Issue: https://github.com/yicheng47/runner/issues/940. Work only on the existing branch `fix/940-codex-draft-hold` in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-940-codex-draft-hold`, created from `origin/main` at `be8feff6efc3aac4d069005223b23c59f98ab134`. Keep the root checkout on main and use this worktree's own target directory.

## Problem and outcome

A Codex mission member can finish a natively entered prompt, show an empty composer, and still retain a Drafting delivery hold. Crew messages then wait indefinitely until Runner's Submit / clear control resets tracking. The issue records native failures on the feature candidate and its pre-feature baseline; an October 20 comment records Jason encountering it in daily use. Fix normal submission and empty-composer recognition so inbox delivery resumes naturally, while genuine unsent drafts continue to block delivery.

The likely mechanism is a hypothesis to verify: native Return during Probing does not mark the probe submitted; subsequent echo can be mistaken for a live draft although it is now transcript history. Marking the probe submitted alone may still fail because the Submitted path treats unchanged text on that history row as a rejected submission and returns to Drafting. Do not assume the diagnostic proves the live event ordering.

## Read first

- `AGENTS.md`, the full issue body and its October 20 investigation comment.
- `crates/runner-terminal/src/input_state.rs`: `observe_input`, `submit`, `track_delivery_submit`, `observe_output`, `observe_composer`, echo matching and row relocation; existing input-state tests and `crates/runner-terminal/tests/input_state_replay.rs`.
- `crates/runner-daemon/src/session/state/mod.rs` (`draft_hold`) and `session/manager/mod.rs` (`delivery_readiness`), plus the callers of changed tracker behavior.
- `crates/runner-app/src/surfaces/mission_workspace/input.rs`: existing Submit / clear recovery.
- `docs/tests/archive/826-targeted-messaging.md` for native evidence; old temporary artifacts may no longer exist. `docs/tests/regression/missions.md`: MIS-DRAFT-01, MIS-DRAFT-03 and MIS-MSG-04.

## Deliverables and boundaries

1. Establish a deterministic failing regression on the starting code for native submit before echo and a new empty composer below submitted history. Use the issue's diagnostic as a starting point, adapting it to current helpers. Capture what it proves and what still requires native evidence.
2. Implement the smallest justified correction. Distinguish the live composer from transcript echoes through observed terminal state. Cover the actual confirmed mechanism rather than merely clearing every hold on Return or adding a timer that can erase an unsent draft.
3. Verify ordinary typing/paste and immediate Return, delayed/coalesced echo, submitted text still visible in history, and a subsequent genuine draft. Include wrapping/scroll movement and multiline or collapsed-paste protection where affected by the implementation. Preserve genuine failed submission, composition protection and generated notification behavior. Reuse the existing fixture/replay infrastructure; do not rewrite expected results just to pass.
4. Keep this a focused tracker/delivery correctness fix. No terminal redesign, runtime refactor, watch feature, unrelated cleanup, or fix for #918/#935. Existing UI conventions remain; no visual design work is planned. Update relevant guide text if documented user-visible behavior or recovery advice changes, as required by AGENTS.md.

## Validation and handoffs

Coder implements and runs targeted terminal input-state/replay tests and affected daemon readiness tests, followed by the applicable crate tests and required formatting/Clippy/CI gates. Before testing, state the changed behaviors and why the selected checks cover them. Use locked dependencies and the repository's CI profile where applicable. Do not run an unrelated full smoke suite.

Reviewer waits for the coder's explicit Runner handoff, then reviews the working-tree diff and test adequacy before implementation commits or PR creation. Reviewer does not implement fixes. Both coder and reviewer must finish their turn when waiting, consume `runner msg read` when notified, and never launch built-in subagents or poll for teammates.

Native verification is valuable because this bug depends on terminal echo ordering. Neither coder nor reviewer launches the app. When a reviewed candidate is ready and live evidence is needed, request the authorized QA addition through Runner with the worktree, HEAD, diff SHA256, build identity and a bounded plan. The outside operator may add one configured `qa` role; the crew must not bypass Runner to launch agents. QA freezes the candidate, runs the issue's native paste/immediate-Return scenario and relevant MIS-DRAFT-01 / MIS-DRAFT-03 / MIS-MSG-04 variants, recording an empty composer, no false hold, one clean notice/read/ACK, and intact real drafts. Use development data and owned scratch fixtures only under AGENTS.md's standing test authorization; preserve existing sessions and global configuration. Coordinate the single Cargo/build owner. Record runtime/platform gaps honestly; Windows CI is not native Windows evidence. A blocked live check is not a pass. Record native results in the repository's dated run format and hand failures back to coder for correction and affected re-review/retest.

## Authorization and completion

This authorized crew follows AGENTS.md's default open-PR workflow: the operator commits this brief before launch; after clean working-tree review, coder commits scoped implementation/tests/docs, pushes this branch and opens one PR against main, then drives required macOS and Windows CI green. Fold the initial brief into the implementation commit and fold later fixes into the commits they correct. Use force-with-lease only for authorized history folding, never plain force; rebase only when repository gates require it. Do not add tool-specific coauthor trailers.

Report through Runner with PR URL, final SHA, root cause, changes, checks and any unresolved native validation gaps. Stop at the open PR; no merge, release/nightly, local branch/worktree deletion or mission archival is authorized. Start with exactly coder and reviewer; one optional QA role is authorized, no additional agents. The external Codex operator has no documented idle-event watch facility, so do not assume its chat will wake; use Runner handoffs and structured human questions when operator action is required.
