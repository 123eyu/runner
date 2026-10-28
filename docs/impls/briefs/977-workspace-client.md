# #977 — One workspace client for local and remote targets (items 7 and 8)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. This is the third mission. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-workspace-client`, on branch `refactor/977-workspace-client`, based on `origin/main` at `bbc3a074`, which includes #1054's shared `AttachedChat` setup. The root checkout stays on `main`; this worktree uses its own `target/`.

A parallel mission (item 1, branch `refactor/977-runtime-adapter`) changes daemon runtime code and `runner-app/src/terminal_paste.rs`; stay out of those. Read items 7 and 8 in the [2026-10-26 re-audit](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449). No behavior change is intended. The point is that SSH hosts (#808, 0.21) and the iOS companion (#912, 0.20) add a target without adding a third branch at every call site.

## The problem, measured on `bbc3a074`

- `NativeRoot` (`crates/runner-app/src/main.rs`) has about 560 methods across 48 files. Remote work added most of the recent growth as `impl NativeRoot` blocks in `surfaces/remote_client/{workspace,model,feed,transport,pairing,picker,mod}.rs`. Its remote state is `client_ui`, `remote_window: Option<RemoteWindow>` and `_remote_subscription`.
- About 280 production checks of `remote_window` are spread over 33 files. The largest are `chat.rs` 40, `remote_client/workspace.rs` 31, `panes.rs` 29, `app_shell.rs` 24 and `start_chat.rs` 23; `start_chat.rs` alone has about ten `if self.remote_window.is_some()` forks.
- `MissionWorkspace` (`surfaces/mission_workspace/`) branches on `self.remote: Option<remote::RemoteMission>` about 90 times across 14 files.
- `FrontPageClient` (`surfaces/front_page/client.rs`) already has the right shape: `start_chat`, `start_shell`, `stop`, `resume`, `restart`, `fork`, `close`, `rename`, `pin`, `move_item`, `archive`, `start_mission`, `feed`, `post`, `answer` and more. But `action()` returns "This operation requires a remote target." for `FrontPageTarget::Local`, so local callers bypass it and the surfaces fork instead.

## Deliverables

1. **Plan first.** Write `docs/impls/977-workspace-client.md` (short: the target shape, the ordered steps, what stays) and send it to `reviewer` before moving code. It must answer:
   - Which type is the one client interface? Likely `FrontPageClient`, extended so every operation it exposes works for `Local` through the local `DaemonClient`, unless a trait is clearly simpler.
   - Where does remote state live once it's out of `NativeRoot`? For example, a `RemoteWindow`-owning entity or struct whose methods move with it.
   - How does `MissionWorkspace` get one client instead of `self.remote` branches?
   - Which `remote_window` checks are about genuinely different presentation, and stay? Examples: the computer picker, pairing, host-palette drawing, remote sidebar chrome.

   The plan lands in this PR, and I relay it to Jason.
2. **Local implementations.** Implement the client operations for the local target. Each delegates to the same daemon call that the local branch it replaces makes today, so local behavior is identical, including error text and toasts.
3. **Call sites.** Route the operation call sites in `start_chat.rs`, `start_mission.rs`, `chat.rs`, `panes.rs`, `app_shell.rs`, `front_page/`, `sidebar/`, `crews/`, `roles/` and `mission_workspace/` through the client. Remove the duplicated local/remote branches. Feature gating for older hosts (`supports`) stays where it is.
4. **Move remote state and methods out of `NativeRoot`.** Leave a thin delegating method only where GPUI ownership requires it. `NativeRoot`'s method count and the `remote_window` check count should both drop substantially. Report the before and after numbers in the PR.

If the full scope is too large for one reviewable PR, the plan may split it into two PRs: (a) the client and its call sites, (b) the move out of `NativeRoot`. In that case this mission opens (a), and (b) waits for Jason.

## Non-goals

- New remote features, companion protocol changes, SSH or the iOS work.
- The file splits of items 4 and 5, which are the next mission.
- Changing what any surface shows.
- Touching the daemon or the item 1 files.

## Read first

- `AGENTS.md` (Engineering Conventions, Test Scope, Crew Missions).
- `docs/tech/remote-control.md` and `docs/arch/arch.md` §3.6 (surface hierarchy).
- `surfaces/front_page/client.rs`, `surfaces/remote_client/workspace.rs`, `surfaces/mission_workspace/remote.rs`, and the `NativeRoot` struct in `main.rs`.

## Validation and handoff

Before running checks, list the changed behaviors and the check that covers each. Run the `runner-app` tests (they include the remote client, front page and mission workspace remote tests), then `make clippy` and `make fmt`. `make verify` must pass before the PR. Where a local path newly goes through the client, add a test proving it reaches the same daemon call. Existing remote tests stay green unedited, except where they only name a moved symbol, and the PR lists those.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` reviews it and names live checks under Test Scope. This change reroutes local chat, mission and sidebar operations, so expect selected local checks: start, stop, resume, fork, rename and archive a chat, and start a mission. Remote checks need two computers. Live checks are left to Jason and listed in the PR as not run.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: the plan doc, implementation, tests, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977", and driving macOS, Windows and Linux CI green. Fold this brief into the first commit, fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the before/after counts, the named live checks and anything deliberately left behind, through Runner.
