# PR 981 — Fix the Windows relaunch pipe race

Jason requested a single Opus mission on 2026-10-21 to fix the blocking review finding in https://github.com/yicheng47/runner/pull/981, which addresses issue #980. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-980-windows-single-instance` on the existing PR branch `fix/980-windows-single-instance`. The starting PR head is `88dfcd907b69f7d883eb3508cdde734bd73e2a4f`, based on main at `59c1e1621b3152d8c097536d48d6e0718701e2f1`. The root checkout stays on main. Use this worktree's own Cargo target directory. No additional agents, subagents, crews, worktrees, or PRs.

## Read first

Read `AGENTS.md`, the current PR body and diff, `crates/runner-app/src/single_instance.rs`, its startup and reopen callers in `crates/runner-app/src/main.rs`, and `.github/workflows/ci.yaml`. The Windows-only module currently uses `#[cfg(all(windows, not(test)))]`; ensure new regression tests actually compile and run on Windows rather than being excluded by that gate.

## Blocking finding

In `single_instance.rs:96-99`, every `ConnectNamedPipe` failure except `ERROR_PIPE_CONNECTED` terminates the listener. The secondary process writes one byte and immediately closes its handle at lines 138-139. If it connects, writes, and closes before the listener calls `ConnectNamedPipe` (during startup or between accept cycles), Windows can report `ERROR_NO_DATA`. The listener exits without disconnecting or closing its raw handle. Later launches cannot hand off; after the 10-second fallback they can start duplicate app processes and take the daemon's main-window registration again.

This finding was established by code review and API semantics, not reproduced on a Windows host. Verify it rather than treating the proposed remedy as fixed design. Sources: https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-connectnamedpipe and Microsoft's go-winio `listenerRoutine`, which explicitly retries immediate client disconnects: https://github.com/microsoft/go-winio/blob/main/pipe.go.

## Deliverables

1. Fix early client disconnect handling and listener lifetime so valid activation requests are not silently lost and subsequent requests remain serviceable. Choose the smallest correct protocol or recovery change. Handle pipe resources consistently when a listener exits or cannot start; avoid introducing an unbounded wait into the launching process. Preserve the primary/secondary distinction, separate dev/installed identities, foreground-rights handoff, reopening/restoring main, and the intended takeover when the prior process exits.
2. Add deterministic Windows regression coverage for a client that connects, writes, and closes before the server's accept call, then another activation through the same listener. Also cover an early disconnect without a valid request if the chosen protocol needs it. Use uniquely named test pipes and bounded coordination; test meaningful behavior and avoid timing-only sleeps. Demonstrate that the relevant regression fails against the original implementation where feasible. Ensure the test is included in Windows CI.
3. Inspect the complete PR diff for regressions related to this change. Update the relevant guide page (`docs/guide/guides/chats-and-windows.md`) if needed to document Windows relaunch behavior, as required by AGENTS.md. No visual design change is requested; no Pencil work is needed. Keep the fix scoped.
4. Update the existing PR's body with current validation evidence, the final handoff behavior, and remaining Windows live-check gaps. Its old body says Windows CI is still pending, but both platform jobs passed for the original head; those results do not validate the new changes.

## Validation and boundaries

Before tests, state the changed behaviors and selected checks with their reasons. Run the relevant pipe regression tests on Windows through CI (or an already available authorized Windows test environment), runner-app tests, workspace Clippy with warnings denied, updater-feature Clippy on macOS, `cargo fmt --all --check`, and `git diff --check`. Prefer the CI profile and locked dependencies. Record commands and exit codes, distinguishing macOS checks from the Windows-only behavior they cannot exercise. Gate platform-specific test imports correctly. No full smoke or unrelated regression expansion is requested.

This is an implementation mission. Do not run the desktop app, operate existing sessions, change global agent configuration, or start additional agents. The Windows UI checklist remains explicitly unverified unless separate authorized live evidence exists: relaunch while open, restore minimized main, reopen main while a secondary window survives, and installer relaunch after an update. A CI build is not a UI smoke pass.

## Authorization and handoff

The authorized mission includes implementation, tests, commits, pushing the existing branch, updating PR #981, and driving its required macOS and Windows CI jobs green. Fold this brief and the review fix into the existing fix commit, using amend or fixup/autosquash, and push with `git push --force-with-lease`; do not use plain force. Keep clean history on main and never merge main into the branch. Rebase only if GitHub reports a conflict or overlapping main changes require it under AGENTS.md. Do not add tool-specific coauthor trailers.

Stop at the updated open PR with green CI and report through Runner: PR URL, final head SHA, what changed, regression evidence, checks and CI results, and any remaining live-test gaps. Do not merge, release, cut a nightly, archive the mission, or remove branches/worktrees. The requesting reviewer will assess the fix under Jason's earlier conditional merge authorization.
