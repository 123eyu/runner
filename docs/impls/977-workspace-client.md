# #977 — One workspace client

## Target shape

`FrontPageClient` is the workspace operation interface. It keeps the current remote capability and generation guards, and implements local operations with the exact `DaemonClient` calls used by the existing local surfaces. Typed operation results retain the information needed by local completion handlers; presentation, attachment and refresh handlers remain on their owning GPUI entities. No new trait or daemon/protocol change is needed.

`MissionWorkspace` owns a `FrontPageClient` for both targets rather than obtaining its operation client from `RemoteMission`. Its remote state continues to hold the remote overview, terminal sizes, feed subscription and presentation cache. Operation dispatch uses the client; genuinely different snapshot, attachment, feature availability and completion behavior remains explicit.

## Ordered steps and PR boundary

1. Complete local client operations used by the shared call sites, including mission start, mission layout/pin, viewed marks and mission communication. Add request-level tests proving local dispatch retains the existing daemon methods and arguments; preserve the existing remote tests. Catalog, folder browsing and feed reads retain their existing remote-only client contract because native surfaces continue using AppStore data, native dialogs and their existing feed path.
2. Route chat, mission, tab, sidebar and library operation calls through the client in `start_chat`, `start_mission`, `chat`, `panes`, `app_shell`, `front_page`, `sidebar`, `crews`, `roles` and `mission_workspace`. Preserve local error wording, toasts, validation, defaults and completion handling. Keep older-host `supports` gates.
3. Review the working-tree diff, run the required app tests and workspace gates, and open PR (a), the client and its call sites. This is the current mission's deliverable. Moving GPUI ownership at the same time would mix operation parity with a second large change across asynchronous callbacks, so PR (b) waits for Jason as the brief permits.
4. In PR (b), move `client_ui`, `remote_window` and the remote subscription into a remote workspace entity. Move transport, overview/feed synchronization, pairing, picker and remote workspace methods with that entity. `NativeRoot` delegates only where window/context ownership requires it. This removes the remaining remote coordination method growth without changing rendering or terminal ownership.

## What stays

Computer selection and pairing, host terminal palettes, connection cards, remote sidebar chrome, host catalog availability, remote snapshot reconciliation, terminal transport/attachment and per-target layout/presentation state are genuinely target-specific. They remain explicit in PR (a); they belong to the remote entity in PR (b). Local filesystem dialogs and remote folder browsing retain their current UI. Shared command dispatch must not depend on those presentation branches.

`FrontPageClient::catalog`, `folders` and `feed` have only remote callers and retain the existing remote-target error for Local. This refactor does not add unused local readers or a parallel catalog/feed representation. `front_page/local.rs` holds only the mission-resume loop moved from its active native call site.

No behavior, guide, CLI help, daemon runtime, `terminal_paste.rs`, remote protocol, design or agent configuration changes are intended. Existing local completion paths retain their refresh/focus behavior. Live local checks (chat start/stop/resume/fork/rename/archive and mission start), and equivalent remote checks requiring two computers, are left to Jason; neither duo slot launches the app.

## Measurement and validation

Record reproducible before/after production counts of `NativeRoot` methods and `remote_window` references, excluding inline tests, with the PR and explain that remote ownership is deliberately deferred. Before checks, map each rerouted operation to its request parity tests and relevant existing app tests. Run `runner-app` tests, `make clippy`, `make fmt` and `make verify`; drive macOS, Windows and Linux CI green after clean working-tree review. Existing remote tests may change only to name a moved symbol, and any such changes are listed in the PR.

## PR (a) implementation boundary

The client now owns local command dispatch, including the existing per-member mission-resume loop and its partial-error result. Native completion handlers retain local node/mission rows where they need them. Chat launch, shell creation, chat/mission lifecycle, rename, layout persistence, archive batches and sidebar reorder operations use shared request paths. `WorkspaceAction` owns transient action preparation and completion guards; permanent remote window ownership remains deferred. Local role/crew editors remain local-only, as before; their configuration CRUD is not a new remote feature. The role-list chat shortcut retains its older `session_start_direct` request, while the New chat form retains `session_start_direct_with_speed`/`session_start_runtime_with_speed`.

PR (a) deliberately retains local attachment/transition completions and remote snapshot/generation completions. The remaining `remote_window` checks largely choose those presentation owners. It does not claim to finish the remote-ownership move or substantially shrink `NativeRoot`; that is PR (b).

| Production source metric | `bbc3a074` | PR (a) |
| --- | ---: | ---: |
| `NativeRoot` methods | 577 | 577 |
| Files with `NativeRoot` methods | 50 | 50 |
| `remote_window` references | 279 | 263 |
| Files with `remote_window` references | 33 | 33 |
| `self.remote` references in mission workspace | 88 | 75 |

The scan counts explicit `impl NativeRoot` and `impl crate::NativeRoot` methods and identifier references after removing comments, strings, test files and `#[cfg(test)]` items. It includes every platform source file rather than only the platform selected by the current build, so its method total differs from a macOS-only estimate. The before/after scan uses the same rules.


The command-path audit and remaining target forks are recorded separately in `docs/tests/977-workspace-client.md`.
