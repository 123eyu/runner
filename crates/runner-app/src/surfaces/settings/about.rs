use gpui::prelude::*;
use gpui::{div, rems, svg, Context, CursorStyle, KeyDownEvent, Render, Window};
use runner_app::ui::{PaneHeader, SettingsCard};

use crate::theme;

pub(super) const DISCORD_URL: &str = "https://discord.gg/6BrfZdHKNf";
pub(super) const WECHAT_README_URL: &str = "https://github.com/yicheng47/runner#community";

pub(crate) struct AboutPane;

impl AboutPane {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Render for AboutPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(PaneHeader::new(
                crate::i18n::text("ui.about"),
                crate::i18n::text("ui.version_credits_and_links"),
            ))
            .child(super::app_identity_card(
                [div()
                    .truncate()
                    .text_size(theme::text_ui())
                    .text_color(theme::muted())
                    .child(crate::i18n::text("ui.local_cockpit_for_coding_agents"))
                    .into_any_element()],
                None,
            ))
            .child(SettingsCard::new([
                link_row(
                    "about-github",
                    "github.svg",
                    "GitHub",
                    Some("https://github.com/yicheng47/runner"),
                    None,
                ),
                link_row(
                    "about-documentation",
                    "book-text.svg",
                    crate::i18n::text("ui.documentation"),
                    Some("https://github.com/yicheng47/runner#readme"),
                    None,
                ),
                link_row(
                    "about-discord",
                    "discord.svg",
                    "Discord",
                    Some(DISCORD_URL),
                    None,
                ),
                link_row(
                    "about-wechat-readme",
                    "wechat.svg",
                    crate::i18n::text("community.wechat_readme"),
                    Some(WECHAT_README_URL),
                    None,
                ),
                link_row(
                    "about-license",
                    "scale.svg",
                    crate::i18n::text("ui.license"),
                    None,
                    Some("MIT"),
                ),
            ]))
            .child(
                div()
                    .debug_selector(|| "ABOUT_INVITE_GUIDANCE".into())
                    .text_size(theme::text_ui())
                    .text_color(theme::muted())
                    .child(crate::i18n::text("community.invite_guidance")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(theme::text_meta())
                    .text_color(theme::faint())
                    .child("© 2026 Yicheng Wang (Jason Wang)"),
            )
    }
}

fn link_row(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    url: Option<&'static str>,
    trailing: Option<&'static str>,
) -> gpui::AnyElement {
    let interactive = url.is_some();
    let mut row = div()
        .id(id)
        .debug_selector(move || id.into())
        .tab_index(if interactive { 0 } else { -1 })
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .cursor(if interactive {
            CursorStyle::PointingHand
        } else {
            CursorStyle::Arrow
        })
        .when(interactive, |row| {
            row.hover(|row| row.bg(theme::with_alpha(theme::raised(), 0.4)))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(rems(10. / 16.))
                .child(
                    svg()
                        .flex_none()
                        .path(icon)
                        .size(rems(14. / 16.))
                        .text_color(theme::muted()),
                )
                .child(
                    div()
                        .text_size(theme::text_body())
                        .text_color(theme::text())
                        .child(label),
                ),
        )
        .child(if let Some(trailing) = trailing {
            div()
                .text_size(theme::text_ui())
                .text_color(theme::faint())
                .child(trailing)
        } else {
            div().child(
                svg()
                    .flex_none()
                    .path("external-link.svg")
                    .size(rems(12. / 16.))
                    .text_color(theme::faint()),
            )
        });
    if let Some(url) = url {
        row = row.on_click(move |_, _, cx| cx.open_url(url)).on_key_down(
            move |event: &KeyDownEvent, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    cx.stop_propagation();
                    cx.open_url(url);
                }
            },
        );
    }
    row.into_any_element()
}
