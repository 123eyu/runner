# Mission briefs

A brief gives a mission its working directory, problem, deliverables, validation and authorization. Write it on the mission's branch before launch, then fold it into an implementation commit when the work lands. Keep three examples of different sizes; other completed briefs live in git history. Active missions keep their briefs until their work is finished.

## Examples by size

These are real briefs, with their original dates, paths and task-specific permissions. Copy their structure and replace the details. They do not authorize a new mission or override the current [`AGENTS.md`](../../../AGENTS.md).

| Size | Brief | What to take from it |
|---|---|---|
| Small | [Mission inbox guidance](mission-inbox-guidance.md) | One coder, one focused change, named authoritative files, bounded checks and an open-PR handoff. Merged in [#985](https://github.com/yicheng47/runner/pull/985). |
| Medium | [#735 — Start a chat modal](735-start-chat-modal.md) | A designed UI feature: spec and PNG references, concrete states and request mappings, coder/reviewer handoffs and native-check gaps. Shipped in 0.12.5. |
| Large | [#823 — Linux support](823-linux.md) | A cross-platform change: ordered deliverables across runtime, UI, packaging and CI, review risks, and a separate live-test record. Active in [#982](https://github.com/yicheng47/runner/pull/982). |

Size describes the work, not a required agent count or brief length. Small fixes normally stay inline; launch a mission only when Jason requests or approves one. Large programs may need several independently reviewable missions rather than one long goal. Keep each launch goal under 8 KB and link to its spec or implementation plan for detail. The historical medium and large files exceed that launch limit: use them as supporting references and summarize the goal instead of passing the whole file unchanged.

## Writing the next brief

1. **Starting point.** Name the issue or existing PR, branch, exact worktree and base revision. The root checkout stays on `main`; every worktree uses its own Cargo target directory.
2. **Problem and outcome.** Describe the trigger, current behavior and intended result. Name the spec and design in `yeelinks_design` when applicable, and say which source wins on detail.
3. **Read first.** Point to the few source files and docs needed to begin. Prefer symbols over line numbers that will drift.
4. **Deliverables and boundaries.** State observable acceptance criteria, affected guide pages, non-goals and any approved probes or fixtures. Avoid repeating the repo's rulebook.
5. **Validation and handoffs.** Tie each selected check to changed behavior under [Test Scope](../../../AGENTS.md#test-scope). State who implements, reviews and runs live checks, and what evidence the final handoff carries.
6. **Authorization.** Record the actual approved agent count and Git actions. An authorized crew normally ends at an open PR with the required platform CI green; merging, releases and local cleanup need their own authorization. Fold review and CI fixes into the commits they correct.

Use the current [Crew Missions](../../../AGENTS.md#crew-missions) workflow. In particular, the old blanket “crews never run the dev app” in the medium and large examples is historical: coders and reviewers do not run it, while the person assigned selected live checks may use the isolated development environment under [Test Scope](../../../AGENTS.md#test-scope). Use current crate names and verification commands. Rebase only when the repo's current rules require it.

Inside a mission, participants finish their turn when waiting for a teammate and read the inbox when Runner notifies them. An outside operator starting a different mission follows the [external watch workflow](../../guide/guides/drive-runner-from-agents.md#start-and-follow-a-crew).

## Retention

At a docs sweep, prune completed briefs except these three examples. Keep a brief for active work even when it is not a sample. Before pruning, preserve unique current decisions in architecture docs, outstanding scope in its issue, and validation evidence in `docs/tests/`; fix references to use the owning document or a fixed git revision. Do not remove test evidence or implementation logs with a brief.

The 2026-10-22 sweep reduced ten briefs to these three. #823 remains both the large example and an active brief; the requirements in its open PR may have evolved beyond the original launch goal.
