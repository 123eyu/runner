# 701 — Desktop notifications when an agent needs you

> Tracking issue: [#701](https://github.com/yicheng47/runner/issues/701)
> Priority: P2, milestone 0.16 (moved from 0.13 to 0.14 on 2026-10-07, renumbered 0.16 on 2026-10-10). Platforms: macOS and Windows.
> Decision, 2026-10-13: use the operating system's own notifications, Notification Center on macOS and toast notifications on Windows, posted through GPUI's system notification API. This replaces the 2026-09-22 decision to copy Zed's self-drawn popup window.
> Design: [`design/specs/701-desktop-notifications.pen`](../../design/specs/701-desktop-notifications.pen), waiting for review. The system draws the notification, so the design covers the Settings → Notifications pane (`K8FVAY`, with the new nav row in `cmp/SettingsNav`) and how each trigger fills the notification's title and body, drawn as macOS banners (`w9LiW` dark, `Ozj1d` light).

## Motivation

Runner shows that an agent needs you only inside its own windows: the sidebar's unread and attention dots, and the status control on each pane. Once Runner is behind an editor or a browser, an agent waiting on an approval, a question, or a finished turn waits until you happen to look back. With a crew and several chats running at once, that idle time is the main cost of running agents in parallel.

[#130](https://github.com/yicheng47/runner/issues/130) asked for macOS notifications in the Tauri era and was cut down to the in-app status signal before any notification shipped. Notifications were never built.

## Why native notifications

On macOS, apps normally alert through Notification Center; Slack, Mail, Messages and iTerm2 all do. A popup that an app draws itself outside its own windows is unusual. Zed's agent notification is the main example, and the first version of this spec copied it. Jason chose the native route on 2026-10-13 after reviewing the popup design.

Native notifications bring for free what a self-drawn popup would have to imitate or would lack:

- **Platform look**: the Mac banner on macOS and a toast on Windows, including the app's icon and name.
- **User control**: Focus and Do Not Disturb, Notification Center history, and the per-app banner style in System Settings.

They do not need Swift, an extension or new dependencies. `gpui-pre` 0.3.7, the GPUI Runner pins, already provides the API:

- **Calls**: `App::show_system_notification`, `App::dismiss_system_notification`, `App::on_system_notification_response` and `App::set_app_identity` (`gpui-pre/src/app.rs:1638`, the platform trait at `platform.rs:323`).
- **Notification**: a `SystemNotification` has a `tag`, a `title`, a `body` and optional action buttons. Posting with an existing tag replaces that notification, and a click returns its tag.
- **macOS**: it uses `UNUserNotificationCenter` (`gpui-pre-macos/src/system_notifications.rs`). It requests authorization on the first post, shows banners even while Runner is frontmost, and does nothing when the process is not in an app bundle instead of aborting.
- **Windows**: it uses WinRT toasts (`gpui-pre-windows/src/system_notifications.rs`). With `set_app_identity` it sets the process AppUserModelID and registers it under `HKCU\Software\Classes\AppUserModelId`, so an unpackaged build can show toasts too.

The costs, accepted:

- **macOS needs a bundle**: a plain `make run` build is not in an app bundle and posts nothing. QA and testing use a wrapped `.app`; see Verification.
- **Permission prompt**: the first notification on macOS asks the user for permission, so notifications start working only once the user allows them. After Don't Allow, nothing shows until the user allows Runner in System Settings. Windows shows toasts without asking.
- **Less control over presentation**: the system decides placement and stacking, and the notification has two lines of text and no custom styling.

## Behavior

### Triggers

A notification posts when a session *enters* one of these states. It does not post again on every status refresh, and a session that is already waiting when Runner starts does not post; the first load sets the baseline.

| Trigger | Signal | Status text |
| --- | --- | --- |
| Approval wait | a new `WaitReason::Approval` in `observation.interactions` | Approval needed |
| Answer wait | a new `WaitReason::Answer` | Answer needed |
| Input wait | a new `WaitReason::Unknown` | Needs you |
| Turn finished | a direct chat: `status.unread_since` becomes set, as recorded by `record_session_completion`; a mission slot: its observation goes from Working to Idle with `TurnOutcome::Completed` | Finished |
| Failed turn | a failed turn outcome (`failed_since`) | Response failed |
| Process exited | `Lifecycle::Error` (`error_since`) | Error · exit code N |
| Mission question | a `human_question` event is appended, keyed to its `ask_human` | @asker asked you |

The waits and failures use the short labels of the pane status control (`StatusKind::label` in `ui/agent_status.rs`), so the notification reads the same as the pane you land on. The exit code is the one detail added to the plain Error label.

Each trigger has its own switch in Settings → Notifications, separately for chats and missions; see the table there. Shell sessions never notify, which matches `record_session_completion` skipping them.

### When a notification posts

A notification posts only when its switch is on and the focused Runner window does not show the session:

- **Runner in the background**: it posts.
- **Runner in front, session in another tab or window**: it posts.
- **Session in a visible pane of the focused window**: nothing posts, whether or not that pane has keyboard focus, because its pane header already shows the status.

This is wider than `WindowRegistry::any_focused_displaying` (`crates/runner-daemon/src/windows.rs`), the check that decides whether a finished turn counts as unread. Each window reports a single `viewed_session_id` to the registry: the focused pane of the active tab, and only on the chat route (`report_current_subjects`, `surfaces/windowing.rs`). Using the registry alone would post for the other pane of a split you are looking at, and for every mission slot. The controller therefore asks the windows directly: the focused window's active tab and its panes, or the mission it shows. The app also keeps the registry snapshot in `AppStore::window_entries`.

For a mission, the focused window's mission workspace counts as showing a slot when the slot's tab is open. For a `human_question`, it counts when the feed is open, since the question's ask card is in the feed.

### Content

The system shows Runner's icon and name above two lines of text that Runner supplies:

| Trigger | Title | Body |
| --- | --- | --- |
| Direct chat | the chat title as the sidebar shows it | status text · project, e.g. "Approval needed · runner" |
| Mission slot | `@handle` · mission title | status text, e.g. "Approval needed" |
| Mission question | "@asker asked you" · mission title | the question itself; the system wraps and cuts it |

Notifications carry no action buttons. Clicking the notification is View, and the system's own close control is Dismiss. They carry no sound.

### Tags and replacement

The tag is the session id, or `question:<event id>` for a mission question, so each session has at most one notification. A newer trigger for the same session replaces the old one in place.

### Clicking a notification

The response callback, registered once at startup, receives the tag:

- **Direct chat**: Runner comes forward, raises the window that holds the session, activates the tab and focuses the pane. It uses the paths a sidebar pick already uses: `open_chat_session` (`surfaces/sidebar/activation.rs:176`), `PaneLayout::activate_session` (`pane_layout.rs:714`) and `focus_other_window` (`main.rs:1884`). A session that no window holds opens in the main window, as a sidebar pick would.
- **Mission slot**: the mission workspace opens on that slot.
- **Mission question**: the mission workspace opens on the feed with the ask card.

A click on a notification left over from an earlier run brings Runner forward; routing it to its session is best-effort.

### Withdrawing a notification

Runner calls `dismiss_system_notification` for a session when:

- the session comes into view: a window gains focus, or the tab or pane switches to it;
- the wait clears: the approval is given, or the user types;
- the session is archived or closed;
- the switch for its event is turned off;
- Runner quits, because nothing is left to route a click or to clear the notification later.

On macOS this removes the notification from the screen and from Notification Center. On Windows it hides the toasts posted by the running process, and older ones age out of the action centre.

### Settings → Notifications

Notifications get their own Settings pane rather than a row in General, and users choose which events notify them (Jason, 2026-10-13). The pane sits in the App group after Appearance, with a bell icon. Its header reads **Notifications**, "Choose when Runner tells you about an agent whose chat is out of view." Below the System row, every control is a toggle:

| Section | Row | Subtitle | Default |
| --- | --- | --- | --- |
| Chats | Needs your input | Approval needed, Answer needed or Needs you. | on |
| Chats | Finishes a turn | The agent is done and waiting for your next message. | on |
| Chats | Fails or exits | Response failed, or the agent process stopped with an error. | on |
| Missions | Asks you a question | A crew member asks you something through the feed. | on |
| Missions | Needs your input | A crew member waits for approval, an answer or input. | on |
| Missions | Finishes a turn | Every crew member's turns. Busy crews finish turns constantly. | off |
| Missions | Fails or exits | A crew member's response fails or its process stops with an error. | on |

- **No master switch in Runner** (Jason, 2026-10-13). The system's own setting that allows Runner's notifications is the only on/off. A second switch in Runner would allow a state where Runner's switch shows on, the system blocks Runner, and nothing arrives, and Runner could not explain it because gpui-pre does not report the system's permission. To silence everything, the user turns Runner off in the system's settings, uses Focus, or turns each event off.
- **Needs your input** covers the three waits together, and **Fails or exits** covers the failed turn and the process exit together. The pane offers no finer switches.
- **Storage**: one `NotificationSettings` value inside the app settings in `app_settings.rs`. Its fields are `chats` with `input`, `finished` and `failed`, and `missions` with `question`, `input`, `finished` and `failed`.
- **System row**: a System section at the top of the pane, above the switches, has one row. **Allow notifications, banner style and sound** has the subtitle "Set by macOS in System Settings → Notifications → Runner." and an **Open System Settings** button, which opens `x-apple.systempreferences:com.apple.Notifications-Settings.extension`. On Windows the subtitle names Settings → System → Notifications, and the button reads **Open Settings** and opens `ms-settings:notifications`. Allowing notifications, banner style, sound and Focus stay the system's settings and are not repeated in Runner.

## Implementation notes

- **Where the code goes**: a notification controller in `crates/runner-app/src/notifications.rs`. It is an app global like `GlobalAppStore`, not a field of `NativeRoot`: every window has its own `NativeRoot`, so a controller on each would post once per open window.
- **App identity**: call `cx.set_app_identity` once at startup. The identifier is `com.wycstudios.runner`, matching the installer shortcut's `AppUserModelID` in `script/windows/runner.iss:71`, and `com.wycstudios.runner-dev` for debug builds so development toasts stay separate. On macOS the bundle supplies the identity.
- **Trigger detection**: a pure function from the previous and next `AppStore::session_statuses` (`app_store.rs:267`, a map of `AgentStatus`), plus the newly appended mission events, to a list of `(session_id, reason)`. It runs on the refreshes that `StoreRefreshKind::for_event` already routes (`session/status`, and `event/appended` for `human_question`), so `runnerd` and the client protocol do not change. Keeping it pure keeps the edge rules unit-testable.
- **Content**: a pure function from the trigger and session to the `SystemNotification` title and body, tested for each row of the content table.
- **Response routing**: `on_system_notification_response` registered during app startup, mapping the tag back to a session or a question and running the View paths above.
- **Settings pane**: `SettingsPane::Notifications` in `surfaces/settings_page.rs`, with the route key `notifications`, the label Notifications and `bell.svg`. The lucide bell icon is added to `assets.rs`, and the pane goes into `APP_PANES` after Appearance.

## Non-goals

- A Runner-drawn popup window; the 2026-09-22 design was dropped on 2026-10-13.
- Sound, action buttons, and approving or answering from the notification itself.
- A switch limiting notifications to when Runner is in the background; it was dropped on 2026-10-13 as more confusing than useful.
- A master switch or mute inside Runner; the system's setting is the only on/off (2026-10-13). Showing in the pane that the system blocks Runner is a possible follow-up: gpui-pre does not report the permission, so it needs Runner's own call on each platform.
- Switches for each session or project, finer switches than the pane's, snoozing, and Time Sensitive notifications that break through Focus.
- Dock bounce and taskbar flashing; the system notification is the alert.
- A Dock badge count, a tray or menu-bar icon, and a view of everything that needs you. That view was [#552](https://github.com/yicheng47/runner/issues/552), closed for the desktop app on 2026-10-08 because it belongs in a mobile companion.
- Notifications while Runner is quit. With **When Runner quits** set to Keep running, `runnerd` keeps the sessions working, but no app is connected to post. Following agents from away from the app is the iOS companion's job ([#836](https://github.com/yicheng47/runner/issues/836)). Whether `runnerd`, which ships inside `Runner.app`, could post with the app's identity is a separate probe, not part of this feature.

## Implementation phases

1. **Design**: the Settings → Notifications pane and the content of each trigger in `design/specs/701-desktop-notifications.pen`, reviewed before any code.
2. **Direct chats**: app identity, the controller, trigger detection, content, response routing, withdrawal and the Notifications pane.
3. **Missions**: slot waits and crashes, `human_question`, and View into the mission workspace.

## Verification

- **Unit tests**:
  - Trigger detection posts only when a state is entered, and the first load sets the baseline.
  - Shells never post, and mission slots never post for a finished turn.
  - The visibility rule, including the unfocused pane of a visible split and a mission slot whose tab is open.
  - One tag per session.
  - Each switch: nothing posts for a switched-off event.
  - The defaults, including Missions → Finishes a turn off.
  - The content table, row by row.
- **Checks**: `runner-app` tests and workspace clippy.
- **Live QA on macOS**: notifications need an app bundle, so a plain `make run` build posts nothing.
  - QA builds with `make run` and drives the development build through the `.app` wrapper of the [full smoke-test procedure](../tests/full-smoke-test.md#macos-native-control-of-the-debug-executable): bundle id `com.wycstudios.runner.smoke-dev`, development data and daemon.
  - The wrapper has its own notification permission, separate from an installed Runner. QA accepts the permission prompt on the first notification.
  - If macOS refuses authorization for the unsigned wrapper, ad-hoc sign it with `codesign --force --sign - "$smoke_bundle"` and record that in the procedure.
  - Do not test with `script/bundle-mac --dev`. Its `Runner.app` uses the production bundle id `com.wycstudios.runner` and release data paths, so it would share notification permission and clicks with an installed Runner.
- **Live QA on Windows**: debug builds post toasts once `set_app_identity` has registered the development identity, so `.\make.cmd run` is enough.
- **Manual checks on both platforms**:
  - With Runner behind another app, a direct chat that reaches an approval prompt posts one notification without taking focus. Its title is the chat title and its body "Approval needed · <project>".
  - Clicking it brings Runner forward on the right window, tab and pane, including for a session in a second window (⇧⌘N).
  - With Runner in front and the session in another tab, a notification posts; switching to that tab withdraws it. Approving in the terminal withdraws it too.
  - In a split tab of the focused window, an approval wait in the pane without keyboard focus posts nothing.
  - A second trigger for the same chat replaces its notification rather than adding one.
  - In a mission, the lead's `ask_human` posts "@lead asked you · <mission>" with the question as the body, and clicking it opens the mission workspace on the feed. With the default switches, crew turns finishing post nothing; with Missions → Finishes a turn on, each one posts.
  - Turning an event's switch off withdraws its visible notifications, and nothing more posts for it.
  - After Don't Allow on the permission prompt, nothing shows; allowing Runner in System Settings brings notifications back without restarting it.
  - Open System Settings (Open Settings on Windows) lands on the system's notification settings.
  - Quitting Runner withdraws its notifications.
