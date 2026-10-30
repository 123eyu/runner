# #1044 — Remote PIN setup and offline pairing

Jason requested a codex duo mission on 2026-10-30 (one coder, one reviewer). Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-1044-remote-pin-setup` on `fix/1044-remote-pin-setup`, based on `origin/main` at `6dfc734a`. The root checkout stays on `main`, and this worktree builds into its own `target/`.

## Problem and outcome

Issue [#1044](https://github.com/yicheng47/runner/issues/1044): a new host never asks for a PIN, and pairing with a paused or offline host looks like a wrong PIN. Implement [the spec](../../features/1044-remote-pairing-setup.md). It settles the issue's open questions, fixes the copy in English and zh-hans and lists the tests. The spec wins on detail; raise a conflict with the code to Jason through Runner instead of guessing.

In short:

- Host: open the existing Set PIN dialog right after a sign-in started in that window completes with no PIN. Show a warning-tinted setup row under the status header, in place of the PIN row, until a PIN exists. `runner remote login` prints a `runner remote pin` hint when no PIN is set.
- Other computer: `open_computer_pair` opens an **Offline** state with no PIN field for a computer listed offline. Classify submission failures (relay 4409, timeout, unreachable, other) so the typed PIN is kept and **Try again** resubmits it. Only `wrongPin` clears the field.

## Read first

- `crates/runner-app/src/surfaces/remote_client/pairing.rs`: `PairState`, `refusal`, `open_computer_pair`, `sync_computer_pair`, `submit_computer_pin`, `render_computer_pair`.
- `crates/runner-app/src/surfaces/remote_client/bridge.rs` (`choose_computer`), `mod.rs` (`status_label`, Settings computers list **Pair**), and `workspace/render.rs` (**Pair** / **Pair again**).
- `crates/runner-app/src/surfaces/settings/remote.rs`: `relay_card`, `remote_pairing` (the `pin_locked` row pattern), `remote_open_pin`, `remote_connect`, `warn_pin_lock`. Also `settings/remote/account.rs` and the status observer in `crates/runner-app/src/app_store.rs` (`apply_remote_update`).
- `crates/runner-daemon/src/remote/client/route.rs` (`close_error`, `pair`), `client/mod.rs` (`pair`) and `remote/transport.rs` (`Failure`, `client_refusal`): the source of the error strings. Read them; do not change them.
- `crates/runner-cli/src/remote.rs`: the login success path.
- `crates/runner-app/i18n/en.toml` and `zh-hans.toml`.
- `docs/guide/guides/remote-access.md` and `docs/guide/zh-CN/guides/remote-access.md`.

## Boundaries

No client protocol, `ClientError`, daemon message or `runner_server` change, and no listing field for paused or PIN state. Nothing from #1041: the paired-host banner and unavailable workspace stay as they are. No new UI components and no Pencil work: reuse the existing dialog, `SettingsRow`, button and tint patterns the spec names. No polling and no automatic PIN retries. No unrelated refactors. The CLI line means the PR is not `frontend-only`; do not apply that label.

## Validation and handoffs

Before testing, list the changed behaviours and tie each check to one, following AGENTS.md Test Scope. Run the spec's automated checks in the existing `settings::remote` and `remote_client` test modules and `runner-cli`. Then run `cargo nextest run --locked -p runner-app -p runner-cli --cargo-profile ci`, `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, `cargo fmt --all --check` and `git diff --check`. CLI help does not change.

The coder implements and hands the candidate SHA to the reviewer through Runner. The reviewer reviews the diff against the spec. Pay particular attention to: the prompt firing only on this window's sign-in transition and never from a stale trust snapshot; `sync_computer_pair` not dropping a kept PIN, while still resetting on an identity change; each string classification matching the daemon's real text; and **Try again** sending exactly one `remote_pair`. The coder folds review fixes into the commits they correct. Coders and reviewers do not run the app. The live checks in the spec need two computers and are left to Jason; list them in the PR body.

## Authorization

Jason authorized implementation, validation, commits on this branch, pushing it and opening one PR against `main` that closes #1044. Fold this brief and the spec into the implementation commit; split commits only where each one stands on its own. Fold later review or CI fixes into the commit they correct and push with `git push --force-with-lease`, never a plain force push. Rebase onto `origin/main` only when AGENTS.md requires it; never merge `main` in. No tool-specific coauthor trailers.

Stop at an open PR with macOS and Windows CI green. Report through Runner with the PR URL, final SHA, a short summary of the changes and the validation results. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. While waiting for a teammate, finish the turn and rely on the pushed inbox notification; do not start a watcher or a polling loop.
