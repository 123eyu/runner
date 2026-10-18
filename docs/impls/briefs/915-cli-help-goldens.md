# 915 — CLI help goldens drop commands behind a positional cap

Fix [#915](https://github.com/yicheng47/runner/issues/915). Jason requested this `codex duo` mission on 2026-10-18. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-915-cli-help-goldens`, on the existing branch `fix/915-cli-help-goldens`, created from `origin/main` at `140296bc`. The root checkout stays on main. Do not create another branch or worktree, and do not share a Cargo target directory. Other worktrees under `.worktrees/` belong to other work; treat them as another machine's checkout.

## The bug

`help_cases()` in `crates/runner-cli/tests/golden.rs` (about line 666) builds the `--help` golden cases from the first 50 entries of `cases()` with a positional `.take(50)`. Adding a case earlier in `cases()` silently pushes a later command's help out of the goldens, and the test still passes. #906's empty-start cases did exactly that: `runner ask --help` and `runner call --help` have no golden in `crates/runner-cli/tests/goldens/cli.txt`, so changes to their help text go unchecked.

## Read first

- `AGENTS.md`, including Worktrees, Test Scope and Crew Missions.
- Issue #915.
- `crates/runner-cli/tests/golden.rs`: `cases()`, `help_cases()`, and how `UPDATE_RUNNER_GOLDENS` regenerates the golden file (about lines 788 and 824).
- `crates/runner-cli/tests/goldens/cli.txt`.

## Deliverables

1. **Every command keeps its help golden.** Remove the positional cap so every command and subcommand reached by `cases()` gets its `--help` case, however many other cases the file has. Prefer deriving from every case without the cap; if that produces duplicates or nonsense, list the help cases explicitly. Keep the existing explicit entries and the one-word handling for `status`, `signal`, `ask` and `call`.
2. **A guard against regressions.** Add a check that fails when a visible top-level command or subcommand of the clap `Cli` has no `--help` case, so a new command cannot go unchecked again. Reuse the existing clap introspection pattern in `crates/runner-cli/src/help.rs` tests (`Cli::command()`) if it fits; do not build a new harness.
3. **Regenerate the goldens** with `UPDATE_RUNNER_GOLDENS=1` and confirm the only changes in `cli.txt` are added `--help` cases, including `runner ask --help` and `runner call --help`; no existing case may change. Goldens also run on Windows CI, so check whether the file or its normalization differs by platform before assuming one regeneration covers both.

Keep the change scoped to the test and its golden file. No CLI behavior or help-text change, so `docs/guide/reference/cli.md` and `make guide-cli` stay untouched.

## Boundaries

Crews never run the dev app or drive Jason's Runner. Do not start extra agents, crews or subagents.

## Review, verification and authorization

The coder owns implementation and checks. The reviewer waits for an explicit Runner handoff, then reviews the full branch diff against #915 with must-fix findings first and file:line pointers. Focus on no help case being lost or changed, the guard actually failing when a command lacks a case, and no sleep or platform assumption in the test. Iterate until the reviewer posts `NO REMAINING MUST-FIX ISSUES`.

Run `cargo test --locked -p runner-cli --profile ci`, workspace Clippy with warnings denied (`cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`), `cargo fmt --all --check` and `git diff --check`. Log each to a file and record the exact commands and exit codes, read from the command itself, never through `tail` or `grep`. Gate any import or helper used only by `cfg(unix)` tests with `#[cfg(unix)]`.

After a clean review, Jason authorizes rebasing all work on this branch into clean history on top of current `origin/main`, this brief folded into the commit, usually one commit with a subject that names the fix (for example `test(cli): keep a help golden for every command`), with no co-author trailers; pushing `fix/915-cli-help-goldens`; and opening a PR against main whose body says `Fixes #915` and lists the checks with exit codes and the reviewer's verdict, with no agent session links. If main has moved, rebase; never merge main into the branch. Review or CI fixes after the push are folded into the commit they correct and pushed with `git push --force-with-lease`. Drive CI green on macOS and Windows with `gh pr checks <n> --watch`. Do not merge, delete the branch or worktree, or cut a nightly or release. Final Runner handoff: PR URL, what changed, checks and exit codes, CI result and the reviewer's verdict. Then stand by.
