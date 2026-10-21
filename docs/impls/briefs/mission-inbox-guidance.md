# Mission participants use pushed inbox notifications

Jason requested a separate mission with one coder and an open PR on 2026-10-21. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-mission-inbox-guidance` on `fix/mission-inbox-guidance`, based on `origin/main` at `59c1e1621b3152d8c097536d48d6e0718701e2f1`. The root checkout stays on main. This is independent of PR #981 and its Windows relaunch worktree.

## Problem

A reviewer inside a Runner mission unnecessarily started `runner mission feed --follow` and repeatedly waited on it while waiting for another member. Runner already pushes inbox notifications to mission participants. Existing coordination text explains how to read an inbox notification but does not explicitly tell every participant to finish the turn when waiting. The generated skill and CLI help also emphasize external start-and-watch instructions without a sufficiently explicit boundary. Jason asked to prevent this recurrence.

## Intended guidance

Use concise wording equivalent to: "Inside a mission, Runner pushes inbox notifications. When waiting for a teammate and no independent work remains, finish your turn. Do not start a feed watcher, poll, or sleep-loop to wait for messages. Run `runner msg read` when notified. External monitoring instructions do not apply to receiving messages in your own mission."

This applies to leads, workers and members of a mission with no lead. Iterating with another member means a conversation across turns, not staying active in a waiting loop. Preserve unfiltered inbox consumption and one-shot history reads after resume. Do not prohibit bounded diagnostic inspection, substantive independent work, or watching CI for an authorized PR. External operators who launch a different mission still need the existing documented monitoring or honest host-limitation path; do not claim that an external chat automatically receives mission updates. Do not implement the separate #748 watch-delivery feature.

## Scope

Read AGENTS.md and update the existing authoritative sources together:

- `crates/runner-daemon/src/router/prompt.rs`: worker and leadless-team preambles plus the lead's composed coordination text.
- `crates/runner-core/src/protocol/agent_skill.rs`: both generated skill variants, with the participant rule prominent and external watch rules explicitly scoped.
- `crates/runner-cli/src/help.rs`: agent guide and examples, preserving the outside-operator watch contract without telling participants to watch their own inbox.
- `docs/guide/guides/drive-runner-from-agents.md` and generated `docs/guide/reference/cli.md`; regenerate the CLI reference with `make guide-cli`.
- Affected existing prompt/help/skill assertions and golden snapshots, including runtime spawn expectations when required. Update them intentionally through their existing workflow and inspect the changes.

Keep this a focused instruction-text change. No delivery/runtime refactor, UI change, Pencil work, new framework, global configuration change, installed skill rewrite, or unrelated cleanup. Update generated source, not the installed skill file. Do not launch the desktop app or real-agent smoke fixtures. Exactly one coder is authorized; no reviewers, QA slots, extra missions, internal subagents or delegated agents.

## Validation

Before running checks, name the changed behavior and why each selected check covers it. Run the existing relevant prompt composition, skill rendering, CLI help and affected golden tests with locked dependencies and the CI profile, using this worktree's own target directory. Check that all lead/worker/leadless prompt paths and both runner/runner-dev skill variants carry the rule, and that external watch guidance remains valid. Run the guide-generation check, `cargo fmt --all --check`, `git diff --check`, and applicable workspace Clippy with warnings denied. Drive the required macOS and Windows CI checks green. No full smoke or unrelated regression expansion.

## Authorization and handoff

Jason authorized implementation, validation, commits, pushing this branch, and opening one PR against main. Review your own full diff inline; this single-coder mission does not require another agent's review before opening the PR. Fold this brief into the implementation commit, leaving one focused commit unless the change genuinely warrants more. Fold later fixes into the commit they correct and use `git push --force-with-lease` when rewriting already-pushed history; never use plain force. Do not add tool-specific coauthor trailers. Rebase only when required by AGENTS.md; never merge main into the branch.

Stop at an open PR with green CI. Report through Runner with the PR URL, final SHA, concise changes and validation results. Do not merge, release, cut a nightly, archive the mission, delete branches, or remove worktrees. When waiting for a teammate or human response, finish the turn and rely on the pushed inbox notification; do not start an inbox watcher or polling loop.
