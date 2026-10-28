# #1023 — Speed up CI

Jason approved a `codex duo` mission (one coder, one reviewer) on 2026-10-28. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/feat-1023-ci-speed` on `feat/1023-ci-speed`, based on `origin/main` at `cd8e1b86`. The root checkout stays on `main`; build only in this worktree's own `target/`.

## Problem

[#1023](https://github.com/yicheng47/runner/issues/1023) is the spec: cut CI wall time without weakening coverage or adding flakes. [#846](https://github.com/yicheng47/runner/pull/846) introduced nextest; its record is `docs/tests/archive/845-ci-nextest.md`. Nothing user-visible changes, so no `docs/features/` spec or guide page is needed.

Baseline from run [40681403222](https://github.com/yicheng47/runner/actions/runs/40681403222) (PR #1070, warm cache):

| Job | Total | Test step | Test execution |
|---|---:|---:|---:|
| `Rust / Windows` | 17m44s | 778s (~3 min compile) | 592s, 2,863 tests |
| `Rust / macOS` | 11m04s | 427s | 252s, 2,982 tests |
| `Rust / Linux` (x86_64, aarch64) | ~7.5 min | 288s / 261s | |

Windows is the critical path. Its other serial steps add about 3.6 min: Clippy 82s, "Build Runner for resource verification" 55s, "Check generated CLI reference" 49s, "Verify Windows installer behavior" 29s.

Median test time is the same on both platforms (~0.04s), so per-process overhead is not the problem. The Windows tail is:

- `runner-app` GPUI tests: p90 2.45s on Windows vs 0.27s on macOS; package total 935s vs 109s.
- CLI goldens: p50 4.5s vs 1.4s, inside the `real-daemon` nextest group capped at two threads; `built_cli_goldens_help` alone takes 98s on Windows (7.9s on macOS).
- `runner-daemon`: 570s vs 229s in total.

## Read first

`.github/workflows/ci.yaml`, `.config/nextest.toml`, `docs/tests/archive/845-ci-nextest.md`, the [Test Scope](../../../AGENTS.md#test-scope) and CI commands in `AGENTS.md`, and the slowest tests named above (`crates/runner-cli/tests/golden.rs`, `crates/runner-terminal` snapshot tests, `runner-app` window/render tests such as `surfaces::start_chat` and `surfaces::app_shell`).

## Deliverables, in order of expected payoff

1. **Windows GPUI test cost.** Find out why window/render tests are about 9× slower on Windows than macOS (for example per-test text-system, font or window setup) and remove the repeated cost while every assertion still runs on Windows. Measure before choosing a fix.
2. **CLI goldens.** Split `built_cli_goldens_help` and other long goldens into separately schedulable tests where their assertions allow. Measure CPU, process and port use before changing the `real-daemon` two-thread cap; keep the cap where correctness or stability needs it.
3. **Workflow.** Move independent steps off the critical path (parallel jobs, or reusing a build instead of rebuilding) where this materially shortens wall time. Keep every existing gate (formatting, Clippy including the macOS updater pass, CLI reference, lockfile, Windows installer and resource checks, Linux packaging checks) and the required check names `Rust / macOS` and `Rust / Windows` exactly.
4. **Redundant tests.** Audit for duplicated coverage last. Remove or merge a test only when its distinct assertions and failure modes stay covered, and name the retained tests in the test record.

Non-goals: retries, skipped or ignored tests, weaker assertions, timing tests that stop measuring real timing requirements, dropped platforms, and product behavior changes. If a production code change becomes necessary, keep it behavior-preserving and explain it.

## Validation

Record everything in `docs/tests/1023-ci-speed.md`: before/after per-step and test-execution timings for each platform, at least two warm-cache runs after the change (and one cold build if caching changes), and the test-count change with its explanation. For every test made newly concurrent or split, run it repeatedly on macOS and Windows CI and locally, and check for shared-file, port, process and fixture interference; passing must not depend on retries.

Before running local checks, name the changed behavior each one covers. Run `make verify` locally (three `runner-daemon` `launch_definition::claude_*` tests fail when `CLAUDE_CONFIG_DIR` is set in the shell; unset it). All four CI jobs must end green. No live app checks are needed unless the reviewer finds a change to what the running app does.

Known pre-existing flake: `runner-daemon companion::tests::exit_archive_and_status_recovery_drain_a_held_frame_before_closing` fails about 0.2% of runs under load (it expects a separate `status` frame, but a later `overview` push supersedes the queued one). It is out of scope. If it fails, record the occurrence and re-run that job; do not re-run for any other failure.

## Authorization and handoff

Jason authorized implementation, validation, commits on this branch, pushing it, and one PR against `main`. CI runs only on pull requests to `main`, so as an exception to the crew's PR-after-review rule the coder may open that PR as a **draft** early, solely to measure CI; a newer push cancels the run in progress, so let measurement runs finish. Request review from `reviewer` on the working-tree diff as usual, and mark the PR ready only after review is clean, CI is green on all four jobs, and the operator has completed their own review of the full branch diff (added by the operator through Runner on 2026-10-28). Draft-PR pushes for CI measurement remain authorized before that operator review. Label the PR `no-behavior-change` if the shipped app behaves the same, and say why in the description.

Shape the history per `AGENTS.md`: fold this brief into an implementation commit, keep commits that each build and pass, fold review and CI fixes into the commit they correct, and use `git push --force-with-lease`, never plain force. No tool-specific coauthor trailers. Rebase only when `AGENTS.md` requires it; never merge `main` in.

Only the two crew slots are authorized: no extra missions, internal subagents or delegated agents. Stop at a ready PR with green CI and report through Runner: PR URL, final SHA, the before/after table and any open risks. Do not merge, release, cut a nightly, archive the mission, delete branches or remove worktrees. When waiting for a teammate, finish the turn and rely on the pushed inbox notification.
