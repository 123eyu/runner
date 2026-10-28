# #977 — Remote client ownership

## Owner and callback boundary

Use a plain `RemoteClient` held by `NativeRoot`. It owns `ClientUi`, the optional remote workspace, and the client-state subscription. It also holds the existing AppStore entity and window focus/identity needed by its presentation helpers. A second GPUI entity would force nested entity updates while the root is rendering shared panes and handling actions; keeping the existing `Context<NativeRoot>` preserves synchronous notification, focus and completion ordering without that reentrancy problem.

Remote methods become methods of this owner, with `Context<NativeRoot>` only as the GPUI callback/window bridge. Background callbacks still upgrade the existing weak root, then call its remote owner; all epoch, target, identity, request and attachment guards stay in place. Root adapters perform the few changes to shared modals, local workspace activation and shared page rendering. The owner never takes a whole mutable root to reach its own state.

## Moves

Move computer picker, pairing and computer-table coordination; overview and transport synchronization; terminal attachment, sizing, control, rendering and palette lookup; remote navigation and selection persistence; remote catalog/library refresh; and remote shared-action completion into `RemoteClient`. The existing transport, feed and client-state types stay in their modules and retain their protocol behavior. Keep remote workspace state private to the owner module where possible, exposing borrowed workspace access only for shared surfaces that still own layout/forms. Replace mode checks with owner queries and move the remote-only work behind owner methods rather than passing the root through a renamed namespace.

Root keeps thin adapters for target transitions (shared modal clearing and local activation), combined local/remote action dispatch, and rendering shared role/crew/tab pages before the remote owner adds its chrome. Host enrollment/settings in `settings/remote.rs` stays: it manages this computer's daemon state, independent of the viewer target, and moving it would not help this ownership boundary.

## Mission workspace

Introduce a presentation owner for local versus remote mission data. Move remote snapshot/member/status/settings/feed/drawer queries into that owner and keep transport/feed subscriptions, saved remote presentations and remote action bookkeeping there. Workspace callbacks continue to own shared mission layout, terminal focus and event projection; remote-specific selectors and updates go through the presentation owner. Preserve existing generations and target-switch resets. Avoid a second operation interface: `FrontPageClient` remains the shared request owner from #1057.

## Validation and measurements

Before checks, record the moved behavior and selected existing coverage in `docs/tests/977-remote-root.md`. Existing remote-client, front-page, mission-remote and host-settings tests retain their fixtures/assertions; list moved-symbol edits. Add boundary tests only if extraction introduces behavior. Run `cargo test -p runner-app`, `make clippy`, `make fmt`, and `make verify`, using a command-local descriptor limit if the documented pooled SQLite test pressure requires it. No app launch or live checks by this duo; reviewer names the local/remote checks for Jason.

Use #1057's production scanner (strip comments/strings, test files and `cfg(test)` items; count all platforms) against `e0b69671` and the final tree for `NativeRoot` methods, `remote_window` identifiers and mission-workspace `self.remote` identifiers. Also report remaining direct borrowed workspace accesses so a field rename cannot masquerade as the ownership result. Request working-tree review before commits, then fold the mission brief into the first implementation commit and open a PR saying "Part of #977" after clean review. Stop at an open PR with macOS/Windows/Linux CI green.

## Implemented boundary and measurements

`RemoteClient` owns the subscription, viewer UI, target workspace, overview/attachment reconciliation, computer UI, library refresh and action completion. Its root-context callbacks preserve the existing weak-root lifetime and generation checks. `remote_client/bridge.rs` contains the root adapters for shared dialogs, local workspace release/activation and shared page rendering. `MissionPresentation` owns local/remote snapshot selection, remote member/status/settings/drawer queries, feed reconciliation and selected-member state; shared mission layout, event projection, dialogs and action UI remain on `MissionWorkspace`.

| Production metric | `e0b69671` | Candidate |
| --- | ---: | ---: |
| `NativeRoot` methods | 577 | 544 |
| Files with `NativeRoot` methods | 50 | 45 |
| Root methods named for remote/computer/pairing | 68 | 37 |
| `remote_window` references | 263 | 0 |
| Files with `remote_window` references | 33 | 0 |
| `self.remote` references in mission-workspace sources | 75 | 14 |

The remaining fourteen `self.remote` references call private selectors on the presentation owner, rather than accessing a field on `MissionWorkspace`. The full viewer `workspace()`/`workspace_mut()` accessors are now `cfg(test)` inspection APIs: production consumers have zero full-workspace borrows and cannot reach the mutable window. Shared surfaces use narrow layout, bounds, role/crew surface and computer queries. Role/crew filtering and paging, layout preflight and resize drafts, launch completion tokens, project deletion and sidebar propagation belong to the owner.

For a comparable mission coupling measure, count whitespace-tolerant `self.remote` field accesses inside `impl MissionWorkspace` on the base, and `self.presentation.{is_remote,remote,remote_mut}` calls inside those same impls on the candidate: **82 → 39**. All 39 remaining calls are local-only lifecycle/configuration guards or distinct terminal/overlay availability; there are zero broad `remote()`/`remote_mut()` consumer accesses. The owner now saves/restores remote presentations, resets target/action state, binds/releases feeds, reconciles drawer snapshots, handles pending actions and shell retries, and selects composer/member/status/settings/feed data. Shared layout/dialog/event/focus updates remain on the workspace.

The scan uses the #1057 rules: strip comments and string/character literals; exclude test files and `#[cfg(test)]` items; count methods in explicit `impl NativeRoot` and `impl crate::NativeRoot` blocks across all platforms, identifier `remote_window`, and `self.remote` in mission-workspace sources. The supplementary borrowed-state count matches `\.remote_client\s*\.(?:workspace|workspace_mut)\(` on that same production source; the mission consumer metric above uses whitespace-tolerant accesses and only `impl MissionWorkspace` bodies, including all platforms and excluding tests.
