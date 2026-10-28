use super::*;

impl NativeRoot {
    pub(super) fn render_chat_tab_header(
        &mut self,
        layout: &PaneLayout,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        #[cfg(test)]
        crate::render_counts::count_render("tab-bar", cx);
        let session_ids = layout.session_ids();
        let grouped = pane_identity_visible(layout.root.leaves().len());
        let focused_session_id = layout.focused_session_id().map(str::to_owned);
        let focused_entry = focused_session_id
            .as_deref()
            .and_then(|session_id| self.front_page_session(session_id, cx));
        let focused_shell = focused_entry.as_ref().is_some_and(|entry| {
            Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
        });
        let terminal_tab = self.active_tab_is_terminal(cx);
        let focused_secondary = self.remote_client.is_local()
            && focused_session_id
                .as_deref()
                .is_some_and(|session_id| self.cached_chat_secondary_state(session_id).secondary);
        let label = self.tab_label(layout, cx);
        let lifecycle_busy = session_ids
            .iter()
            .filter(|session_id| {
                self.front_page_session(session_id, cx)
                    .is_some_and(|entry| {
                        !Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
                    })
            })
            .any(|session_id| self.session_lifecycle_disabled(session_id, cx));
        self.configure_chat_action_menu(layout, lifecycle_busy, cx);
        let sidebar_toggle = self.render_collapsed_titlebar_spacer();
        let root = cx.entity();
        let fork_root = root.clone();
        let drawer_root = root.clone();
        let panel_root = root.clone();
        let control = (!focused_secondary)
            .then(|| self.render_topbar_session_control(layout, window, cx))
            .flatten();
        let fork_pending = focused_session_id.as_deref().is_some_and(|session_id| {
            self.fork_confirm
                .as_ref()
                .is_some_and(|confirm| confirm.pending && confirm.session_id == session_id)
                || super::super::chat::fork_in_progress(&self.forking_sessions, session_id)
        });
        let fork_action = focused_session_id.clone().and_then(|session_id| {
            let (disabled, tooltip) = match if self.remote_client.is_active() {
                if self.front_page_client(cx).supports(&["catalog", "fork"])
                    && focused_entry.as_ref().is_some_and(|e| e.native_fork)
                {
                    HeaderForkState::Enabled
                } else {
                    HeaderForkState::Hidden
                }
            } else {
                header_fork_state(
                    focused_entry
                        .as_ref()
                        .and_then(|entry| entry.local.as_ref()),
                    focused_secondary,
                )
            } {
                HeaderForkState::Enabled if fork_pending => (true, None),
                HeaderForkState::Enabled => (
                    false,
                    Some(crate::i18n::text("ui.fork_chat_into_a_new_tab")),
                ),
                HeaderForkState::Disabled(_) if fork_pending => (true, None),
                HeaderForkState::Disabled(caption) => (true, caption),
                HeaderForkState::Hidden => return None,
            };
            let mut button = IconButton::new("fork-chat", "git-fork.svg")
                .disabled(disabled)
                .on_press(move |window, cx| {
                    fork_root.update(cx, |this, cx| this.fork_chat(&session_id, window, cx));
                });
            if let Some(tooltip) = tooltip {
                button = button.tooltip(tooltip);
            }
            Some(button.into_any_element())
        });
        let focused_program_status = focused_entry
            .as_ref()
            .and_then(|entry| self.front_page_status(&entry.session_id, cx))
            .is_some_and(|status| status.program_status);
        let single_status = (!grouped && (!focused_shell || focused_program_status))
            .then_some(focused_entry.as_ref())
            .flatten()
            .map(|entry| {
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
                let target = layout.focused_pane_id.clone();
                let width = self
                    .front_page_pane_bounds()
                    .get(&PaneKey::new(&layout.id, &target))
                    .map_or(480., |size| {
                        f32::from(size.width) / (f32::from(window.rem_size()) / 16.)
                    });
                div()
                    .id("single-pane-status")
                    .flex_none()
                    .child(runner_app::ui::agent_status::header_status_indicator(
                        status.clone(),
                        status.shows_label(width),
                        status.shows_detail(width),
                        "tab-status",
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
            });
        let account = (!grouped)
            .then(|| {
                focused_entry.as_ref().and_then(|entry| {
                    entry
                        .local
                        .as_ref()
                        .and_then(|entry| self.claude_account_details(entry, cx))
                })
            })
            .flatten()
            .map(|(name, _, _)| account_chip(name).into_any_element());
        let title_actions = account
            .into_iter()
            .chain(
                (!session_ids.is_empty() && !focused_secondary)
                    .then(|| self.chat_action_menu.clone().into_any_element()),
            )
            .chain(control)
            .chain(fork_action)
            .chain(single_status);
        let keymap_overrides = self.settings(cx).keymap_overrides.clone();
        // A single-pane tab has no identity line, so the header carries its
        // split menu; once split, every identity line carries its own.
        let split_action = (!grouped
            && self.front_page_client(cx).supports(if terminal_tab {
                &["layout", "startShell"]
            } else {
                &["layout"]
            }))
        .then(|| {
            let tab_id = layout.id.clone();
            let pane_id = layout.focused_pane_id.clone();
            self.split_menu(SplitMenuSurface::Header, &tab_id, &pane_id, cx)
                .into_any_element()
        });
        let drawer_action = (!terminal_tab
            && (!layout.drawer_shells().is_empty()
                || self
                    .front_page_client(cx)
                    .supports(&["startShell", "layout"])))
        .then(|| {
            let open = layout.drawer_open();
            let tooltip = terminal_drawer_tooltip(open, &keymap_overrides);
            IconButton::new(
                "terminal-drawer-toggle",
                if open {
                    "panel-bottom-open.svg"
                } else {
                    "panel-bottom-hidden.svg"
                },
            )
            .tooltip(tooltip)
            .on_press(move |window, cx| {
                drawer_root.update(cx, |this, cx| this.toggle_terminal_drawer(window, cx));
            })
            .into_any_element()
        });
        let panel_action = (self.remote_client.is_local()
            && !side_panel_open(self.settings(cx).chat_panel_open, focused_shell)
            && !focused_shell)
            .then(|| {
                IconButton::new("open-chat-panel", "panel-right-hidden.svg")
                    .tooltip(crate::i18n::text("ui.open_side_panel"))
                    .on_press(move |_, cx| {
                        panel_root.update(cx, |this, cx| {
                            this.update_app_settings(cx, true, |settings| {
                                settings.chat_panel_open = true;
                                true
                            });
                            cx.notify();
                        });
                    })
                    .into_any_element()
            });
        let header_icon = workspace_header_icon(
            grouped,
            focused_session_id.as_deref(),
            focused_entry
                .as_ref()
                .map(|entry| entry.agent_runtime.as_str()),
        );
        let header_live = focused_entry
            .as_ref()
            .is_some_and(|entry| entry.status == SessionStatus::Running);
        let header = WorkspaceHeader::new(
            px(self.workspace_titlebar_padding(window, cx)),
            header_icon.render(
                rems(15. / 16.),
                header_icon.color(theme::accent(), header_live),
                header_live,
            ),
            label,
        )
        .sidebar_toggle(sidebar_toggle)
        .title_actions(title_actions)
        .trailing_actions(
            split_action
                .into_iter()
                .chain(drawer_action)
                .chain(panel_action),
        )
        .into_div()
        .w_full()
        .rounded_tl(rems(11. / 16.))
        .when(
            !side_panel_open(self.settings(cx).chat_panel_open, focused_shell),
            |header| header.rounded_tr(rems(11. / 16.)),
        )
        .when(cfg!(test), |header| {
            header.debug_selector(|| "CHAT_TAB_HEADER_CONTENT".into())
        })
        .map(|element| {
            #[cfg(test)]
            let element = crate::theme_snapshot::record_fill("CHAT_TAB_HEADER_CONTENT", element);
            element
        });
        self.render_titlebar_drag_area("chat-titlebar-drag", header, cx)
            .into_any_element()
    }

    pub(super) fn claude_account_details(
        &self,
        detail: &DirectSessionEntry,
        cx: &App,
    ) -> Option<(String, Option<String>, String)> {
        let store = self.app_store.read(cx);
        if detail.agent_runtime != "claude-code" || !store.claude_accounts.multiple() {
            return None;
        }
        let session = detail.claude_account.as_ref();
        let account = store.claude_accounts.session_account(session);
        let name = account
            .map(|account| account.name.clone())
            .or_else(|| session.map(|session| session.folder.clone()))?;
        let email = account
            .and_then(|account| store.claude_accounts.statuses.get(&account.id))
            .and_then(|status| status.email.clone());
        let folder = session
            .map(|session| session.folder.clone())
            .or_else(|| account.map(|account| account.folder.clone()))?;
        Some((name, email, folder))
    }

    pub(super) fn claude_paused_text(
        &self,
        detail: &DirectSessionEntry,
        cx: &App,
    ) -> Option<String> {
        if detail
            .claude_account
            .as_ref()
            .is_some_and(|account| account.pinned)
        {
            return None;
        }
        let (previous, _, folder) = self.claude_account_details(detail, cx)?;
        let store = self.app_store.read(cx);
        let active = store.claude_accounts.active()?;
        let session = detail.claude_account.as_ref();
        if folder == active.folder && session.is_none_or(|session| session.native == active.native)
        {
            return None;
        }
        Some(crate::i18n::tf(
            "accounts.paused",
            &[("previous", previous), ("active", active.name.clone())],
        ))
    }

    pub(super) fn render_chat_side_panel(
        &self,
        detail: Option<&DirectSessionEntry>,
        session_key_copy: Entity<CopyValueButton>,
        visibility: f32,
        show_panel: bool,
        border_on: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        #[cfg(test)]
        crate::render_counts::count_render("side-panel", cx);
        let width = self.settings(cx).chat_panel_width;
        let visible_width = width * visibility;
        if !show_panel {
            return div()
                .id("chat-side-panel")
                .relative()
                .w(rems(visible_width / 16.))
                .h_full()
                .flex_none()
                .overflow_hidden()
                .into_any_element();
        }
        let role = detail
            .and_then(|detail| detail.role_id.as_deref())
            .and_then(|role_id| {
                self.app_store
                    .read(cx)
                    .roles
                    .iter()
                    .find(|role| role.id == role_id)
                    .cloned()
            });
        let collapse_root = cx.entity();
        let header = self.render_titlebar_drag_area(
            "chat-panel-titlebar-drag",
            div()
                .flex_none()
                .h(rems(WORKSPACE_HEADER_HEIGHT / 16.))
                .pl_4()
                .pr_2()
                .flex()
                .items_center()
                .justify_end()
                .border_b_1()
                .border_color(theme::border())
                .rounded_tr(rems(11. / 16.))
                .map(|element| {
                    #[cfg(test)]
                    let element = crate::theme_snapshot::record_fill("CHAT_PANEL_HEADER", element);
                    element
                })
                .child(
                    IconButton::new("collapse-chat-panel", "panel-right-open.svg")
                        .tooltip(crate::i18n::text("ui.collapse_side_panel"))
                        .on_press(move |_, cx| {
                            collapse_root.update(cx, |this, cx| {
                                this.update_app_settings(cx, true, |settings| {
                                    settings.chat_panel_open = false;
                                    true
                                });
                                cx.notify();
                            });
                        }),
                ),
            cx,
        );
        let content = if let Some(detail) = detail {
            let open_root = cx.entity();
            let toggle_root = cx.entity();
            let actions = ChatPanelActions {
                open_role: role.as_ref().map(|role| {
                    let handle = role.handle.clone();
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        open_root.update(cx, |this, cx| {
                            this.open_role_detail(handle.clone(), window, cx)
                        });
                    }) as ClickHandler
                }),
                toggle_prompt: Rc::new(move |_, cx| {
                    toggle_root.update(cx, |this, cx| {
                        this.chat_panel_prompt_expanded = !this.chat_panel_prompt_expanded;
                        cx.notify();
                    });
                }),
                expanded: self.chat_panel_prompt_expanded,
                view: cx.entity_id(),
            };
            chat_panel_content(
                detail,
                role.as_ref(),
                width,
                session_key_copy,
                actions,
                self.claude_account_details(detail, cx),
                cx,
            )
        } else {
            div()
                .text_size(theme::text_ui())
                .text_color(theme::faint())
                .child(crate::i18n::text("ui.loading_chat"))
                .into_any_element()
        };
        let panel = div()
            .relative()
            .w(rems(width / 16.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(header)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .pb(rems(8. / 16.))
                    .child(
                        div()
                            .id("chat-panel-scroll")
                            .flex_1()
                            .min_h(px(0.))
                            .overflow_y_scroll()
                            .p_5()
                            .child(content),
                    ),
            );
        div()
            .id("chat-side-panel")
            .debug_selector(|| "CHAT_SIDE_PANEL".into())
            .relative()
            .w(rems(visible_width / 16.))
            .h_full()
            .flex_none()
            .overflow_hidden()
            .when(border_on, |panel| {
                panel.border_l_1().border_color(theme::border())
            })
            .child(panel)
            .into_any_element()
    }
}
