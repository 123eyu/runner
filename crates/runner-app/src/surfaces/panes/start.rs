use super::*;

pub(super) fn start_action_shortcut(
    binding: &str,
    overrides: &keymap::KeymapOverrides,
) -> Option<String> {
    keymap::effective_binding(binding, overrides).map(|combo| keymap::format_combo(&combo))
}

pub(super) fn start_action_badge(shortcut: String, compact: bool) -> Div {
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .when(compact, |badge| {
            badge
                .py(rems(3. / 16.))
                .px(rems(7. / 16.))
                .rounded(rems(5. / 16.))
                .text_size(rems(11. / 16.))
        })
        .when(!compact, |badge| {
            badge
                .h(rems(28. / 16.))
                .min_w(rems(58. / 16.))
                .px(rems(7. / 16.))
                .rounded(rems(6. / 16.))
                .text_size(rems(12. / 16.))
        })
        .bg(theme::raised())
        .border_1()
        .border_color(theme::border_strong())
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme::muted())
        .child(shortcut)
}

pub(super) fn start_surface_action_row(
    binding: &'static str,
    label: &'static str,
    description: &'static str,
    icon: &'static str,
    action: impl gpui::Action,
    overrides: &keymap::KeymapOverrides,
) -> impl IntoElement {
    div()
        .id(binding)
        .h(rems(84. / 16.))
        .flex_none()
        .py(rems(1.))
        .px(rems(20. / 16.))
        .flex()
        .items_center()
        .gap(rems(1.))
        .rounded(rems(8. / 16.))
        .cursor_pointer()
        .hover(|row| row.bg(theme::sidebar_selected()))
        .child(
            svg()
                .path(icon)
                .size(rems(22. / 16.))
                .flex_none()
                .text_color(if binding == "new-chat" {
                    theme::accent()
                } else {
                    theme::muted()
                }),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(rems(5. / 16.))
                .child(
                    div()
                        .text_size(rems(15. / 16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::text())
                        .child(label),
                )
                .child(
                    div()
                        .text_size(rems(13. / 16.))
                        .text_color(theme::muted())
                        .child(description),
                ),
        )
        .children(
            start_action_shortcut(binding, overrides)
                .map(|shortcut| start_action_badge(shortcut, false)),
        )
        .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
}

pub(super) fn render_start_surface(
    overrides: &keymap::KeymapOverrides,
    client: &crate::surfaces::front_page::FrontPageClient,
) -> Div {
    div()
        .when(cfg!(test), |surface| {
            surface.debug_selector(|| "START_SURFACE".into())
        })
        .relative()
        .flex_1()
        .min_h(px(0.))
        .flex()
        .items_center()
        .justify_center()
        .pb(rems(48. / 16.))
        .bg(theme::bg())
        .child(div().absolute().inset_0().bg(linear_gradient(
            180.,
            linear_color_stop(theme::with_alpha(theme::accent(), 18. / 255.), 0.),
            linear_color_stop(theme::with_alpha(theme::accent(), 0.), 0.6),
        )))
        .child(
            div()
                .when(cfg!(test), |column| {
                    column.debug_selector(|| "START_SURFACE_COLUMN".into())
                })
                .relative()
                .w(rems(480. / 16.))
                .flex_none()
                .flex()
                .flex_col()
                .items_center()
                .gap(rems(28. / 16.))
                .child(
                    svg()
                        .path("brand-mark-light.svg")
                        .size(rems(44. / 16.))
                        .text_color(theme::accent()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(rems(8. / 16.))
                        .child(
                            div()
                                .text_size(rems(28. / 16.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text())
                                .child(crate::i18n::text("ui.start_something_new")),
                        )
                        .child(
                            div()
                                .text_size(rems(14. / 16.))
                                .text_color(theme::muted())
                                .child(crate::i18n::text("ui.choose_how_you_want_to_work")),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .p(rems(6. / 16.))
                        .flex()
                        .flex_col()
                        .gap(rems(2. / 16.))
                        .bg(theme::panel())
                        .border_1()
                        .border_color(theme::border())
                        .rounded(rems(12. / 16.))
                        .children((client.supports(&["catalog", "startChat"])).then(|| {
                            start_surface_action_row(
                                "new-chat",
                                crate::i18n::text("ui.new_chat"),
                                crate::i18n::text("ui.work_with_an_agent_or_a_saved_role"),
                                "message-square-plus.svg",
                                NewTab,
                                overrides,
                            )
                        }))
                        .children((client.supports(&["startShell", "layout"])).then(|| {
                            start_surface_action_row(
                                "new-terminal",
                                crate::i18n::text("ui.new_terminal"),
                                crate::i18n::text("ui.run_commands_in_a_shell"),
                                "terminal.svg",
                                NewTerminal,
                                overrides,
                            )
                        }))
                        .children((client.supports(&["catalog", "startMission"])).then(|| {
                            start_surface_action_row(
                                "new-mission",
                                crate::i18n::text("ui.new_mission"),
                                crate::i18n::text("ui.work_with_a_crew_of_agents"),
                                "users.svg",
                                NewMission,
                                overrides,
                            )
                        })),
                ),
        )
}

pub(super) fn empty_pane_action_row(
    binding: &str,
    label: &'static str,
    icon: &'static str,
    overrides: &keymap::KeymapOverrides,
) -> Div {
    div()
        .h(rems(40. / 16.))
        .flex_none()
        .pl(rems(12. / 16.))
        .pr(rems(10. / 16.))
        .flex()
        .items_center()
        .gap(rems(10. / 16.))
        .rounded(rems(6. / 16.))
        .cursor_pointer()
        .hover(|row| row.bg(theme::sidebar_selected()))
        .child(
            svg()
                .path(icon)
                .size(rems(1.))
                .flex_none()
                .text_color(if binding == "new-chat" {
                    theme::accent()
                } else {
                    theme::muted()
                }),
        )
        .child(
            div()
                .flex_1()
                .text_size(rems(13. / 16.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text())
                .child(label),
        )
        .children(
            start_action_shortcut(binding, overrides)
                .map(|shortcut| start_action_badge(shortcut, true)),
        )
}
