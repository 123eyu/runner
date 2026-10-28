# #977 app layout and production-file splits

This is a pure module restructure on `refactor/977-app-layout`, based on `4bc32f5a` (the existing brief commit is above that base). Preserve every function body, UI string, style and test assertion. Changes to imports, visibility, module declarations and relative asset paths are mechanical consequences of the moves. No live app checks are needed for the intended pure moves; the reviewer will confirm that from the final diff.

## One module tree and a thin entry point

The library becomes the app's single module tree, retaining its public modules for integration tests. `main.rs` keeps the Windows subsystem attribute and calls the library's startup entry point. Use the crate's self alias to preserve existing `runner_app::` imports without rewriting unrelated files. Declare `test_support` and `theme_snapshot` only once.

Move `NativeRoot`, its associated state types and core implementation to `surfaces/root/`: `mod.rs` owns state and small shared helpers, `init.rs` owns construction, and `store.rs` owns store reactions. Re-export only the crate-visible names the existing surfaces need. Field and helper visibility changes are limited to reaching the same callers through the new module boundary. Existing inline root tests move intact with this module.

Move startup, menus and platform startup helpers to `startup.rs`. Move free window-open, focus, close and layout-checkpoint helpers to `surfaces/windowing/` alongside the existing ownership helpers: `mod.rs` keeps ownership coordination, `windows.rs` holds opening/focusing/closing, and `checkpoint.rs` holds persistence. Keep platform `cfg` attributes attached to their original items.

`platform_ui` owns the platform implementations, `material_macos`, and the three font modules. Select and re-export the font module there using ordinary module declarations; the library's existing `platform_fonts` path remains a compatibility re-export. Update the material call's module path. There are no cross-tree `#[path]` attributes and no second platform module tree.

## Existing responsibility boundaries

| Original file | Target modules and responsibilities |
|---|---|
| `surfaces/panes.rs` | `panes/mod.rs` for shared state and pure layout helpers; `tab.rs` for archived/active tabs and topbar; `header.rs` for header and side-panel rendering; `dialogs.rs` for confirmations and rename; `tree.rs` for pane tree and menus; `drawer.rs` for drawer strip/terminal; `pane.rs` for one pane; `panel.rs` for side-panel content; `start.rs` for empty/start surface. |
| `surfaces/settings_page.rs` | Move under `settings/page/`: `mod.rs` for state, routing and selection/save actions; `render.rs` for settings takeover/navigation; `shortcuts.rs` for shortcut rows/recording; `general.rs` for General and command installation; `appearance.rs` for appearance/terminal controls and palette helpers. Preserve both inline test modules intact under the page module; change external settings-page imports to the new path. |
| `surfaces/start_chat.rs` | `start_chat/mod.rs` for modal state and small shared selection helpers; `terminal.rs` for terminal and drawer creation; `modal.rs` for modal opening/construction/selection; `launch.rs` for launch submission/request helpers; `render.rs` for modal/direct-fields rendering and render helpers; `controls.rs` for runtime/model/effort/speed controls. Keep the existing `add_role.rs` and `role_setup.rs` children and the inline tests. |
| `surfaces/chat.rs` | `chat/mod.rs` for lifecycle events, transitions and fork state; `actions.rs` for session controls/pin/rename; `attachment.rs` for attachment, focus, input and resume; `layout.rs` for move/split/resize; `close.rs` for pane/tab/drawer close and archive confirmation. |
| `surfaces/app_shell.rs` | `app_shell/mod.rs` for route and shell composition; `usage.rs` for usage helpers/popover; `sidebar.rs` for sidebar/brand/resize/preview; `actions.rs` for theme/settings/window and navigation actions; `notices.rs` for daemon banners/toasts/retry. Preserve the existing account-menu child. |
| `surfaces/settings/agents.rs` | `agents/mod.rs` for state, discovery and preference/override actions; `render.rs` for list/detail rendering; `presentation.rs` for status/presentation helpers. |
| `surfaces/remote_client/workspace.rs` | `workspace/mod.rs` for remote window state and visit/return; `sync.rs` for state synchronization/metadata/attachment; `render.rs` for connection/workspace/terminal rendering; `layout.rs` for sidebar nodes, layout mutations and accessors. |

Target each resulting production file below approximately 1,500 lines. Keep an existing inline test module together even when total file size exceeds that target; the target excludes inline tests. Files outside this mission's enumerated app files are not split: any pre-existing file over the target will be listed in the PR rather than expanding this pure-move scope.

## Commit sequence and validation

After clean working-tree review, create one focused commit for the single-tree/root/window move (including this plan and the mission brief), then one commit per original large file in the table, preserving a buildable and tested history. Keep temporary per-step patches under `/tmp/977-app-layout-*` so the fully reviewed diff can be assembled into those commits without introducing unrelated changes.

Changed behaviors: none. Validate module resolution and platform gating with `runner-app` tests including integration tests, workspace Clippy and its updater pass, formatting, and `make verify`. Use the app tests at each move boundary, and the complete required gates on the final candidate; inspect intermediate commit trees before publishing. No new tests or assertion edits are intended. Record every test whose import path changes.

Measure physical production lines for every touched Rust file before and after, excluding complete inline test items and test files, using the same parser-based method on both snapshots. Report the app-wide count above 1,000 production lines and any remaining file above 1,500, and preserve the numbers in the PR.
