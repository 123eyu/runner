use super::*;

pub(crate) type TerminalDrawerSessionCallback = Rc<dyn Fn(String, &mut Window, &mut App)>;

pub(crate) type TerminalDrawerActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;

pub(crate) struct TerminalDrawerCallbacks {
    pub can_add: bool,
    pub can_close: bool,
    pub activate: TerminalDrawerSessionCallback,
    pub close: TerminalDrawerSessionCallback,
    pub add: TerminalDrawerActionCallback,
    pub hide: TerminalDrawerActionCallback,
}

pub(crate) fn render_terminal_drawer_strip(
    id_prefix: &str,
    shells: &[String],
    active_id: Option<&str>,
    labels: &[String],
    callbacks: TerminalDrawerCallbacks,
) -> AnyElement {
    let TerminalDrawerCallbacks {
        can_add,
        can_close,
        activate,
        close,
        add,
        hide,
    } = callbacks;
    let chips = shells
        .iter()
        .enumerate()
        .map(|(index, session_id)| {
            let active = active_id == Some(session_id.as_str());
            let label = labels.get(index).cloned().unwrap_or_else(|| "shell".into());
            let activate = Rc::clone(&activate);
            let close = Rc::clone(&close);
            let activate_id = session_id.clone();
            let close_id = session_id.clone();
            div()
                .id(SharedString::from(format!(
                    "{id_prefix}-drawer-chip-{session_id}"
                )))
                .flex_none()
                .h(rems(24. / 16.))
                .max_w(rems(180. / 16.))
                .px_2()
                .flex()
                .items_center()
                .gap(rems(6. / 16.))
                .rounded(rems(5. / 16.))
                .bg(if active {
                    theme::raised()
                } else {
                    gpui::transparent_black()
                })
                .text_color(if active {
                    theme::text()
                } else {
                    theme::muted()
                })
                .cursor_pointer()
                .hover(|chip| chip.bg(theme::raised()))
                .child(
                    svg()
                        .path("terminal.svg")
                        .size(rems(12. / 16.))
                        .flex_none()
                        .text_color(theme::muted()),
                )
                .child(
                    div()
                        .min_w(px(0.))
                        .truncate()
                        .text_size(theme::text_ui())
                        .child(label),
                )
                .children(can_close.then(|| {
                    IconButton::new(
                        SharedString::from(format!("{id_prefix}-close-drawer-shell-{session_id}")),
                        "close.svg",
                    )
                    .size(IconButtonSize::Xs)
                    .stop_click_propagation(true)
                    .tooltip(crate::i18n::text("ui.close_terminal"))
                    .on_press(move |window, cx| close(close_id.clone(), window, cx))
                }))
                .on_click(move |_, window, cx| activate(activate_id.clone(), window, cx))
        })
        .collect::<Vec<_>>();
    let add_id = SharedString::from(format!("{id_prefix}-add-drawer-shell"));
    let hide_id = SharedString::from(format!("{id_prefix}-hide-terminal-drawer"));
    div()
        .flex_none()
        .h(rems(32. / 16.))
        .w_full()
        .pl(rems(10. / 16.))
        .pr(rems(8. / 16.))
        .flex()
        .items_center()
        .gap(rems(6. / 16.))
        .border_b_1()
        .border_color(theme::border())
        .map(|element| {
            #[cfg(test)]
            let element = crate::theme_snapshot::record_fill("DRAWER_HEADER", element);
            element
        })
        .children(chips)
        .children(can_add.then(|| {
            IconButton::new(add_id, "plus.svg")
                .size(IconButtonSize::Sm)
                .tooltip(crate::i18n::text("ui.new_terminal"))
                .on_press(move |window, cx| add(window, cx))
        }))
        .child(div().flex_1())
        .child(
            IconButton::new(hide_id, "chevron-down.svg")
                .size(IconButtonSize::Sm)
                .tooltip(crate::i18n::text("ui.hide_terminal_drawer"))
                .on_press(move |window, cx| hide(window, cx)),
        )
        .into_any_element()
}

pub(crate) struct TerminalSurface {
    pub(crate) terminal: Arc<TerminalMirror>,
    pub(crate) interaction: Entity<TerminalInteraction>,
    pub(crate) input: Entity<TerminalInput>,
    pub(crate) focus: FocusHandle,
    pub(crate) scrollbar: Entity<Scrollbar>,
    pub(crate) style: crate::terminal::element::TerminalStyle,
    pub(crate) interactive: bool,
    pub(crate) resize_owner: bool,
    pub(crate) scrollable: bool,
}

impl TerminalSurface {
    pub(crate) fn render(self) -> Div {
        let background = crate::terminal::element::to_hsla(self.style.palette.background, 1.);
        div()
            .absolute()
            .inset_0()
            .key_context("Terminal")
            .track_focus(&self.focus)
            .flex()
            .py_3()
            .pl_3()
            .pr_1()
            .bg(background)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_w(px(0.))
                    .min_h(px(0.))
                    .pr(runner_app::ui::terminal_scrollbar_gutter())
                    .child(
                        TerminalElement::new(
                            self.terminal,
                            self.interaction,
                            self.input,
                            self.focus,
                            self.interactive,
                            self.resize_owner,
                            self.style,
                        )
                        .scrollable(self.scrollable)
                        .cached(),
                    )
                    .child(self.scrollbar),
            )
    }
}

