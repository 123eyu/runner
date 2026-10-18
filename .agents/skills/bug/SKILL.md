---
name: bug
description: Report, list, or manage bug reports as GitHub issues
---

# Bug Reporting

Report and manage bugs for Runners via GitHub Issues with the `bug` label.

## Usage

`/bug <action> [args]`

### Actions

#### `new`
Report a new bug.

1. Ask the user to describe the bug: what happened, what was expected, and steps to reproduce (if known).
2. Investigate the codebase to confirm the area of code involved. Note relevant file paths.
3. **Pick a priority** (see Priority below). If the user didn't state one, propose one based on the bug's severity and the rubric, and confirm before filing. Don't file unlabeled.
4. **Pick a readiness label** (see Readiness below) when the investigation makes it clear, and propose it together with the priority. Leave it off when unsure.
5. Create a GitHub Issue with labels `bug`, the chosen `P0`/`P1`/`P2`/`P3` and the readiness label, if any:
   - Title: `bug: <short description>`
   - Body using this template:

     ```
     ## Description
     <what's wrong>

     ## Expected behavior
     <what should happen>

     ## Steps to reproduce
     <if known>

     ## Relevant code
     <file paths and brief pointers to the affected area>

     ## Environment
     - OS: <if relevant>
     - Version: <app version from `crates/runner-app/Cargo.toml`>
     ```
   - Command shape: `gh issue create --label bug --label P1 [--label agent-ready] --title "…" --body "…"`
6. Report the issue URL, the assigned priority and the readiness label.

#### `list`
Show all bugs, sorted by priority.

1. Fetch open bugs with priority + metadata as JSON so they can be sorted:
   `gh issue list --label bug --state open --limit 50 --json number,title,labels,createdAt,assignees`
2. Sort the rows by priority (`P0` first, then `P1`, `P2`, `P3`, then unlabeled-by-priority last). Within a priority bucket, sort by `createdAt` ascending (oldest first — they've been waiting longest).
3. Present a table with columns: **Priority**, **Readiness** (`agent-ready`, `needs-design` or —), **#**, **Title**, **Created**, **Assignee** (if any). Use the priority label as the leading column so the ranking is visually obvious. Issues without a P-label show `—` in the priority column and a callout asking the user to triage them.
4. If the user asks for closed bugs too, repeat with `--state all` and add a **State** column.

#### `close <issue-number>`
Mark a bug as fixed.

1. Close the GitHub Issue: `gh issue close <number>`
2. Confirm closure to the user.

#### `prioritize <issue-number> <P0|P1|P2|P3>`
Set or change the priority of an existing bug.

1. Remove any existing P-label on the issue, then add the new one:
   `gh issue edit <number> --remove-label P0 --remove-label P1 --remove-label P2 --remove-label P3 --add-label <priority>`
   (Removing all four is safe — `gh` ignores remove-label for labels not present.)
2. Confirm the new priority.

#### `ready <issue-number> <agent-ready|needs-design|none>`
Set or change an existing bug's readiness.

1. Remove both readiness labels, then add the new one unless it is `none`:
   `gh issue edit <number> --remove-label agent-ready --remove-label needs-design [--add-label <readiness>]`
2. Confirm the new readiness.

#### `view <issue-number>`
Show details of a specific bug report.

1. Fetch the issue: `gh issue view <number>`
2. Display the full issue body, current priority, comments count, and state.

## Labels

- `bug` — all bug issues use this label.
- `P0` / `P1` / `P2` / `P3` — priority, exactly one per issue.
- `agent-ready` / `needs-design` — readiness, at most one per issue (see Readiness below).

## Readiness

A readiness label says what has to happen before the fix can start. Many bugs get neither.

- **`agent-ready`** — small and fully specified: the cause and the fix are known, nothing is left to decide, and an agent can make and verify the fix in one pass. A fix that can only be verified on a Windows host, or that needs a reproduction first, is not agent-ready. Fix it inline, not as a crew mission.
- **`needs-design`** — the fix cannot start yet: it changes UI that has no signed-off Pencil frame, it has open questions the user must decide, or it spans several surfaces and needs a plan first.
- **Neither** — a known but substantial fix for a crew mission, or a bug whose cause still needs investigating.

Update the label as the issue moves: remove `needs-design` once the design is signed off or the questions are settled, and add `agent-ready` if what is left is small.

## Priority

Every bug gets exactly one priority label. Rubric:

- **P0** — Ship-blocker, data-loss risk, security issue, or recent regression breaking a load-bearing flow. Drop other work.
- **P1** — Real user friction in a common path; broken affordance the user sees daily. Fix this cycle.
- **P2** — Annoying but workaround exists; rare path; cosmetic issue in a prominent surface. Fix when convenient.
- **P3** — Edge case, theoretical, or "would be nice." No fixed timeline.

When in doubt between two levels, pick the lower-urgency one and say why; over-labeling P0/P1 dilutes the signal.

## Notes

- Do not commit or push unless the user explicitly asks.
- When creating issues, investigate the codebase first to include relevant file paths — this makes bugs actionable.
- Read the app version from `crates/runner-app/Cargo.toml` when populating the Environment section.
