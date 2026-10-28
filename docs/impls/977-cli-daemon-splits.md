# #977 CLI and daemon file splits

Pure moves from `4bc32f5a` on `refactor/977-cli-daemon-splits`; no behavior, help, error-text or test-body changes. `runner-app` and daemon files outside `session/manager/` are out of scope.

## Module map and commit sequence

1. Split `runner-cli/src/command.rs`: keep dispatch and local/remote validation in the entry module; move the entire clap tree to `command/cli.rs`; move handlers to `daemon.rs`, `project.rs`, `role.rs`, `crew.rs`, `mission.rs`, `chat.rs`, `session.rs`, `msg.rs`, and `ask.rs`. Put feed filtering/following in `feed.rs`, status/install/Windows PATH/skill inspection in `status.rs`, and shared errors, caller transport, reference resolution and output/value helpers in `shared.rs`. Keep the existing inline test module in `command.rs` unchanged so extraction does not reformat test bodies; production counts exclude it. Fold the mission brief and this plan into this first commit.
2. Split `session/manager/spawn.rs` into existing concern boundaries: retain the unlisted entry path in `spawn.rs`; move argv/environment/spec construction to `spawn_spec.rs`, mission registration/completion to `mission_spawn.rs`, direct entry paths to `direct_spawn.rs`, fork materialization/spawn to `fork.rs`, resume/restart to `resume.rs`, and capture startup/Windows first-turn queuing to `capture.rs`. Group the existing terminal-shell identity and cwd/display helpers in `shell.rs` for #795; retain shell branches inside unchanged methods.
3. Split `session/manager/mod.rs`: keep manager and session state types, constructors and runtime watcher setup in `mod.rs`; move event types/forwarder emission to `events.rs`, delivery and session-map bookkeeping to `bookkeeping.rs`, handle/forwarder installation to `attach.rs`, status/model publication to `status.rs`, and rollout capture to `capture.rs`. Move role/runtime configuration helpers to `spawn_spec.rs`. Existing sibling tests retain their bodies, with import paths adjusted only if required.

Every touched production file targets at most 1,200 lines. The longest indivisible methods (resume around 675 lines, fork around 510) remain intact; no exception above the target is expected. New visibility and imports are limited to what the module moves require. No symbol renames or abstractions.

## Validation and review

Before moving code, build and run `make guide-cli` and save the generated references under `/tmp/977-cli-daemon-splits-*`. After the CLI split, regenerate and byte-compare both references. Each split stage builds and runs its crate tests before the next stage; retain temporary stage snapshots under `/tmp/977-cli-daemon-splits-*` so the eventual commits match those checked stages. No commits, push or PR until the reviewer approves the complete working-tree diff.

Changed behavior: none. The CLI suite covers clap parsing, validation, dispatch, feed following, status and reference resolution; the daemon suite covers spawn/resume/fork, attachment, delivery, status and lifecycle integration. Run both suites, `make clippy`, `make fmt`, and the required `make verify`; no app launch or live checks. Review uses `git diff --color-moved` plus a normalized moved-item comparison. Report production line counts (physical lines excluding inline test modules or standalone test files), every touched test/import, and help identity in the PR. Commit/push/open the PR only after clean review, then drive macOS, Windows and Linux CI green; do not merge or clean up this worktree.

## Validation evidence

The staged CLI suite and both daemon split-stage suites passed (1,504 daemon tests each). `make clippy` and `make fmt` passed. English and Chinese `make guide-cli` output is byte-identical to the baseline. No tests or fixtures were edited: the original CLI inline test block is byte-identical, and daemon tests retain their paths via test-only helper re-exports. Normalized comparisons preserve 135 CLI and 124 manager function/method definitions plus 49 type definitions.

Two full-workspace `make verify` attempts at the inherited 10,240 file-descriptor limit failed to open a temporary database in the unchanged app test `surfaces::start_chat::tests::confirm_binding_precedes_every_control_and_preserves_open_menu_values`. It passed alone. The full gate passed with `ulimit -n 65536 && RUST_TEST_THREADS=8 make verify`; the limit applied only to that validation process, and no tests were disabled.