impl NativeRoot {
    pub(crate) fn render_terminal_drawer(
        &mut self,
        layout: &PaneLayout,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active_id = layout.active_drawer_shell().map(str::to_owned);
        let strip_layout = layout.clone();
        let strip = self.cached_region(
            "chat-drawer-strip",
            gpui::StyleRefinement::default()
                .w_full()
                .h(rems(32. / 16.))
                .flex_none(),
            move |root, _, cx| root.render_chat_drawer_strip(&strip_layout, cx),
            cx,
        );

        let body = active_id
            .as_deref()
            .and_then(|session_id| self.render_drawer_terminal(session_id, cx))
            .unwrap_or_else(|| {
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(theme::text_ui())
                    .text_color(theme::faint())
                    .child(crate::i18n::text("ui.terminal_unavailable"))
                    .into_any_element()
            });
        div()
            .id("terminal-drawer")
            .when(cfg!(test), |drawer| {
                drawer.debug_selector(|| "CHAT_TERMINAL_DRAWER".into())
            })
            .track_focus(&self.drawer_focus)
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(strip)
            .child(body)
            .into_any_element()
    }

    pub(super) fn render_chat_drawer_strip(
        &mut self,
        layout: &PaneLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active_id = layout.active_drawer_shell().map(str::to_owned);
        let labels = layout
            .drawer_shells()
            .iter()
            .map(|session_id| {
                self.front_page_session(session_id, cx)
                    .map(|entry| entry.default_label())
                    .unwrap_or_else(|| "shell".into())
            })
            .collect::<Vec<_>>();
        let activate_root = cx.entity();
        let close_root = activate_root.clone();
        let add_root = activate_root.clone();
        let hide_root = activate_root.clone();
        render_terminal_drawer_strip(
            "chat",
            layout.drawer_shells(),
            active_id.as_deref(),
            &labels,
            TerminalDrawerCallbacks {
                can_add: self
                    .front_page_client(cx)
                    .supports(&["startShell", "layout"]),
                can_close: self.front_page_client(cx).supports(&["close", "layout"]),
                activate: Rc::new(move |session_id, window, cx| {
                    activate_root.update(cx, |this, cx| {
                        this.activate_terminal_drawer_shell(&session_id, window, cx)
                    });
                }),
                close: Rc::new(move |session_id, window, cx| {
                    close_root.update(cx, |this, cx| {
                        this.request_close_drawer_shell(&session_id, window, cx)
                    });
                }),
                add: Rc::new(move |window, cx| {
                    add_root.update(cx, |this, cx| this.add_terminal_drawer_shell(window, cx));
                }),
                hide: Rc::new(move |window, cx| {
                    hide_root.update(cx, |this, cx| this.hide_terminal_drawer(window, cx));
                }),
            },
        )
    }

