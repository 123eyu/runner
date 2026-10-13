use gpui::prelude::*;
use gpui::{div, rems, AnyElement, Context};
use runner_app::ui::{
    Button, ButtonSize, ButtonVariant, PaneHeader, SettingsCard, SettingsRow, Toggle,
};

use crate::app_settings::NotificationSettings;
use crate::surfaces::settings_page::settings_section_heading;
use crate::NativeRoot;

impl NativeRoot {
    pub(crate) fn render_notifications_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        #[cfg(not(windows))]
        let (subtitle, button, url) = (
            "Set by macOS in System Settings → Notifications → Runner.",
            "Open System Settings",
            "x-apple.systempreferences:com.apple.Notifications-Settings.extension",
        );
        #[cfg(windows)]
        let (subtitle, button, url) = (
            "Set by Windows in Settings → System → Notifications → Runner.",
            "Open Settings",
            "ms-settings:notifications",
        );
        let system = SettingsRow::new(
            "Allow notifications, banner style and sound",
            Button::new("notification-system-settings", button)
                .icon("external-link.svg")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .on_press(move |_, cx| cx.open_url(url)),
        )
        .subtitle(subtitle)
        .into_any_element();
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
