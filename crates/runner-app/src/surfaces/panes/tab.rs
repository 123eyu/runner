use super::*;

impl NativeRoot {
    pub(crate) fn render_archived_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(detail) = self.archived_chat_detail.as_ref() else {
            return div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .children(
                    (self.route != AppRoute::Settings)
                        .then(|| self.render_daemon_banner(cx))
                        .flatten(),
                )
                .into_any_element();
        };
        let label = session_label(detail);
        let back_label = if detail.handle.is_some() {
            crate::i18n::text("ui.back_to_role")
        } else {
            crate::i18n::text("ui.back_to_roles")
        };
        let handle = detail.handle.clone();
        let root = cx.entity();
        let panel_root = root.clone();
        let sidebar_toggle = self.render_collapsed_titlebar_spacer();
        let sidebar_divider = sidebar_toggle.is_some().then(|| {
            div()
                .mx_1()
                .h(rems(20. / 16.))
                .w(rems(1. / 16.))
                .flex_none()
                .bg(theme::border())
        });
        let header = div()
            .flex_none()
            .h(rems(WORKSPACE_HEADER_HEIGHT / 16.))
            .pl(px(self.workspace_titlebar_padding(window, cx)))
            .pr_2()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(theme::border())
            .rounded_tl(rems(11. / 16.))
            .when(!self.settings(cx).chat_panel_open, |header| {
                header.rounded_tr(rems(11. / 16.))
            })
            .children(sidebar_toggle)
            .children(sidebar_divider)
            .child(
                svg()
                    .path("terminal.svg")
                    .size(rems(15. / 16.))
                    .flex_none()
                    .text_color(theme::accent()),
            )
            .child(
                div()
                    .min_w(px(0.))
                    .truncate()
                    .text_size(theme::text_body())
                    .font_weight(FontWeight::MEDIUM)
                    .child(label),
            )
            .child(
                Button::new("archived-chat-back", back_label)
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .on_press(move |window, cx| {
                        let handle = handle.clone();
                        root.update(cx, |this, root_cx| {
                            this.archived_chat_detail = None;
                            if let Some(handle) = handle {
                                this.open_role_detail(handle, window, root_cx);
                            } else {
                                this.open_roles(window, root_cx);
                            }
                        });
                    }),
            )
            .child(div().min_w(px(0.)).flex_1())
            .when(!self.settings(cx).chat_panel_open, |header| {
                header.child(
                    IconButton::new("open-archived-chat-panel", "panel-right-hidden.svg")
                        .tooltip(crate::i18n::text("ui.open_side_panel"))
                        .on_press(move |_, cx| {
                            panel_root.update(cx, |this, cx| {
                                this.update_app_settings(cx, true, |settings| {
                                    settings.chat_panel_open = true;
                                    true
                                });
                                cx.notify();
                            });
                        }),
                )
            });
        let chat_column = div()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .flex()
            .flex_col()
            .child(self.render_titlebar_drag_area("archived-chat-titlebar-drag", header, cx))
            .children((self.route != AppRoute::Settings).then(|| self.render_daemon_banner(cx)).flatten())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .max_w(rems(448. / 16.))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .rounded(rems(4. / 16.))
                            .border_1()
                            .border_color(theme::border())
                            .bg(theme::raised())
                            .px_6()
                            .py_5()
                            .text_center()
                            .child(
                                div()
                                    .text_size(theme::text_body())
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(crate::i18n::text("ui.session_ended_terminal_closed")),
                            )
                            .child(
                                div()
                                    .text_size(theme::text_ui())
                                    .text_color(theme::muted())
                                    .child(crate::i18n::text("ui.this_chat_was_archived_the_pty_is_gone_and_the_workspace_is_read_only")),
                            ),
                    ),
            );
        let panel_open = self.settings(cx).chat_panel_open;
        let (panel_visibility, panel_animating) = self.chat_panel_visibility.animate_to(
            if panel_open { 1. } else { 0. },
            Instant::now(),
            Duration::from_millis(CHAT_PANEL_TRANSITION_MS),
        );
        if panel_animating {
            window.request_animation_frame();
        }
        self.reset_chat_panel_prompt(
            self.archived_chat_detail
                .as_ref()
                .map(|chat| chat.session_id.clone()),
        );
        let side_panel = self.render_chat_side_panel(
            self.archived_chat_detail.as_ref(),
            self.archived_session_key_copy.clone(),
            panel_visibility,
            panel_open || panel_animating,
            panel_open && !panel_animating,
            cx,
        );
        div()
            .track_focus(&self.chat_focus)
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .flex()
            .child(chat_column)
            .child(side_panel)
            .into_any_element()
    }

    pub(crate) fn render_active_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.remote_client.is_local() {
            self.ensure_active_chat_detail(cx);
        }
        let error_banner = self
            .chat_error
            .clone()
            .filter(|_| self.remote_client.is_local())
            .filter(|_| !self.app_store.read(cx).daemon_disconnected)
            .map(|error| {
                runner_app::ui::notice_banner(error.resolve(), runner_app::ui::Tone::Danger)
                    .when(cfg!(test), |banner| {
                        banner.debug_selector(|| "CHAT_ERROR_BANNER".into())
                    })
            });
        let warning_root = cx.entity();
        let warning_banner = self
            .chat_warning
            .clone()
            .filter(|_| self.remote_client.is_local())
            .filter(|_| !self.app_store.read(cx).daemon_disconnected)
            .map(|warning| {
                runner_app::ui::notice_banner(warning, runner_app::ui::Tone::Warning).child(
                    Button::new("dismiss-chat-warning", crate::i18n::text("ui.dismiss"))
                        .size(ButtonSize::Sm)
                        .variant(runner_app::ui::ButtonVariant::Ghost)
                        .on_press(move |_, cx| {
                            warning_root.update(cx, |this, cx| {
                                this.chat_warning = None;
                                cx.notify();
                            });
                        }),
                )
            });

        let Some(layout) = self.front_page_layout().cloned() else {
            return div()
                .track_focus(&self.chat_focus)
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .children(
                    (self.route != AppRoute::Settings)
                        .then(|| self.render_daemon_banner(cx))
                        .flatten(),
                )
                .children(error_banner)
                .children(warning_banner)
                .child(render_start_surface(
                    &self.settings(cx).keymap_overrides,
                    &self.front_page_client(cx),
                ))
                .into_any_element();
        };
        if self.remote_client.is_local()
            && layout.all_session_ids().iter().any(|id| {
                self.app_store
                    .read(cx)
                    .bridge
                    .session(id)
                    .is_some_and(|terminal| !terminal.control().desktop())
            })
        {
            if let Err(error) = self.ensure_owned_active_tab_attached(window, cx) {
                self.chat_error = Some(error.to_string().into());
            }
        }
        if self.remote_client.is_local() && !layout.drawer_open() {
            for id in layout.drawer_shells() {
                if let Some(chat) = self
                    .attached
                    .get(id)
                    .filter(|chat| !chat.terminal.control().desktop())
                {
                    chat.terminal_interaction
                        .update(cx, |interaction, _| interaction.pane_size = None);
                }
            }
        }
        let group_control = self.remote_client.is_local() && self.tab_control_covered(&layout, cx);
        let show_start = start_surface_visible(Some(&layout));
        let grouped = pane_identity_visible(layout.root.leaves().len());
        let focused_shell = layout
            .focused_session_id()
            .and_then(|id| self.front_page_session(id, cx))
            .is_some_and(|entry| {
                Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
            });
        if self.remote_client.is_local() {
            self.prune_pane_state(&layout);
        }
        let pane_tree = if show_start {
            render_start_surface(
                &self.settings(cx).keymap_overrides,
                &self.front_page_client(cx),
            )
            .into_any_element()
        } else {
            self.render_pane_node(&layout.root, &layout, cx)
        };
        let tab_archiving =
            split_tab_archiving(&layout, |id| self.sidebar_archiving_session(id, cx));
        let header_layout = layout.clone();
        let header = self.cached_region(
            "tab-bar",
            gpui::StyleRefinement::default()
                .w_full()
                .h(rems(WORKSPACE_HEADER_HEIGHT / 16.))
                .flex_none(),
            move |root, window, cx| root.render_chat_tab_header(&header_layout, window, cx),
            cx,
        );

        let drawer = layout
            .drawer_open()
            .then(|| self.render_terminal_drawer(&layout, window, cx));
        let drawer_height = layout.drawer_height();
        let drawer_drag = DrawerResizeDrag;
        let drawer_resizing = self.front_page_drawer_resizing();
        let zoom = self.settings(cx).app_zoom;
        let tab_body = div()
            .id("chat-tab-body")
            .relative()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .child(pane_tree),
            )
            .children(drawer.map(|drawer| {
                div()
                    .flex_none()
                    .h(rems(drawer_height / 16.))
                    .min_h(rems(MIN_DRAWER_HEIGHT / 16.))
                    .max_h(rems(MAX_DRAWER_HEIGHT / 16.))
                    .flex()
                    .flex_col()
                    .child(
                        resize_strip(
                            "terminal-drawer-resize",
                            ResizeAxis::Rows,
                            drawer_resizing,
                            zoom,
                            Some(theme::border_strong()),
                            true,
                        )
                        .id("terminal-drawer-resize")
                        .on_drag(
                            drawer_drag,
                            |drag: &DrawerResizeDrag, _, _, cx: &mut App| cx.new(|_| drag.clone()),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.))
                            .flex()
                            .flex_col()
                            .pb(rems(8. / 16.))
                            .child(drawer),
                    )
            }))
            .on_drag_move::<DrawerResizeDrag>(cx.listener(
                |this, event: &DragMoveEvent<DrawerResizeDrag>, _, cx| {
                    this.resize_terminal_drawer(event, cx);
                },
            ))
            .on_drop(cx.listener(|this, _: &DrawerResizeDrag, _, cx| {
                this.finish_terminal_drawer_resize(cx);
            }))
            .on_drop(cx.listener(|this, drag: &PaneDrag, window, cx| {
                this.apply_pane_drop(drag, window, cx);
            }))
            .when(tab_archiving, |body| {
                body.child(SessionOverlay::transition(
                    format!("archiving-tab-{}", layout.id),
                    SessionOverlayKind::Archiving,
                ))
            });
        let tab_body = if group_control {
            crate::terminal::control::cover(
                tab_body,
                "take-back-tab",
                &self
                    .control_tab_session_ids(&layout, cx)
                    .iter()
                    .filter_map(|id| self.attached.get(id))
                    .collect::<Vec<_>>(),
                crate::terminal::control::GroupKind::Tab,
            )
        } else {
            tab_body
        };
        let chat_column = div()
            .relative()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .flex()
            .flex_col()
            .children((!show_start).then(|| {
                div()
                    .flex_none()
                    .debug_selector(|| "CHAT_TAB_HEADER".into())
                    .child(header)
            }))
            .children(
                (self.route != AppRoute::Settings)
                    .then(|| self.render_daemon_banner(cx))
                    .flatten(),
            )
            .children(error_banner)
            .children(warning_banner)
            .child(tab_body);
        let panel_open = self.remote_client.is_local()
            && !show_start
            && side_panel_open(self.settings(cx).chat_panel_open, focused_shell);
        let (panel_visibility, panel_animating) = self.chat_panel_visibility.animate_to(
            if panel_open { 1. } else { 0. },
            Instant::now(),
            Duration::from_millis(CHAT_PANEL_TRANSITION_MS),
        );
        if panel_animating {
            window.request_animation_frame();
        }
        self.reset_chat_panel_prompt(
            self.active_chat_detail
                .as_ref()
                .map(|chat| chat.session_id.clone()),
        );
        let side_panel = self.cached_region(
            "side-panel",
            gpui::StyleRefinement::default()
                .w(rems(
                    self.settings(cx).chat_panel_width * panel_visibility / 16.,
                ))
                .h_full()
                .flex_none(),
            move |root, _, cx| {
                root.render_chat_side_panel(
                    root.active_chat_detail.as_ref(),
                    root.session_key_copy.clone(),
                    panel_visibility,
                    panel_open || panel_animating,
                    panel_open && !panel_animating,
                    cx,
                )
            },
            cx,
        );
        // The panel clips its own content, so its splitter sits out here and
        // centres on the border the panel draws.
        let panel_resize = (panel_open || panel_animating).then(|| {
            let zoom = self.settings(cx).app_zoom;
            let divider = self.settings(cx).chat_panel_width * panel_visibility * zoom - 0.5;
            resize_strip(
                "chat-panel-resize",
                ResizeAxis::Columns,
                self.chat_panel_resizing,
                zoom,
                None,
                false,
            )
            .id("chat-panel-resize")
            .map(|handle| {
                #[cfg(any(windows, target_os = "linux"))]
                let handle = handle.occlude();
                handle
            })
            .absolute()
            .top_0()
            .right(px(divider - resize_strip_inset(zoom)))
            .on_drag(
                ChatPanelResizeDrag,
                |drag: &ChatPanelResizeDrag, _, _, cx: &mut App| cx.new(|_| drag.clone()),
            )
        });

        div()
            .relative()
            .when(grouped, |surface| surface.key_context("ChatSplit"))
            .track_focus(&self.chat_focus)
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .flex()
            .child(chat_column)
            .child(side_panel)
            .children(panel_resize)
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.finish_split_resize(cx);
                    this.finish_terminal_drawer_resize(cx);
                    this.finish_chat_panel_resize(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.finish_split_resize(cx);
                    this.finish_terminal_drawer_resize(cx);
                    this.finish_chat_panel_resize(cx);
                }),
            )
            .on_drag_move::<ChatPanelResizeDrag>(cx.listener(
                |this, event: &DragMoveEvent<ChatPanelResizeDrag>, _, cx| {
                    let width = f32::from(event.bounds.right() - event.event.position.x)
                        / this.settings(cx).app_zoom;
                    let width = app_settings::clamp_chat_panel_width(width);
                    if !this.chat_panel_resizing {
                        this.chat_panel_resizing = true;
                        cx.notify();
                    }
                    this.update_app_settings(cx, false, |settings| {
                        if settings.chat_panel_width == width {
                            return false;
                        }
                        settings.chat_panel_width = width;
                        true
                    });
                },
            ))
            .on_drop(cx.listener(|this, _: &ChatPanelResizeDrag, _, cx| {
                this.finish_chat_panel_resize(cx);
                this.save_settings(cx);
            }))
            .into_any_element()
    }

    pub(super) fn configure_chat_action_menu(
        &mut self,
        layout: &PaneLayout,
        lifecycle_busy: bool,
        cx: &mut Context<Self>,
    ) {
        let session_ids = layout.session_ids();
        let all_session_ids = layout.all_session_ids();
        let chat_session_ids = session_ids
            .iter()
            .filter(|session_id| {
                self.front_page_session(session_id, cx)
                    .is_some_and(|entry| {
                        !Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        let grouped = pane_identity_visible(layout.root.leaves().len());
        let mut actions = Vec::new();
        let mut items = Vec::new();
        if grouped && !session_ids.is_empty() {
            items.push(
                UiMenuItem::new(crate::i18n::message("ui.rename_group"))
                    .icon("square-pen.svg")
                    .disabled(lifecycle_busy),
            );
            actions.push(ChatMenuAction::RenameTab {
                tab_id: layout.id.clone(),
                current: layout.name.clone().unwrap_or_default(),
            });
            if !chat_session_ids.is_empty() {
                items.push(
                    UiMenuItem::new(crate::i18n::message("ui.archive"))
                        .icon("archive.svg")
                        .destructive(true)
                        .disabled(lifecycle_busy),
                );
                actions.push(ChatMenuAction::ArchiveAll(all_session_ids));
            }
        } else if let Some(session_id) = session_ids.first() {
            if let Some(entry) = self.front_page_session(session_id, cx) {
                let current = entry.title.clone().unwrap_or_default();
                items.push(
                    UiMenuItem::new(if entry.pinned {
                        crate::i18n::message("ui.unpin")
                    } else {
                        crate::i18n::message("ui.pin")
                    })
                    .icon(if entry.pinned {
                        "pin-off.svg"
                    } else {
                        "pin.svg"
                    })
                    .disabled(lifecycle_busy),
                );
                actions.push(ChatMenuAction::TogglePin {
                    session_id: session_id.clone(),
                    pinned: entry.pinned,
                });
                items.push(
                    UiMenuItem::new(crate::i18n::message("ui.rename"))
                        .icon("square-pen.svg")
                        .disabled(lifecycle_busy),
                );
                actions.push(ChatMenuAction::RenameSession {
                    session_id: session_id.clone(),
                    current,
                });
                if !Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell) {
                    let archive_all = !layout.drawer_shells().is_empty();
                    items.push(
                        UiMenuItem::new(crate::i18n::message("ui.archive"))
                            .icon("archive.svg")
                            .destructive(true)
                            .disabled(lifecycle_busy),
                    );
                    actions.push(if archive_all {
                        ChatMenuAction::ArchiveAll(all_session_ids)
                    } else {
                        ChatMenuAction::Archive(vec![session_id.clone()])
                    });
                }
            }
        }
        if self.remote_client.is_active() {
            let pairs = items
                .into_iter()
                .zip(actions)
                .filter(|(_, action)| {
                    let features: &[&str] = match action {
                        ChatMenuAction::TogglePin { .. } => &["pin"],
                        ChatMenuAction::RenameSession { .. } | ChatMenuAction::RenameTab { .. } => {
                            &["rename"]
                        }
                        ChatMenuAction::Archive(_) => &["archive", "layout"],
                        ChatMenuAction::ArchiveAll(ids) => {
                            if ids.iter().any(|id| {
                                self.front_page_session(id, cx)
                                    .is_some_and(|entry| entry.agent_runtime == "shell")
                            }) {
                                &["archive", "layout", "close"]
                            } else {
                                &["archive", "layout"]
                            }
                        }
                    };
                    self.front_page_client(cx).supports(features)
                })
                .collect::<Vec<_>>();
            (items, actions) = pairs.into_iter().unzip();
        }
        self.chat_menu_actions = actions;
        self.chat_action_menu
            .update(cx, |menu, menu_cx| menu.set_items(items, menu_cx));
    }

    pub(super) fn render_topbar_session_control(
        &self,
        layout: &PaneLayout,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let session_ids = layout.session_ids();
        if session_ids.is_empty() {
            return None;
        }
        let feature = if session_ids.iter().any(|id| {
            self.front_page_session(id, cx)
                .is_some_and(|session| session.status == SessionStatus::Running)
        }) {
            "stop"
        } else {
            "resume"
        };
        if !self.front_page_client(cx).supports(&[feature]) {
            return None;
        }
        let grouped = layout.root.leaves().len() > 1;
        let lifecycle_busy = session_ids
            .iter()
            .any(|session_id| self.session_lifecycle_disabled(session_id, cx));
        let any_running = session_ids.iter().any(|session_id| {
            self.front_page_session(session_id, cx)
                .is_some_and(|entry| entry.status == SessionStatus::Running)
        });
        let any_resuming = session_ids.iter().any(|session_id| {
            self.chat_transitions
                .get(session_id)
                .is_some_and(|transition| transition.kind == TransitionKind::Resuming)
        });
        let root = cx.entity();
        if grouped {
            if any_running {
                let targets = session_ids;
                Some(
                    SessionControl::new("stop-all-chats", SessionControlKind::Stop)
                        .variant(SessionControlVariant::Header)
                        .label(crate::i18n::text("ui.stop_all"))
                        .title(crate::i18n::text("ui.stop_all_sessions"))
                        .lifecycle_disabled(lifecycle_busy)
                        .on_press(move |window, cx| {
                            root.update(cx, |this, cx| {
                                this.stop_chats(targets.clone(), window, cx)
                            });
                        })
                        .into_any_element(),
                )
            } else if any_resuming {
                Some(
                    SessionControl::new("resume-all-chats", SessionControlKind::Resuming)
                        .variant(SessionControlVariant::Header)
                        .into_any_element(),
                )
            } else {
                let targets = session_ids;
                Some(
                    SessionControl::new("resume-all-chats", SessionControlKind::Resume)
                        .variant(SessionControlVariant::Header)
                        .label(crate::i18n::text("ui.resume_all"))
                        .title(crate::i18n::text("ui.resume_all_sessions"))
                        .lifecycle_disabled(lifecycle_busy)
                        .on_press(move |window, cx| {
                            root.update(cx, |this, cx| {
                                this.resume_chats(targets.clone(), window, cx)
                            });
                        })
                        .into_any_element(),
                )
            }
        } else {
            let session_id = session_ids[0].clone();
            let status = self
                .front_page_session(&session_id, cx)
                .map(|entry| entry.status)?;
            let shell = self
                .front_page_session(&session_id, cx)
                .is_some_and(|entry| {
                    Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
                });
            if any_resuming {
                Some(
                    SessionControl::new(
                        "resume-chat-header",
                        if shell {
                            SessionControlKind::Resuming
                        } else {
                            SessionControlKind::Resume
                        },
                    )
                    .variant(SessionControlVariant::Header)
                    .title(crate::i18n::text("ui.resuming"))
                    .lifecycle_disabled(true)
                    .into_any_element(),
                )
            } else if status == SessionStatus::Running {
                Some(
                    SessionControl::new("stop-chat-header", SessionControlKind::Stop)
                        .variant(SessionControlVariant::Header)
                        .title(if shell {
                            crate::i18n::text("ui.stop_terminal")
                        } else {
                            crate::i18n::text("ui.stop_chat")
                        })
                        .lifecycle_disabled(lifecycle_busy)
                        .on_press(move |window, cx| {
                            root.update(cx, |this, cx| this.stop_chat(&session_id, window, cx));
                        })
                        .into_any_element(),
                )
            } else {
                let pane_id = layout.focused_pane_id.clone();
                Some(
                    SessionControl::new("resume-chat-header", SessionControlKind::Resume)
                        .variant(SessionControlVariant::Header)
                        .title(if shell {
                            crate::i18n::text("ui.restart_terminal")
                        } else {
                            crate::i18n::text("ui.resume_chat")
                        })
                        .lifecycle_disabled(lifecycle_busy)
                        .on_press(move |window, cx| {
                            root.update(cx, |this, cx| {
                                this.resume_chat(&pane_id, &session_id, window, cx)
                            });
                        })
                        .into_any_element(),
                )
            }
        }
    }
}
