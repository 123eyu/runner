# PR #1027 — fix the latest review findings

Jason requested this mission on 2026-10-26. Use the codex trio: coder implements, reviewer reviews, QA validates. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/fix-1027-linux-review-findings` on `fix/1027-linux-review-findings`, based on umbrella `feat/823-linux-support` at `889e1c3be61c703a6b1bbdbf9ec7db470f70c910`. Open the follow-up PR against that umbrella. Leave the root on main and preserve other worktrees.

## Evidence and scope

Read [PR #1027](https://github.com/yicheng47/runner/pull/1027) and the [latest review](https://github.com/yicheng47/runner/pull/1027#pullrequestreview-5649924156), posted 2026-10-26 at 09:34 UTC: two confirmed findings and a disputed documentation observation. The reviewed SHA is still the umbrella head; its four platform CI jobs passed before these fixes.

### 1. P2 — cancelled sign-in must not clear a newer attempt in another window

[Finding](https://github.com/yicheng47/runner/pull/1027#discussion_r4375759296). In `crates/runner-app/src/surfaces/settings/remote/account.rs`, `remote_cancel_start` waits for an abandoned start and then unconditionally calls `remote_logout()`. The cleanup barrier lives in each window's `RemotePaneState` in `remote.rs`. Window B therefore bypasses A's barrier. With A pending during service retry, cancel A and start B before A completes. The daemon replies to A with `Enrollment replaced.` before performing B's HTTP request (`crates/runner-daemon/src/remote/mod.rs`, around lines 676–692); A's deferred Logout can then clear B's successful enrollment.

Scope cancellation to its attempt, or serialize Start/cleanup through shared app state across windows if that fully covers the ordering. Preserve immediate button recovery, suppressed abandoned browser launch, ordinary sign-out and same-window retry. Add a deterministic two-window regression sharing the daemon/app store: B starts before A completes, with late success/failure. Keep cancel-before-dispatch and retry checks meaningful. Account for compatibility if a protocol change is necessary.

### 2. P3 — Linux updater must handle replacement of its running executable

[Finding](https://github.com/yicheng47/runner/pull/1027#discussion_r4375759311). In `crates/runner-app/src/updater/linux/install.rs`, updater paths use `std::env::current_exe()`. After `install.sh` or APT replaces the running binary, Linux may report `/proc/self/exe` with ` (deleted)` appended. Tar installation then succeeds but relaunch targets a nonexistent pathname; deb detection fails its exact `/usr/lib/runner/Runner` comparison, selects tar and rejects installation. Verified from code, not a live Linux GUI.

Resolve and validate a stable managed executable path consistently for detection, privileged invocation and relaunch, or explicitly require a restart before offering installation after replacement. Preserve ownership, symlink and package-layout checks, signature/channel/architecture verification, root installer restrictions and running-GUI preservation on failure. Do not blindly strip a suffix and trust the resulting file. Add Linux coverage in which the executable inode is actually replaced while its old process remains alive, alongside targeted tar/deb detection and relaunch assertions. Prefer disposable subprocess/install fixtures to touching an installed Runner.

### 3. Reconcile the disputed publication wording with current evidence

[Both reviewers' positions](https://github.com/yicheng47/runner/pull/1027#discussion_r4375759324). A workflow definition is not publication evidence, but setup verified the public `linux-nightly` prerelease was published 2026-10-25 and contains `889e1c3` tar.gz/deb packages and signatures for both architectures. Recheck [the release](https://github.com/yicheng47/runner/releases/tag/linux-nightly) and qualify the unpublished-download claims in the English/Chinese READMEs and installation/architecture text: temporary umbrella prerelease availability is distinct from stable/main-nightly availability and native acceptance. Preserve one-build Linux retention; clarify wording only if needed.

## Validation and documentation

Before testing, list changed behaviors and the selected checks with reasons. Run focused regressions, the `runner-app` tests, affected core/daemon tests if changed, workspace Clippy with warnings denied, the applicable macOS updater-feature Clippy pass, formatting and diff checks using the locked CI profile and this worktree's own target. Do not share CARGO_TARGET_DIR. Drive follow-up PR CI green on macOS, Windows, Linux x86_64 and aarch64. Preserve failures and disclose inaccessible checks; no unrelated full regression run.

Reviewer reads the complete fix diff and relevant callers/tests, then hands a clean candidate to QA. Coder never launches the app. QA runs only the relevant live cases: pending sign-in startup failure, Cancel and immediate retry in the same and a second window (`REMOTE-ACCOUNT-01` extension), and Linux managed tar/deb replacement while the old GUI remains open, update detection, failure/retry and correct relaunch (adapt the relevant update cases). Use `docs/tests/regression/README.md`, `docs/tests/823-linux-remote-sign-in.md`, `docs/tests/823-linux-updates.md` and `docs/tests/runs/2026-10-21-linux-updates.md` as references. The review's broader Linux acceptance matrix remains separate from this narrow fix.

The development app and disposable test roots/local test services are authorized under AGENTS.md. Preserve existing sessions, real account state and installed Runner. Use isolated development/local enrollment fixtures and disposable Linux installs; do not sign out a real account or install over a user's package to reproduce a bug. Do not launch additional agent sessions except bounded development test sessions already authorized by AGENTS.md. If native Linux or an isolated service is unavailable, record the exact required check as Blocked and give Jason reproduction steps; CI or a macOS UI check is not Linux live evidence. QA may finish with disclosed platform gaps, but must not claim the blocked cases passed.

Record the final candidate, selected matrix, results and evidence in `docs/tests/1027-linux-review-findings.md`. Update relevant `docs/guide/` pages for changed user-visible behavior, keeping prose one paragraph per line. No UI redesign is requested; preserve existing layouts and design conventions.

## Authorization and completion

This requested crew mission authorizes scoped implementation, tests, documentation, committing the brief before launch, and committing/pushing the fix branch and opening one follow-up PR against `feat/823-linux-support` after clean working-tree code review and QA with any limitations explicitly disclosed. Fold this brief into the implementation commit; split only independently meaningful, buildable changes. Fold subsequent review/CI fixes into the commits they correct and push with `git push --force-with-lease`, never plain force. No tool-specific co-author trailers.

Stop at the open follow-up PR with green CI and report its URL, final SHA, resolution of each finding, review/QA verdict and remaining human Linux checks through Runner. Do not merge either PR, cut a nightly/release, modify publication/cache/retention behavior, restore deliberately removed RPM support, address the known UTM Venus issue, archive the mission, delete branches or remove worktrees. Use Runner messages for handoffs and finish turns when waiting; do not use built-in subagents or start extra implementation/review agents.
