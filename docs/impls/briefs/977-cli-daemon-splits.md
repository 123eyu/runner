# #977 — CLI and daemon large-file splits (item 5, non-app)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-cli-daemon-splits`, on branch `refactor/977-cli-daemon-splits`, based on `origin/main` at `4bc32f5a`. That base includes #1056, which moved runtime behavior out of `spawn.rs` and into the adapters. The root checkout stays on `main`; this worktree uses its own `target/`.

A parallel mission restructures `runner-app` on branch `refactor/977-app-layout`, so leave `runner-app` alone. Name scratch files `/tmp/977-cli-daemon-splits-*`.

Read item 5 in the issue body and the [2026-10-26 re-audit](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449). Every change here is a pure move: no behavior change, and no edits to logic, help text, error text or tests beyond paths.

## Measured on `4bc32f5a` (production lines, excluding inline tests)

- **`crates/runner-cli/src/command.rs`:** 2,591 production lines, 4,458 in total. It holds:
  - the whole clap tree: `Cli`, `Command`, and the `Daemon`, `Project`, `Role`, `Crew`, `Mission`, `Chat`, `Session` and `Msg` command enums with their argument structs;
  - `run`/dispatch, local versus remote validation, daemon stop and wait, status and install status, Windows PATH reading, skill status, and feed filtering/following.
- **`crates/runner-daemon/src/session/manager/spawn.rs`:** 2,889 production lines.
- **`crates/runner-daemon/src/session/manager/mod.rs`:** 2,155 production lines.

## Deliverables

1. **Plan first.** Write a short plan in `docs/impls/977-cli-daemon-splits.md` and send it to `reviewer` before moving code. It covers:
   - the target modules for each file;
   - the commit sequence;
   - a target that no touched production file stays above about 1,200 lines unless the plan says why.
2. **`command.rs`.** Separate the clap definitions from the handlers, with one module per top-level command group (for example `command/{cli,daemon,project,role,crew,mission,chat,session,msg,feed,status}.rs`). Shared helpers such as `CliError` and output postprocessing get one home. `runner --help` and every subcommand's help must be byte-identical. Prove it by diffing the output of `make guide-cli`, or the help for every command, before and after; CI checks the generated CLI reference.
3. **`spawn.rs` and `manager/mod.rs`.** Split them along the responsibilities they already have, for example argv/env building, resume and fork, capture start, attach and output wiring, and session bookkeeping, as one module per concern under `session/manager/`.

   Keep in mind [#795](https://github.com/yicheng47/runner/issues/795) (0.21), which will separate the shell lifecycle from agent orchestration in these same files. Group shell-specific code so #795 can lift it out, but don't do #795 here.
4. **Report numbers.** Report before and after production line counts for every touched file in the PR.

Each commit should be a reviewable pure move, for example one per file, verifiable with `git diff --color-moved`, and each one builds and passes its tests. Don't mix in renames or logic edits. Note them in the PR instead.

## Non-goals

- Behavior, help-text or error-text changes.
- Changes to `runner-app`.
- New abstractions.
- The #795 shell split.
- Other daemon files.

## Validation and handoff

Run the `runner-cli` and `runner-daemon` tests, then `make clippy` and `make fmt`. `make verify` must pass before the PR, along with the help-output identity check above. No test edits beyond `use` paths. List any test you touched.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` checks that the moves are pure and names live checks under Test Scope. A pure-move refactor the tests cover normally needs none, so say why if it does. Live checks are left to Jason.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: the plan doc, the moves, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977", and driving macOS, Windows and Linux CI green. Fold this brief into the first commit, fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the before/after line counts and the help-identity evidence, through Runner.
