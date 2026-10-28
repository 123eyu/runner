# #977 — Remote client state out of `NativeRoot` (items 7–8, part b)

## Starting point

Jason asked on 2026-10-28 for the rest of [#977](https://github.com/yicheng47/runner/issues/977) to be fixed by `codex duo` missions, one pull request each. This is part (b) of items 7 and 8. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/refactor-977-remote-root`, on branch `refactor/977-remote-root`, based on `origin/main` at `e0b69671`. That base includes [#1057](https://github.com/yicheng47/runner/pull/1057), which put every workspace operation behind `FrontPageClient` for both targets. The root checkout stays on `main`; this worktree uses its own `target/`. Name scratch files `/tmp/977-remote-root-*`, since another mission may run at the same time.

Read first:

- the [2026-10-26 re-audit](https://github.com/yicheng47/runner/issues/977#issuecomment-6408777449), items 7 and 8;
- #1057's plan `docs/impls/977-workspace-client.md` and validation record `docs/tests/977-workspace-client.md`. Step 4 of the plan describes this PR, and the record lists the remaining presentation and ownership forks.

No behavior change is intended. Jason will run the two-computer remote checks after this PR, covering #1057 and this one together.

## The problem, measured on #1057's branch

- `NativeRoot` (`crates/runner-app/src/main.rs`) has about 579 methods. About 70 of them are named for remote, computer or pairing, spread over these files:
  - `remote_client/workspace.rs` (19)
  - `settings/remote.rs` (18)
  - `front_page/actions.rs` (9)
  - `remote_client/{picker,pairing,mod}.rs` (12)
  - `start_chat.rs`, `start_mission.rs`, `front_page/library.rs` and others (a few each)

  More remote coordination sits in methods without those words in their names.
- `NativeRoot` holds the viewer's remote state directly: `client_ui: ClientUi`, `remote_window: Option<RemoteWindow>` (a struct of about 50 fields) and `_remote_subscription`. About 263 production references to `remote_window` remain, in about 33 files.
- `MissionWorkspace` still branches on `self.remote` about 75 times.

## Deliverables

1. **Plan first.** Add a short plan to `docs/impls/977-remote-root.md` and send it to `reviewer` before moving code. It covers:
   - the owning type, for example a `RemoteClient` GPUI entity, or a plain struct `NativeRoot` holds, if GPUI ownership makes an entity worse;
   - how its callbacks reach `NativeRoot` and window state;
   - which methods move with it;
   - which `remote_window` checks become calls on the new owner;
   - what stays, and why.
2. **Move the state and its coordination.** Move the viewer's remote state and its coordination methods out of `NativeRoot` into that owner:
   - transport, overview and feed sync;
   - picker and pairing;
   - the remote workspace's tabs, panes and terminals;
   - the remote library, and remote completions of shared actions.

   `NativeRoot` keeps only thin delegation where window or context ownership requires it. The host-side settings pane (`settings/remote.rs`, which enrolls this computer as a host) may move to its own owner too, if the plan shows that's simpler. Otherwise leave it.
3. **Mission workspace.** Make the same move for `MissionWorkspace`'s remote state where its `self.remote` branches choose presentation owners.
4. **Report numbers.** Report the before and after counts in the PR: `NativeRoot` methods, `remote_window` references and `MissionWorkspace` `self.remote` references, using #1057's scan rules. The aim is a substantial drop in all three.

## Non-goals

- New remote features, companion protocol changes, SSH (#808) or iOS (#912) work.
- Daemon changes.
- The `main.rs` / `NativeRoot` module move and the large-file splits of items 4 and 5, which are the next mission. Don't move `NativeRoot` itself out of `main.rs` here.
- Changing what any surface shows.

## Validation and handoff

Before running checks, list the changed behaviors and the check that covers each. Run the `runner-app` tests. The remote client, front page, mission workspace remote and settings-remote tests must stay green, edited only where they name a moved symbol, and the PR lists those edits. Then run `make clippy` and `make fmt`. `make verify` must pass before the PR. Add tests only where an ownership boundary gains behavior: for example, a stale remote completion is dropped after a computer switch, or the owner is dropped on disconnect.

`coder` implements and hands the working-tree diff to `reviewer` before committing. `reviewer` names live checks under Test Scope. Jason runs the remote ones on two computers after this PR, so list them in the PR as not run. Any local check the reviewer names is also left to Jason.

## Authorization

The two-slot `codex duo` crew is authorized; no extra missions, slots or internal subagents. Authorized: the plan doc, implementation, tests, focused commits on this branch, pushing it to `origin`, opening a PR against `main` that says "Part of #977", and driving macOS, Windows and Linux CI green. Fold this brief into the first commit, fold review and CI fixes into the commits they correct, and push rewritten history with `--force-with-lease`. No tool-specific co-author trailers.

Not authorized: merging, releases or nightlies, archiving this mission, deleting branches or removing worktrees, launching the app, or changing global agent configuration. Stop at an open PR with CI green, and report the PR URL, the final SHA, the review verdict, the before/after counts, the named live checks and anything deliberately left behind, through Runner.
