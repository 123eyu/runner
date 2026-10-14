use gpui::prelude::*;
use gpui::{div, rems, AnyElement, Context};
use runner_app::ui::{
    Button, ButtonSize, ButtonVariant, PaneHeader, SettingsCard, SettingsRow, Toggle,
};

use crate::app_settings::NotificationSettings;
use crate::notifications::Permission;
use crate::surfaces::settings_page::settings_section_heading;
use crate::{theme, NativeRoot};

#[cfg(not(windows))]
const SETTINGS_BUTTON: &str = "Open System Settings";
#[cfg(windows)]
const SETTINGS_BUTTON: &str = "Open Settings";

#[derive(Debug, PartialEq, Eq)]
struct SystemRow {
    title: &'static str,
    subtitle: &'static str,
    allow: bool,
    warn: bool,
}

fn system_row(permission: Permission) -> SystemRow {
    let row = |title, subtitle| SystemRow {
        title,
        subtitle,
        allow: false,
        warn: false,
    };
    #[cfg(not(windows))]
    match permission {
        Permission::Unknown => row(
            "Allow notifications, banner style and sound",
            "Set by macOS in System Settings → Notifications → Runner.",
        ),
        Permission::NotAsked => SystemRow {
            allow: true,
            ..row(
                "Notifications aren't set up",
                "macOS asks the first time Runner notifies you, or you can allow them now.",
            )
        },
        Permission::Allowed => row(
            "Notifications are on",
            "Banner style and sound are set in System Settings → Notifications → Runner.",
        ),
        Permission::Denied => SystemRow {
            warn: true,
            ..row(
                "Notifications are off",
                "Not allowed in System Settings, so Runner can't notify you.",
            )
        },
    }
    #[cfg(windows)]
    match permission {
        Permission::Unknown => row(
            "Allow notifications, banner style and sound",
            "Set by Windows in Settings → System → Notifications → Runner.",
        ),
        Permission::NotAsked | Permission::Allowed => row(
            "Notifications are on",
            "Banners and sound are set in Settings → System → Notifications → Runner.",
        ),
        Permission::Denied => SystemRow {
            warn: true,
            ..row(
                "Notifications are off",
                "Not allowed in Windows Settings, so Runner can't notify you.",
            )
        },
    }
}

impl NativeRoot {
    pub(crate) fn render_notifications_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let row = system_row(crate::notifications::permission(cx));
        let control = if row.allow {
            Button::new("notification-allow", "Allow notifications")
                .icon("bell.svg")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Primary)
                .on_press(|_, cx| crate::notifications::request_permission(cx))
        } else {
            let url = crate::notifications::settings_url();
            Button::new("notification-system-settings", SETTINGS_BUTTON)
                .icon("external-link.svg")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .on_press(move |_, cx| cx.open_url(&url))
        };
        let mut system = SettingsRow::new(row.title, control).subtitle(row.subtitle);
        if row.warn {
            system = system.label_icon("triangle-alert.svg", theme::warning());
        }
        let system = system.into_any_element();
        let chats = vec![
            self.notification_toggle(
                "notification-chats-input",
                "Needs your input",
                "Approval requests, questions or other input.",
                |s| &mut s.chats.input,
                cx,
            ),
            self.notification_toggle(
                "notification-chats-finished",
                "Finishes a turn",
                "The agent is done and waiting for your next message.",
                |s| &mut s.chats.finished,
                cx,
            ),
            self.notification_toggle(
                "notification-chats-failed",
                "Fails or exits",
                "Response failed, or the agent process stopped with an error.",
                |s| &mut s.chats.failed,
                cx,
            ),
        ];
        let missions = vec![
            self.notification_toggle(
                "notification-missions-question",
                "Asks you a question",
                "A crew member asks you something through the feed.",
                |s| &mut s.missions.question,
                cx,
            ),
            self.notification_toggle(
                "notification-missions-input",
                "Needs your input",
                "A crew member waits for approval, an answer or input.",
                |s| &mut s.missions.input,
                cx,
            ),
            self.notification_toggle(
                "notification-missions-idle",
                "Goes idle",
                "Every crew member is idle, so the mission has stopped making progress.",
                |s| &mut s.missions.idle,
                cx,
            ),
            self.notification_toggle(
                "notification-missions-failed",
                "Fails or exits",
                "A crew member's response fails or its process stops with an error.",
                |s| &mut s.missions.failed,
                cx,
            ),
        ];
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(PaneHeader::new(
                "Notifications",
                "Choose when Runner notifies you while the app is in the background.",
            ))
            .children(
                [
                    ("System", vec![system]),
                    ("Chats", chats),
                    ("Missions", missions),
                ]
                .into_iter()
                .map(|(heading, rows)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(rems(8. / 16.))
                        .child(settings_section_heading(heading))
                        .child(SettingsCard::new(rows))
                }),
            )
            .into_any_element()
    }

    fn notification_toggle(
        &self,
        id: &'static str,
        label: &'static str,
        subtitle: &'static str,
        field: fn(&mut NotificationSettings) -> &mut bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        SettingsRow::new(label, self.notification_switch(id, field, cx))
            .subtitle(subtitle)
            .into_any_element()
    }

    fn notification_switch(
        &self,
        id: &'static str,
        field: fn(&mut NotificationSettings) -> &mut bool,
        cx: &mut Context<Self>,
    ) -> Toggle {
        let enabled = *field(&mut self.app_store.read(cx).settings.notifications.clone());
        let root = cx.entity();
        Toggle::new(id, enabled).on_change(move |enabled, _, cx| {
            root.update(cx, |this, cx| {
                this.update_app_settings(cx, true, |settings| {
                    *field(&mut settings.notifications) = enabled;
                    true
                });
            });
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_row_follows_the_system_permission() {
        let unknown = system_row(Permission::Unknown);
        assert!(!unknown.allow && !unknown.warn);
        assert_eq!(unknown.title, "Allow notifications, banner style and sound");

        let allowed = system_row(Permission::Allowed);
        assert!(!allowed.allow && !allowed.warn);
        assert_eq!(allowed.title, "Notifications are on");

        let denied = system_row(Permission::Denied);
        assert!(!denied.allow && denied.warn);
        assert_eq!(denied.title, "Notifications are off");

        let not_asked = system_row(Permission::NotAsked);
        #[cfg(not(windows))]
        {
            assert!(not_asked.allow && !not_asked.warn);
            assert_eq!(not_asked.title, "Notifications aren't set up");
        }
        #[cfg(windows)]
        assert_eq!(not_asked, allowed);
    }
}
