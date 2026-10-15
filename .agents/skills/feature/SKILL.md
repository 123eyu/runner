---
name: feature
description: File, list, or manage feature issues, and write a feature's spec when its development starts
---

# Feature Management

A feature is a GitHub issue with the `feature` label until someone starts building it. Filing, prioritizing and scheduling a feature touch only GitHub: nothing is written to the repo and nothing is committed. Its spec in `docs/features/` is written when development starts, on the feature's branch, and reaches `main` in the same pull request as the code, together with any impl plan or mission brief.

## Usage

`/feature <action> [args]`

### Actions

#### `new <name>`
File a new feature as an issue.

1. Ask the user to describe the feature (motivation, scope, key decisions) unless they already have.
2. **Pick a priority** (see Priority below). If the user didn't state one, propose one and confirm before filing. Don't file unlabeled.
3. Set a milestone only when the user names one.
4. Create the issue with labels `feature` and the chosen `P0`/`P1`/`P2`/`P3`:
   - Title: `feat: <short description>`
   - Body: Motivation, Scope, and Open questions when there are any. The issue holds the design discussion until development starts.
   - Command shape: `gh issue create --label feature --label P1 --title "…" --body "…" [--milestone 0.16]`
5. Report the issue URL, priority and milestone. Do not create a spec or edit any doc.

#### `list`
Show all features, sorted by priority.

1. Fetch open features with priority + metadata as JSON so they can be sorted:
   `gh issue list --label feature --state open --limit 50 --json number,title,labels,milestone,createdAt,assignees`
2. List the spec files in `docs/features/` (excluding README); most features have none until development starts.
3. Sort the rows by priority (`P0` first, then `P1`, `P2`, `P3`, then unlabeled-by-priority last). Within a priority bucket, sort by issue number ascending.
4. Present a combined view: **Priority**, feature name, **#** (issue), **Milestone**, spec file (if one exists), **Created**. Issues without a P-label show `—` in priority and a callout asking the user to triage them.
5. If the user asks for closed/shipped features too, repeat with `--state all` and add a **State** column.

#### `spec <issue-number>`
Write the spec when development of a feature starts.

1. Work in the feature's branch worktree (`AGENTS.md`, Worktrees), never in the root checkout on `main`. If the branch does not exist yet, create it as `AGENTS.md` describes once the user has asked to start the work.
2. Create `docs/features/{issue}-{slug}.md`, pre-populated from the issue body and its discussion: a header linking the tracking issue, then Motivation, Scope, Implementation Phases, Verification. Leave priority and milestone out; they live on the issue.
3. Add the spec's line under Active in `docs/features/README.md`.
4. The spec is committed on the branch and lands with the code. A crew mission folds it into its commits like the brief.

#### `close <issue-number>`
Close a feature that shipped or was dropped.

1. Close the GitHub issue: `gh issue close <number>`, adding `--reason "not planned"` when it was dropped.
2. Leave its spec where it is. The release's docs sweep archives specs (`release` skill, `sweep`).

#### `prioritize <issue-number> <P0|P1|P2|P3>`
Set or change the priority of an existing feature.

1. Remove any existing P-label on the issue, then add the new one:
   `gh issue edit <number> --remove-label P0 --remove-label P1 --remove-label P2 --remove-label P3 --add-label <priority>`
   (Removing all four is safe — `gh` ignores remove-label for labels not present.)
2. Confirm the new priority.

## Labels

- `feature` — all feature issues use this label.
- `bug` — for bug reports (not managed by this skill).
- `P0` / `P1` / `P2` / `P3` — priority, exactly one per issue.

## Priority

Every feature gets exactly one priority label. Rubric:

- **P0** — Required for the next ship; nothing else moves until this lands. Rare for features.
- **P1** — Wanted this cycle; blocks an active user workflow or has a stakeholder commitment.
- **P2** — Real product win, but no urgency. Pick up when the P1 queue is clear.
- **P3** — Idea / nice-to-have / "if we ever revisit X." OK to sit indefinitely; closing as `wontfix` later is fine.

When in doubt between two levels, pick the lower-urgency one and say why; over-labeling P0/P1 dilutes the signal.

## Conventions

- A spec is named after its tracking issue, `{issue}-{slug}.md`, with a lowercase kebab-case slug.
- Specs, impl plans (`docs/impls/`) and mission briefs (`docs/impls/briefs/`) are written on the feature's branch and land with its code. Design files are the exception: `.pen` files are settled on `main` first (`AGENTS.md`, Engineering Conventions).
- At the release's docs sweep, retain specs in `docs/features/archive/` only when their decisions, compatibility constraints or implementation context still help. Obsolete or duplicate proposals can be pruned after preserving useful unique content and fixing references; issues and git history are enough for many dropped ideas. The implementation is the source of truth.

## Notes

- Do not commit or push unless the user explicitly asks.
- A spec always links its GitHub issue URL.