    pub(super) fn render_drawer_terminal(
        &mut self,
        session_id: &str,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let entry = self.front_page_session(session_id, cx)?;
        let session_id = entry.session_id.clone();
        let transition = self
            .chat_transitions
            .get(&session_id)
            .filter(|_| self.remote_client.is_local())
            .map(|item| item.kind);
        let overlay = resolve_pane_overlay(
            false,
            transition,
            entry.status,
            entry.resumable,
            self.session_exit_codes
                .get(&session_id)
                .filter(|_| self.remote_client.is_local())
                .copied()
                .flatten(),
        );
        let interactive = self.cached_session_is_interactive(&session_id, cx);
        let scrollable = self.route == AppRoute::Chat && transition.is_none();
        let terminal_style = self.terminal_style(cx);
        let terminal_background =
            crate::terminal::element::to_hsla(terminal_style.palette.background, 1.);
        let terminal_surface = if self.remote_client.is_active() {
            self.remote_client
                .render_remote_terminal_content(&session_id, cx)
        } else if let Some(chat) = self.attached.get(&session_id) {
            let terminal = Arc::clone(&chat.terminal);
            let terminal_interaction = chat.terminal_interaction.clone();
            let terminal_scrollbar = chat.terminal_scrollbar.clone();
            let terminal_input = chat.terminal_input.clone();
            let terminal_focus = chat.terminal_focus.clone();
            let resize_owner = scrollable;
            let key_session_id = session_id.clone();
            let copy_session_id = session_id.clone();
            let paste_session_id = session_id.clone();
            let drop_session_id = session_id.clone();
            TerminalSurface {
                terminal,
                interaction: terminal_interaction,
                input: terminal_input,
                focus: terminal_focus,
                scrollbar: terminal_scrollbar,
                style: terminal_style,
                interactive,
                resize_owner,
                scrollable,
            }
            .render()
            .id(SharedString::from(format!("drawer-terminal-{session_id}")))
            .opacity(match overlay {
                PaneOverlayState::Starting | PaneOverlayState::Resuming => 0.,
                PaneOverlayState::Ended { .. } => 0.45,
                PaneOverlayState::Archiving | PaneOverlayState::None => 1.,
            })
            .on_action(cx.listener(move |this, action: &Copy, window, cx| {
                this.on_terminal_copy(&copy_session_id, action, window, cx);
            }))
            .when(
                interactive
                    || self
                        .attached
                        .get(&session_id)
                        .is_some_and(|chat| !chat.terminal.control().desktop()),
                |surface| {
                    surface
                        .on_key_down(cx.listener(move |this, event, window, cx| {
                            this.on_key_down(&key_session_id, event, window, cx);
                        }))
                        .on_action(cx.listener(move |this, action, window, cx| {
                            this.on_paste(&paste_session_id, action, window, cx);
                        }))
                        .child(runner_app::ui::file_drop_target(
                            &session_id,
                            cx.listener(move |this, paths, window, cx| {
                                this.on_file_drop(None, &drop_session_id, paths, window, cx);
                            }),
                        ))
                },
            )
            .into_any_element()
        } else {
            div()
                .absolute()
                .inset_0()
                .bg(terminal_background)
                .text_size(theme::text_ui())
                .text_color(theme::faint())
                .when(matches!(overlay, PaneOverlayState::None), |surface| {
                    surface
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(crate::i18n::text("ui.attaching_terminal"))
                })
                .into_any_element()
        };
        let overlay_element = match overlay {
            PaneOverlayState::Resuming => Some(
                SessionOverlay::transition(
                    format!("drawer-resuming-{session_id}"),
                    SessionOverlayKind::Resuming,
                )
                .label(crate::i18n::text("ui.restarting_terminal"))
                .into_any_element(),
            ),
            PaneOverlayState::Starting => Some(
                SessionOverlay::transition(
                    format!("drawer-starting-{session_id}"),
                    SessionOverlayKind::Starting,
                )
                .label(crate::i18n::text("ui.starting_terminal"))
                .into_any_element(),
            ),
            PaneOverlayState::Ended { exit_code, .. } => {
                let restart_root = cx.entity();
                let close_root = restart_root.clone();
                let restart_id = session_id.clone();
                let close_id = session_id.clone();
                Some(
                    SessionOverlay::shell_exited(
                        format!("drawer-ended-{session_id}"),
                        shell_exited_subtitle(
                            exit_code,
                            &entry.default_label(),
                            entry.cwd.as_deref(),
                            self.remote_client
                                .is_local()
                                .then(runner_core::app_paths::home_dir)
                                .flatten()
                                .as_deref()
                                .and_then(|home| home.to_str()),
                        ),
                        move |window, cx| {
                            restart_root.update(cx, |this, cx| {
                                this.resume_drawer_shell(&restart_id, window, cx)
                            });
                        },
                        move |window, cx| {
                            close_root.update(cx, |this, cx| {
                                this.close_drawer_shell(&close_id, window, cx)
                            });
                        },
                    )
                    .actions_visible(
                        self.front_page_client(cx).supports(&["resume"]),
                        self.front_page_client(cx).supports(&["close", "layout"]),
                    )
                    .into_any_element(),
                )
            }
            PaneOverlayState::Archiving | PaneOverlayState::None => None,
        };
        let terminal_surface = if let Some(layout) = self.tabs.active().filter(|layout| {
            layout.drawer_shells().iter().any(|id| {
                self.attached
                    .get(id)
                    .is_some_and(|chat| !chat.terminal.control().desktop())
            })
        }) {
            crate::terminal::control::measure_panes(
                div().absolute().inset_0().child(terminal_surface),
                &layout
                    .drawer_shells()
                    .iter()
                    .filter_map(|id| self.attached.get(id))
                    .collect::<Vec<_>>(),
                self.terminal_style(cx),
            )
            .into_any_element()
        } else {
            terminal_surface
        };
        let remote = self
            .attached
            .get(&session_id)
            .filter(|_| self.remote_client.is_local())
            .filter(|chat| {
                entry.status == SessionStatus::Running && !chat.terminal.control().desktop()
            });
        let group = self
            .tabs
            .active()
            .is_some_and(|layout| self.tab_control_covered(layout, cx));
        let remote_overlay = (!group)
            .then(|| {
                remote.map(|chat| {
                    crate::terminal::control::cover(
                        div().absolute().inset_0(),
                        format!("take-back-drawer-{session_id}"),
                        &[chat],
                        crate::terminal::control::GroupKind::Tab,
                    )
                })
            })
            .flatten();
        let focus_id = session_id.clone();
        Some(
            div()
                .relative()
                .flex_1()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(terminal_background)
                .child(terminal_surface)
                .children(overlay_element)
                .children(remote_overlay)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        this.focus_drawer_terminal(&focus_id, window, cx);
                    }),
                )
                .into_any_element(),
        )
    }
}
