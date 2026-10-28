# #977 — App crate layout and large-file splits (items 4 and 5, app)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. This is the last app-side one. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-app-layout`, on branch `refactor/977-app-layout`, based on `origin/main` at `4bc32f5a`. That base includes #1054, #1056, #1057 and #1059, which shrank the same files this mission splits. The root checkout stays on `main`; this worktree uses its own `target/`.

A parallel mission splits `runner-cli/src/command.rs` and the daemon's `session/manager/{spawn,mod}.rs` on branch `refactor/977-cli-daemon-splits`, so leave those crates alone. Name scratch files `/tmp/977-app-layout-*`.

Read items 4 and 5 in the issue body and the [2026-10-26 re-audit](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449). Every change here is a pure move or module restructure: no behavior change, and no edits to logic, text, styles or tests beyond paths.

## Measured on `4bc32f5a` (production lines, excluding inline tests)

- `crates/runner-app/src/main.rs` has 2,094 production lines (2,471 in total). It holds the `NativeRoot` struct and its core `impl`, window-layout checkpointing, window opening and startup.
- `lib.rs` declares a second module tree. The binary depends on it through `runner_app::…` in about 75 files, and so do the integration tests in `crates/runner-app/tests/`.
- `lib.rs` reaches into `platform_ui/` with four `#[path]` attributes: `material_macos.rs` and `fonts_{macos,windows,linux}.rs`. `test_support` and `theme_snapshot` are declared in both trees and compile twice.
- Large app files, in production lines:

  | File | Lines |
  |---|---:|
  | `surfaces/panes.rs` | 4,244 |
  | `surfaces/settings_page.rs` | 3,498 |
  | `surfaces/start_chat.rs` | 3,388 |
  | `surfaces/chat.rs` | 2,836 |
  | `surfaces/app_shell.rs` | 2,284 |
  | `surfaces/settings/agents.rs` | 1,859 |
  | `surfaces/remote_client/workspace.rs` | 1,809 |

  `settings_page.rs` sits next to the `surfaces/settings/` directory it should have moved into.

## Deliverables

1. **Plan first.** Write a short plan in `docs/impls/977-app-layout.md` and send it to `reviewer` before moving code. It covers:
   - the target module layout for `NativeRoot` and window management, for example `surfaces/root/` or `root.rs`, plus the existing `surfaces/windowing.rs`;
   - how `platform_ui` gets one home in a single module tree, with no `#[path]` reaching across trees;
   - for each large file, the split along responsibilities it already has (render versus actions versus state, one module per pane, dialog or settings page), with a target that no app production file stays above about 1,500 lines unless the plan says why;
   - the commit sequence.
2. **Thin `main.rs`.** Make it an entry point: `NativeRoot`, its state and window management move into modules. Keep the binary/library split only where it does a job, such as integration tests or other crates that use the library. Remove the `#[path]` workarounds and the double compilation of `test_support`/`theme_snapshot`, unless the plan shows a reason one must stay.
3. **Splits.** Split the large files listed above. Finish moving `settings_page.rs` into `surfaces/settings/`.
4. **Report numbers.** Report before and after production line counts for every touched file, plus the count of files over 1,000 lines, in the PR.

Each commit should be a reviewable pure move, for example one per file, so `reviewer` can verify it with `git diff --color-moved` / `git log -p --color-moved`, and each one builds and passes its tests. This PR will be large, which is fine for moves. Don't mix in renames, re-ordering of logic, or "while I'm here" edits. Note them in the PR instead.

## Non-goals

- Behavior, UI, text or style changes.
- New abstractions.
- Changes to `runner-cli`, `runner-daemon`, `runner-core` or `runner-terminal`.
- Item 1 or 8 follow-ups.
- Splitting the test files themselves, unless a moved module's inline tests must move with it.

## Validation and handoff

Run the `runner-app` tests, including its integration tests, then `make clippy` (including the updater pass) and `make fmt`. `make verify` must pass before the PR. Linux and Windows compile through CI: platform-gated modules like `platform_ui`, `fonts_*` and `updates_*` must still build on all three, so check `cfg` attributes moved with their modules. No test edits beyond `use` paths. List any test you touched.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` checks that the moves are pure and names live checks under Test Scope. A pure-move refactor the tests cover normally needs none, so say why if it does. Live checks are left to Jason.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: the plan doc, the moves, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977", and driving macOS, Windows and Linux CI green. Fold this brief into the first commit, fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the before/after line counts and anything left above the target, through Runner.
