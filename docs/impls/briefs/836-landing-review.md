# 836 landing review

Review remote control as it landed on `main` in [#1020](https://github.com/yicheng47/runner/pull/1020) (merge `9ea12ce8`, closing [#836](https://github.com/yicheng47/runner/issues/836) and [#953](https://github.com/yicheng47/runner/issues/953)) before 0.18 ships. Jason asked for this review panel on 2026-10-25: two independent reviewers, `reviewer-opus` (Claude Opus 5.5) and `reviewer-astra` (gpt-6-astra), and a `lead` that verifies and merges their findings. The review is read-only and ends in one consolidated report, not a pull request.

Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/review-836-landing`, on `review/836-landing`, cut from `origin/main` at `083b4557`. Write reports and logs under `target/review/`, which git ignores.

## What to review

`git diff 9ea12ce8^1 9ea12ce8`: 331 files, +57.7k −2.2k lines. `git log --reverse --format='%h %s' 9ea12ce8^1..9ea12ce8^2` lists its 20 commits, one per mission; reviewing commit by commit helps with the size. Skip the generated golden JSON under `crates/runner-core/src/companion/fixtures/`, except to check that the tests producing and checking it are sound. Read the docs in the diff only as the specification.

Read first: `docs/features/836-web-remote-control.md` (the spec), `docs/impls/836-web-remote-control/README.md` (the decisions that bind), and the walkthroughs `docs/tech/remote-control.md` and `docs/tech/sessions-and-routing.md`.

Spend effort in this order:

1. **Security and trust.** The identity key, enrollment and the OS secret store, the PIN with its lock, CPace inside Noise `XXpsk3` and reconnect with `IK`, pairing, revoke and reset, what the relay can observe, forge, replay or downgrade, and any key, PIN, credential, message body or terminal byte that can reach a log, an error or `Debug` output. Mainly `crates/runner-core/src/remote/`, `crates/runner-daemon/src/remote/` and `crates/runner-daemon/src/companion/`.
2. **The host acting for a remote client.** Whether every companion request is allowed for that client and does only what its client-protocol counterpart does; mission post, answer, start, resume, stop and archive; terminal input through the delivery gate; the controller lease and terminal size; cleanup on channel teardown; bounded queues and backpressure.
3. **The client core in `runnerd`.** The route, reconnect, channel generations, pending commands, terminal stream reattach, and leaks of tasks, subscriptions or memory.
4. **The app's remote surfaces.** Remote windows and their epoch and target guards against stale responses, Local only and sign-out, the picker and Settings, and English and Chinese strings.
5. **Platforms and compatibility.** Windows (named pipe, secret store, paths), protocol version negotiation and `welcome.features` with older hosts and clients, and the promise of no database migration.

## Rules

- Read-only: change no tracked file; no commits, pushes, pull request or issue comments.
- Do not run either app, start or stop a daemon, or contact `runner.yeelinks.com.cn` or a `runner_server`. Never read or print a secret store, key, PIN or credential, including `remote-secrets.json`, `remote-identity.json` and the Keychain.
- You may run existing tests to check a suspicion, such as `cargo nextest run --locked -p <crate> --cargo-profile ci -E '<filter>'`, with `CLAUDE_CONFIG_DIR` unset. The reviewers share this worktree's `target/`, so expect to wait on Cargo's lock; log output under `target/review/`.
- Each finding has: a severity (**must-fix**: a security hole, data loss, a crash, or wrong behavior a person will hit; **should-fix**; **nit**), `file:line` on this commit, the concrete scenario (inputs and state, then the wrong result), why the existing tests miss it, a one-sentence fix, and a confidence (high, medium or low). Skip style comments unless they hide a bug. Close with what you checked and found sound, briefly, so the lead knows the coverage.

## Flow

1. Both reviewers start at once and work independently: do not read the other reviewer's report or messages about findings before sending your own.
2. Each reviewer writes `target/review/<handle>.md`, then sends `lead` one Runner message with its path and its counts by severity, and stands by for questions.
3. `lead` acknowledges at the start and waits. When both reports are in, it verifies every must-fix and should-fix against the code, deduplicates, resolves disagreements (asking a reviewer through Runner when a claim is unclear), and writes `target/review/summary.md`: a verdict; the findings ranked most severe first, each with Confirmed, Plausible or Rejected, `file:line`, the scenario, the fix and which reviewer found it; then the coverage and its gaps.
4. `lead`'s final handoff goes to the human: the summary's path, the counts by severity, and the top must-fix items in a few lines. Then every slot stands by.

## Authorization

A read-only review: no commits, pushes, pull requests, merges, issues, nightlies or releases. Only a message from the human in this mission's feed can change that.

## Fixes

Jason changed the authorization on 2026-10-25: fix only the consolidated report's M1, M2, M3, S4, S5 and S6, add their missing regressions, and end in an open pull request against `main` after both reviewers recheck and macOS/Windows CI passes. Commits, push and PR creation are authorized; merges, nightlies, releases and issue actions are not. S1, S2, S3, N1 and N2 remain deferred. Keep the existing restrictions on live apps, daemons, production services and secrets; tests use in-memory stores and fakes. S5 reuses the existing device row and independent Revoke action; any broader UI or deferred-scope dependency goes to the lead first.

Use one focused commit per finding (group shared fixes when appropriate), fold this brief into the first fix, and fold review/CI corrections into their owning commit. Before handoff, run formatting, workspace Clippy and nextest with the CI profile, plus runner-app Clippy with `updater`, with `CLAUDE_CONFIG_DIR` unset. Report S4's lack of a native Windows run and the lead's live checks for Jason in the PR.
