use super::*;

impl NativeRoot {
    pub(crate) fn render_pane(
        &mut self,
        leaf: &PaneLeaf,
        layout: &PaneLayout,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        #[cfg(test)]
        crate::render_counts::count_render(&format!("pane:{}", leaf.id), cx);
        let focused = layout.focused_pane_id == leaf.id;
        let grouped = layout.root.leaves().len() > 1;
        let pane_id = leaf.id.clone();
        let pane_id_for_focus = pane_id.clone();
        let entry = leaf
            .session_id
            .as_deref()
            .and_then(|session_id| self.front_page_session(session_id, cx));
        let secondary = entry.as_ref().map(|entry| {
            if self.remote_client.is_active() {
                Default::default()
            } else {
                self.cached_chat_secondary_state(&entry.session_id)
            }
        });
        let pane_session_for_focus = entry.as_ref().map(|entry| entry.session_id.clone());
        let close_root = cx.entity();
        let header = grouped.then(|| {
            let split_menu = self
                .front_page_client(cx)
                .supports(if self.active_tab_is_terminal(cx) {
                    &["layout", "startShell"]
                } else {
                    &["layout"]
                })
                .then(|| self.split_menu(SplitMenuSurface::Pane, &layout.id, &pane_id, cx));
            let grip = div()
                .w(rems(20. / 16.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .id(SharedString::from(format!("pane-grip-{pane_id}")))
                        .group("pane-grip")
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(rems(20. / 16.))
                        // GPUI captures the base cursor for the drag, before hover refinement.
                        .cursor(CursorStyle::ClosedHand)
                        .hover(|grip| grip.cursor(CursorStyle::OpenHand))
                        .child(
                            svg()
                                .path("grip-horizontal.svg")
                                .size(rems(12. / 16.))
                                .text_color(theme::faint())
                                .group_hover("pane-grip", |icon| icon.text_color(theme::text())),
                        )
                        .on_drag(
                            PaneDrag {
                                tab_id: layout.id.clone(),
                                pane_id: pane_id.clone(),
                                label: entry
                                    .as_ref()
                                    .map(|entry| entry.label(None))
                                    .unwrap_or_else(|| crate::i18n::text("ui.empty").into()),
                                icon: pane_identity_icon(
                                    entry.as_ref().map(|entry| entry.agent_runtime.as_str()),
                                ),
                                live: entry
                                    .as_ref()
                                    .is_some_and(|entry| entry.status == SessionStatus::Running),
                            },
                            |drag: &PaneDrag, _, _, cx: &mut App| cx.new(|_| drag.clone()),
                        ),
                );
            let close_pane_id = pane_id.clone();
            let close_behavior =
                pane_close_behavior(entry.as_ref().map(|entry| entry.agent_runtime.as_str()));
            let close_session_id = entry.as_ref().map(|entry| entry.session_id.clone());
            let rename_input = entry.as_ref().and_then(|entry| {
                self.pane_rename
                    .as_ref()
                    .filter(|rename| rename.session_id == entry.session_id)
                    .map(|rename| rename.input.clone())
            });
            let identity = if let Some(entry) = entry.as_ref() {
                let icon = pane_identity_icon(Some(&entry.agent_runtime));
                let session_id = entry.session_id.clone();
                let live = self
                    .remote_client
                    .is_local()
                    .then(|| self.attached_title(&session_id, cx))
                    .flatten();
                let label = entry.label(live.as_deref());
                let manual_title = entry.title.clone().unwrap_or_default();
                let placeholder = entry.default_label();
                let program_status = self
                    .front_page_status(&entry.session_id, cx)
                    .is_some_and(|status| status.program_status);
                let status =
                    pane_identity_shows_status(&entry.agent_runtime, program_status).then(|| {
                        let mut status = self.front_page_status(&entry.session_id, cx).unwrap();
                        if let Some(transition) = self
                            .chat_transitions
                            .get(&entry.session_id)
                            .filter(|_| self.remote_client.is_local())
                        {
                            status.kind = if transition.kind == TransitionKind::Resuming {
                                runner_app::ui::agent_status::StatusKind::Resuming
                            } else {
                                runner_app::ui::agent_status::StatusKind::Starting
                            };
                            status.estimated = false;
                        }
                        status
                    });
                let disabled = self.session_lifecycle_disabled(&session_id, cx)
                    || secondary.as_ref().is_some_and(|state| state.secondary);
                let menu = (rename_input.is_none()
                    && !pane_action_indexes(entry, &self.front_page_client(cx)).is_empty())
                .then(|| self.pane_action_menu(entry, disabled, cx));
                let name = if let Some(input) = rename_input {
                    div()
                        .ml_2()
                        .w(rems(144. / 16.))
                        .min_w(rems(64. / 16.))
                        .child(input)
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                            match pane_rename_key(&event.keystroke.key) {
                                Some(PaneRenameKey::Submit) => {
                                    cx.stop_propagation();
                                    this.submit_pane_rename(window, cx);
                                }
                                Some(PaneRenameKey::Cancel) => {
                                    cx.stop_propagation();
                                    this.cancel_pane_rename(window, cx);
                                }
                                None => {}
                            }
                        }))
                        .into_any_element()
                } else {
                    let rename_root = cx.entity();
                    div()
                        .id(SharedString::from(format!("pane-name-{session_id}")))
                        .ml_2()
                        .min_w(px(0.))
                        .truncate()
                        .text_size(theme::text_ui())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if entry.status != SessionStatus::Running {
                            theme::faint()
                        } else if focused {
                            theme::text()
                        } else {
                            theme::muted()
                        })
                        .child(label.clone())
                        .on_click(move |event, window, cx| {
                            if event.click_count() != 2 {
                                return;
                            }
                            cx.stop_propagation();
                            let session_id = session_id.clone();
                            let manual_title = manual_title.clone();
                            let placeholder = placeholder.clone();
                            rename_root.update(cx, |this, cx| {
                                this.begin_pane_rename(
                                    session_id,
                                    manual_title,
                                    placeholder,
                                    window,
                                    cx,
                                )
                            });
                        })
                        .into_any_element()
                };
                div()
                    .min_w(px(0.))
                    .flex_1()
                    .flex()
                    .items_center()
                    .child(icon.render(
                        rems(12. / 16.),
                        icon.color(
                            if focused {
                                theme::accent()
                            } else {
                                theme::faint()
                            },
                            entry.status == SessionStatus::Running,
                        ),
                        entry.status == SessionStatus::Running,
                    ))
                    .child(name)
                    .children(
                        entry
                            .local
                            .as_ref()
                            .and_then(|entry| self.claude_account_details(entry, cx))
                            .map(|(name, _, _)| account_chip(name)),
                    )
                    .children(menu)
                    .children(status.map(|status| {
                        let width = self
                            .front_page_pane_bounds()
                            .get(&PaneKey::new(&layout.id, &pane_id))
                            .map_or(480., |size| {
                                f32::from(size.width) / (f32::from(window.rem_size()) / 16.)
                            });
                        let target = pane_id.clone();
                        div()
                            .id(SharedString::from(format!("pane-status-{target}")))
                            .ml_2()
                            .flex_none()
                            .child(runner_app::ui::agent_status::pane_status_indicator(
                                status.clone(),
                                status.shows_label(width),
                                status.shows_detail(width),
                                SharedString::from(format!("status-{target}")),
                                window,
                                cx,
                            ))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.focus_pane(&target, cx);
                                this.mark_active_tab_viewed(window, cx);
                                this.focus_active_terminal(window, cx);
                            }))
                            .into_any_element()
                    }))
                    .into_any_element()
            } else {
                div()
                    .min_w(px(0.))
                    .flex_1()
                    .flex()
                    .items_center()
                    .child(pane_identity_icon(None).render(rems(12. / 16.), theme::faint(), false))
                    .child(
                        div()
                            .ml_2()
                            .text_size(theme::text_ui())
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::faint())
                            .child(crate::i18n::text("ui.empty")),
                    )
                    .into_any_element()
            };
            div()
                .flex_none()
                .h(rems(PANE_HEADER_HEIGHT / 16.))
                .px(rems(8. / 16.))
                .flex()
                .items_center()
                .bg(theme::panel())
                .border_b_1()
                .border_color(theme::border())
                .map(|element| {
                    #[cfg(test)]
                    let element = crate::theme_snapshot::record_fill("PANE_IDENTITY_LINE", element);
                    element.when(self.remote_client.is_active(), |element| {
                        element.debug_selector({
                            let id = leaf.session_id.clone().unwrap_or_else(|| pane_id.clone());
                            move || format!("PANE_HEADER:{id}")
                        })
                    })
                })
                .child(identity)
                .children(
                    self.front_page_client(cx)
                        .supports(&["layout"])
                        .then_some(grip),
                )
                .child(
                    div()
                        .min_w(px(0.))
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .children(split_menu)
                        .children(
                            self.front_page_client(cx)
                                .supports(match close_behavior {
                                    PaneCloseBehavior::CloseTerminal => &["close", "layout"],
                                    PaneCloseBehavior::ArchiveChat => &["archive", "layout"],
                                    PaneCloseBehavior::LayoutOnly => &["layout"],
                                })
                                .then(|| {
                                    div()
                                        .flex_none()
                                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                            cx.stop_propagation()
                                        })
                                        .child(
                                            IconButton::new(
                                                SharedString::from(format!(
                                                    "close-pane-{close_pane_id}"
                                                )),
                                                "close.svg",
                                            )
                                            .size(IconButtonSize::Sm)
                                            .tooltip(crate::i18n::text("ui.close_pane"))
                                            .on_press(
                                                move |window, cx| {
                                                    let pane_id = close_pane_id.clone();
                                                    let session_id = close_session_id.clone();
                                                    close_root.update(cx, |this, cx| {
                                                        match (close_behavior, session_id) {
                                                            (
                                                                PaneCloseBehavior::CloseTerminal,
                                                                Some(session_id),
                                                            ) => {
                                                                this.request_close_terminal_pane(
                                                                    &pane_id,
                                                                    &session_id,
                                                                    window,
                                                                    cx,
                                                                );
                                                            }
                                                            (
                                                                PaneCloseBehavior::ArchiveChat,
                                                                Some(session_id),
                                                            ) => {
                                                                this.request_close_chat_pane(
                                                                    &pane_id,
                                                                    &session_id,
                                                                    window,
                                                                    cx,
                                                                );
                                                            }
                                                            (PaneCloseBehavior::LayoutOnly, _) => {
                                                                this.close_pane(
                                                                    &pane_id, window, cx,
                                                                );
                                                            }
                                                            (
                                                                PaneCloseBehavior::CloseTerminal
                                                                | PaneCloseBehavior::ArchiveChat,
                                                                None,
                                                            ) => {}
                                                        }
                                                    });
                                                },
                                            ),
                                        )
                                }),
                        ),
                )
        });

        let body: AnyElement = if let Some(entry) = entry.as_ref() {
            let session_id = entry.session_id.clone();
            let secondary_state = secondary.clone().unwrap_or_default();
            let fork_materializing = self.remote_client.is_local()
                && super::super::chat::fork_materializing(&self.forking_sessions, &session_id);
            let transition = if fork_materializing {
                Some(TransitionKind::Starting)
            } else {
                self.chat_transitions
                    .get(&session_id)
                    .filter(|_| self.remote_client.is_local())
                    .map(|item| item.kind)
            };
            let overlay = resolve_pane_overlay(
                self.sidebar_archiving_session(&session_id, cx),
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
            let scrollable = self.route == AppRoute::Chat
                && transition.is_none()
                && !self.sidebar_archiving_session(&session_id, cx);
            let terminal_style = self.terminal_style(cx);
            let terminal_background =
                crate::terminal::element::to_hsla(terminal_style.palette.background, 1.);
            let terminal_surface = if self.remote_client.is_active() {
                self.remote_client
                    .render_remote_terminal_content(&session_id, cx)
            } else if secondary_state.secondary
                && self
                    .attached
                    .get(&session_id)
                    .is_none_or(|chat| chat.terminal.control().desktop())
            {
                div()
                    .absolute()
                    .inset_0()
                    .bg(terminal_background)
                    .into_any_element()
            } else if let Some(chat) = self.attached.get(&session_id) {
                let terminal = Arc::clone(&chat.terminal);
                let terminal_interaction = chat.terminal_interaction.clone();
                let terminal_scrollbar = chat.terminal_scrollbar.clone();
                let terminal_input = chat.terminal_input.clone();
                let terminal_focus = chat.terminal_focus.clone();
                let resize_owner = scrollable
                    && !secondary_state.secondary
                    && layout.is_resize_owner(&pane_id, &session_id);
                let key_session_id = session_id.clone();
                let copy_session_id = session_id.clone();
                let paste_session_id = session_id.clone();
                let drop_session_id = session_id.clone();
                let drop_pane_id = pane_id.clone();
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
                .id(SharedString::from(format!("terminal-{session_id}")))
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
                                    this.on_file_drop(
                                        Some(&drop_pane_id),
                                        &drop_session_id,
                                        paths,
                                        window,
                                        cx,
                                    );
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
            let group_control =
                self.remote_client.is_local() && self.tab_control_covered(layout, cx);
            let remote_card = self
                .attached
                .get(&session_id)
                .filter(|_| self.remote_client.is_local())
                .filter(|chat| {
                    entry.status == SessionStatus::Running && !chat.terminal.control().desktop()
                });
            let overlay_element = if let Some(remote_card) = remote_card.filter(|_| !group_control)
            {
                Some(
                    crate::terminal::control::cover(
                        div().absolute().inset_0(),
                        format!("take-back-{session_id}"),
                        &[remote_card],
                        crate::terminal::control::GroupKind::Tab,
                    )
                    .into_any_element(),
                )
            } else if secondary_state.secondary
                && remote_card.is_none()
                && !self.dismissed_duplicate_chats.contains(&session_id)
            {
                let focus_label = secondary_state.primary_label.clone();
                let stay_root = cx.entity();
                let stay_session_id = session_id.clone();
                Some(
                    DuplicateSubjectOverlay::new(
                        SharedString::from(format!("duplicate-chat-{session_id}")),
                        DuplicateSubjectKind::Chat,
                        focus_label.is_some(),
                        move |_, cx| {
                            if let Some(label) = focus_label.as_deref() {
                                focus_other_window(label, cx);
                            }
                        },
                        move |_, cx| {
                            stay_root.update(cx, |this, cx| {
                                this.dismiss_duplicate_chat(&stay_session_id, cx)
                            });
                        },
                    )
                    .into_any_element(),
                )
            } else {
                match overlay {
                    PaneOverlayState::Archiving
                        if split_tab_archiving(layout, |session_id| {
                            self.sidebar_archiving_session(session_id, cx)
                        }) =>
                    {
                        None
                    }
                    PaneOverlayState::Archiving => Some(
                        SessionOverlay::transition(
                            format!("archiving-{session_id}"),
                            SessionOverlayKind::Archiving,
                        )
                        .into_any_element(),
                    ),
                    PaneOverlayState::Resuming => Some(
                        if Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell) {
                            SessionOverlay::transition(
                                format!("resuming-{session_id}"),
                                SessionOverlayKind::Resuming,
                            )
                            .label(crate::i18n::text("ui.restarting_terminal"))
                            .into_any_element()
                        } else {
                            SessionOverlay::transition(
                                format!("resuming-{session_id}"),
                                SessionOverlayKind::Resuming,
                            )
                            .into_any_element()
                        },
                    ),
                    PaneOverlayState::Starting => {
                        let overlay = SessionOverlay::transition(
                            format!("starting-{session_id}"),
                            SessionOverlayKind::Starting,
                        );
                        Some(
                            match starting_overlay_label(&entry.agent_runtime, fork_materializing) {
                                Some(label) => overlay.label(label).into_any_element(),
                                None => overlay.into_any_element(),
                            },
                        )
                    }
                    PaneOverlayState::Ended {
                        status,
                        resumable,
                        exit_code,
                    } => {
                        if Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell) {
                            let restart_root = cx.entity();
                            let close_root = restart_root.clone();
                            let restart_id = session_id.clone();
                            let close_id = session_id.clone();
                            let restart_pane = pane_id.clone();
                            let close_pane = pane_id.clone();
                            Some(
                                SessionOverlay::shell_exited(
                                    format!("ended-{session_id}"),
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
                                            this.resume_chat(&restart_pane, &restart_id, window, cx)
                                        });
                                    },
                                    move |window, cx| {
                                        close_root.update(cx, |this, cx| {
                                            this.close_terminal_pane(
                                                &close_pane,
                                                &close_id,
                                                window,
                                                cx,
                                            )
                                        });
                                    },
                                )
                                .actions_visible(
                                    self.front_page_client(cx).supports(&["resume"]),
                                    self.front_page_client(cx).supports(&["close", "layout"]),
                                )
                                .when(cfg!(test), |overlay| {
                                    overlay.primary_debug_selector(format!(
                                        "BUTTON:ended-{session_id}-restart"
                                    ))
                                })
                                .into_any_element(),
                            )
                        } else {
                            let resume_root = cx.entity();
                            let archive_root = resume_root.clone();
                            let resume_id = session_id.clone();
                            let archive_id = session_id.clone();
                            let resume_pane = pane_id.clone();
                            Some(
                                SessionOverlay::ended(
                                    format!("ended-{session_id}"),
                                    entry
                                        .local
                                        .as_ref()
                                        .and_then(|entry| self.claude_paused_text(entry, cx))
                                        .map_or_else(
                                            || ended_subtitle(status, resumable, exit_code),
                                            |account| {
                                                if !resumable || status == SessionStatus::Crashed {
                                                    format!(
                                                        "{} {account}",
                                                        ended_subtitle(
                                                            status, resumable, exit_code
                                                        )
                                                    )
                                                } else {
                                                    account
                                                }
                                            },
                                        ),
                                    move |window, cx| {
                                        resume_root.update(cx, |this, cx| {
                                            this.resume_chat(&resume_pane, &resume_id, window, cx)
                                        });
                                    },
                                    move |window, cx| {
                                        archive_root.update(cx, |this, cx| {
                                            this.archive_chat_sessions(
                                                vec![archive_id.clone()],
                                                window,
                                                cx,
                                            )
                                        });
                                    },
                                )
                                .actions_visible(
                                    self.front_page_client(cx).supports(&["resume"]),
                                    self.front_page_client(cx).supports(&["archive", "layout"]),
                                )
                                .when(cfg!(test), |overlay| {
                                    overlay.primary_debug_selector(format!(
                                        "BUTTON:ended-{session_id}-resume"
                                    ))
                                })
                                .into_any_element(),
                            )
                        }
                    }
                    PaneOverlayState::None => None,
                }
            };
            div()
                .relative()
                .flex_1()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(terminal_background)
                .child(terminal_surface)
                .children(overlay_element)
                .into_any_element()
        } else {
            let new_chat_pane_id = pane_id.clone();
            let new_chat_root = cx.entity();
            let new_terminal_pane_id = pane_id.clone();
            let new_terminal_root = cx.entity();
            let overrides = &self.settings(cx).keymap_overrides;
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .when(cfg!(test), |column| {
                            column.debug_selector(|| "EMPTY_PANE_START".into())
                        })
                        .w(rems(280. / 16.))
                        .max_w_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(rems(12. / 16.))
                        .child(
                            div()
                                .text_size(rems(13. / 16.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::muted())
                                .child(crate::i18n::text("ui.open_in_this_pane")),
                        )
                        .child(
                            div()
                                .w_full()
                                .p(rems(4. / 16.))
                                .flex()
                                .flex_col()
                                .gap(rems(2. / 16.))
                                .bg(theme::panel())
                                .border_1()
                                .border_color(theme::border())
                                .rounded(rems(10. / 16.))
                                .children(
                                    self.front_page_client(cx)
                                        .supports(&["catalog", "startChat", "layout"])
                                        .then(|| {
                                            empty_pane_action_row(
                                                "new-chat",
                                                crate::i18n::text("ui.new_chat"),
                                                "message-square-plus.svg",
                                                overrides,
                                            )
                                            .id(SharedString::from(format!("new-chat-{pane_id}")))
                                            .on_click(
                                                move |_, window, cx| {
                                                    new_chat_root.update(cx, |this, cx| {
                                                        this.open_pane_chat_modal(
                                                            &new_chat_pane_id,
                                                            window,
                                                            cx,
                                                        )
                                                    });
                                                },
                                            )
                                        }),
                                )
                                .children(
                                    self.front_page_client(cx)
                                        .supports(&["startShell", "layout"])
                                        .then(|| {
                                            empty_pane_action_row(
                                                "new-terminal",
                                                crate::i18n::text("ui.new_terminal"),
                                                "terminal.svg",
                                                overrides,
                                            )
                                            .id(SharedString::from(format!(
                                                "new-terminal-{pane_id}"
                                            )))
                                            .on_click(
                                                move |_, window, cx| {
                                                    new_terminal_root.update(cx, |this, cx| {
                                                        if this.create_modal_open() {
                                                            return;
                                                        }
                                                        this.focus_pane(&new_terminal_pane_id, cx);
                                                        this.new_terminal_action(
                                                            &NewTerminal,
                                                            window,
                                                            cx,
                                                        );
                                                    });
                                                },
                                            )
                                        }),
                                ),
                        ),
                )
                .into_any_element()
        };

        // The split menu gates its items on how big this pane was last laid
        // out, and those items are built during this entity's render — so a
        // size that actually changed has to ask for another one. Notifying
        // only on a change is what keeps that from looping every frame.
        let measure_root = cx.entity();
        let measured = PaneKey::new(&layout.id, &pane_id);
        let drop_key = measured.clone();
        let preview = self
            .pane_drop
            .as_ref()
            .and_then(|(key, side)| (key == &drop_key).then_some(*side));
        div()
            .id(SharedString::from(format!("pane-{pane_id}")))
            .relative()
            .size_full()
            .min_w(px(0.))
            .min_h(px(0.))
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, _, cx| {
                        measure_root.update(cx, |this, this_cx| {
                            if this.front_page_pane_bounds().get(&measured) == Some(&bounds.size) {
                                return;
                            }
                            this.front_page_pane_bounds_mut()
                                .insert(measured.clone(), bounds.size);
                            this_cx.notify();
                        });
                    },
                )
                .absolute()
                .inset_0(),
            )
            .children(header)
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .opacity(pane_body_opacity(grouped, focused))
                    .child(body),
            )
            .children(preview.map(|side| {
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _| {
                        window.paint_quad(quad(
                            drop_preview(bounds, side),
                            px(0.),
                            theme::with_alpha(theme::accent(), 0.12),
                            px(1.),
                            theme::accent(),
                            BorderStyle::Solid,
                        ));
                    },
                )
                .absolute()
                .inset_0()
            }))
            .on_drag_move::<PaneDrag>(cx.listener(
                move |this, event: &DragMoveEvent<PaneDrag>, _, cx| {
                    let drag = event.drag(cx);
                    let next = if event.bounds.contains(&event.event.position) {
                        let side = drop_side(event.bounds, event.event.position);
                        (drag.tab_id == drop_key.tab_id
                            && drag.pane_id != drop_key.pane_id
                            && drop_allowed(
                                Some(event.bounds.size),
                                side,
                                this.settings(cx).app_zoom,
                            ))
                        .then(|| (drop_key.clone(), side))
                    } else if this
                        .pane_drop
                        .as_ref()
                        .is_some_and(|(key, _)| key == &drop_key)
                    {
                        None
                    } else {
                        return;
                    };
                    if this.pane_drop != next {
                        this.pane_drop = next;
                        cx.notify();
                    }
                },
            ))
            .on_drop(cx.listener(|this, drag: &PaneDrag, window, cx| {
                this.apply_pane_drop(drag, window, cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    if let Some(session_id) = pane_session_for_focus.as_deref() {
                        this.focus_terminal(&pane_id_for_focus, session_id, window, cx);
                    } else {
                        this.focus_pane(&pane_id_for_focus, cx);
                    }
                }),
            )
            .into_any_element()
    }
}
