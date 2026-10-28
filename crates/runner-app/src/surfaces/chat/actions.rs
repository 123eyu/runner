use super::*;

impl NativeRoot {
    pub(crate) fn stop_chat(
        &mut self,
        session_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let remote = self.front_page_client(cx).is_remote();
        if !remote {
            if self.chat_secondary_state(session_id, cx).secondary {
                return;
            }
            if !self.stopping_sessions.insert(session_id.to_owned()) {
                return;
            }
            self.chat_error = None;
            self.app_store.update(cx, |store, store_cx| {
                store.remove_session_activity(session_id, store_cx)
            });
            if self.route == AppRoute::Chat
                && self.active_focused_session_id().as_deref() == Some(session_id)
            {
                self.chat_focus.focus(window, cx);
            }
            cx.notify();
        }
        let Some(operation) =
            WorkspaceAction::prepare(self, format!("session:{session_id}"), &["stop"], cx)
        else {
            return;
        };
        let core = operation.client.clone();
        let target = session_id.to_owned();
        let stop_target = target.clone();
        let stop = cx.background_spawn(async move {
            core.stop(&stop_target).map_err(|error| error.to_string())
        });
        let origin_epoch = self.remote_client.ui.epoch;
        cx.spawn_in(window, async move |weak, cx| {
            let result = stop.await;
            let _ = weak.update_in(cx, |this, window, cx| {
                let Some(result) =
                    WorkspaceAction::complete(this, operation, result, |()| Default::default(), cx)
                else {
                    return;
                };
                if this.remote_client.ui.epoch != origin_epoch {
                    this.stopping_sessions.remove(&target);
                    this.chat_transitions.remove(&target);
                    cx.notify();
                    return;
                }
                this.stopping_sessions.remove(&target);
                this.chat_transitions.remove(&target);
                this.refresh_sessions(cx);
                this.sync_active_chat_detail(cx);
                match result {
                    Ok(()) => {}
                    Err(error) => this.chat_error = Some(error.into()),
                }
                if this.route == AppRoute::Chat
                    && this.active_focused_session_id().as_deref() == Some(target.as_str())
                {
                    this.chat_focus.focus(window, cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn stop_focused_session(
        &mut self,
        _: &StopFocusedSession,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.front_page_is_chat() {
            return;
        }
        let Some(session_id) = self.active_focused_session_id() else {
            return;
        };
        if self
            .front_page_session(&session_id, cx)
            .is_some_and(|entry| entry.status == SessionStatus::Running)
            && !self.session_lifecycle_disabled(&session_id, cx)
        {
            self.stop_chat(&session_id, window, cx);
        }
    }

    pub(crate) fn resume_focused_session(
        &mut self,
        _: &ResumeFocusedSession,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.front_page_is_chat() {
            return;
        }
        let Some(session_id) = self.active_focused_session_id() else {
            return;
        };
        self.resume_chats(vec![session_id], window, cx);
    }

    pub(crate) fn stop_chats(
        &mut self,
        session_ids: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for session_id in session_ids {
            if self
                .front_page_session(&session_id, cx)
                .is_some_and(|entry| entry.status == SessionStatus::Running)
            {
                self.stop_chat(&session_id, window, cx);
            }
        }
    }

    pub(crate) fn resume_chats(
        &mut self,
        session_ids: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pane_ids = self
            .front_page_layout()
            .map(|layout| {
                layout
                    .root
                    .leaves()
                    .into_iter()
                    .filter_map(|leaf| {
                        leaf.session_id
                            .as_ref()
                            .map(|session_id| (session_id.clone(), leaf.id.clone()))
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        for session_id in session_ids {
            let resumable = self
                .front_page_session(&session_id, cx)
                .is_some_and(|entry| entry.status != SessionStatus::Running)
                && !self.session_lifecycle_disabled(&session_id, cx)
                && (self.remote_client.is_active()
                    || !self.chat_transitions.contains_key(&session_id));
            if resumable {
                if let Some(pane_id) = pane_ids.get(&session_id) {
                    self.resume_chat(pane_id, &session_id, window, cx);
                }
            }
        }
    }

    pub(crate) fn toggle_chat_pin(
        &mut self,
        session_id: String,
        pinned: bool,
        cx: &mut Context<Self>,
    ) {
        let node = if self.remote_client.is_active() {
            let tab = self.remote_client.session_tab(&session_id);
            let Some(tab) = tab else {
                return;
            };
            let Some(node) = self
                .front_page_sidebar()
                .read(cx)
                .source_nodes(cx)
                .into_iter()
                .find(|node| node.id == tab)
            else {
                return;
            };
            Some(node)
        } else {
            None
        };
        let next = !pinned;
        if node.is_none() {
            self.app_store.update(cx, |store, store_cx| {
                store.set_session_pinned(&session_id, next, store_cx)
            });
            if let Some(detail) = self
                .active_chat_detail
                .as_mut()
                .filter(|entry| entry.session_id == session_id)
            {
                detail.pinned = next;
            }
            self.chat_error = None;
            cx.notify();
        }
        let key = node
            .as_ref()
            .map(|node| format!("pin:{}", node.id))
            .unwrap_or_else(|| format!("pin:{session_id}"));
        let Some(operation) = WorkspaceAction::prepare(self, key, &["pin"], cx) else {
            return;
        };
        let core = operation.client.clone();
        let target = session_id.clone();
        let pin = cx.background_spawn(async move {
            core.pin_chat(&target, next, node.as_ref())
                .map_err(|error| error.to_string())
        });
        let origin_epoch = self.remote_client.ui.epoch;
        cx.spawn(async move |weak, cx| {
            let result = pin.await;
            let _ = weak.update(cx, |this, cx| {
                let Some(result) =
                    WorkspaceAction::complete(this, operation, result, |()| Default::default(), cx)
                else {
                    return;
                };
                if this.remote_client.ui.epoch != origin_epoch {
                    cx.notify();
                    return;
                }
                if let Err(error) = result {
                    this.chat_error = Some(error.into());
                    this.refresh_sessions(cx);
                }
                this.sync_active_chat_detail(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn handle_chat_menu_action(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .active_focused_session_id()
            .is_some_and(|session_id| self.chat_secondary_state(&session_id, cx).secondary)
        {
            return;
        }
        let Some(action) = self.chat_menu_actions.get(index).cloned() else {
            return;
        };
        match action {
            ChatMenuAction::TogglePin { session_id, pinned } => {
                self.toggle_chat_pin(session_id, pinned, cx)
            }
            ChatMenuAction::RenameSession {
                session_id,
                current,
            } => self.open_chat_rename(
                ChatRenameTarget::Session {
                    session_id,
                    original: current,
                },
                window,
                cx,
            ),
            ChatMenuAction::RenameTab { tab_id, current } => self.open_chat_rename(
                ChatRenameTarget::Tab {
                    tab_id,
                    original: current,
                },
                window,
                cx,
            ),
            ChatMenuAction::Archive(session_ids) => {
                self.archive_chat_sessions(session_ids, window, cx)
            }
            ChatMenuAction::ArchiveAll(session_ids) => {
                self.request_archive_all(None, session_ids, ArchiveAllSource::Chat, window, cx)
            }
        }
    }

    pub(super) fn open_chat_rename(
        &mut self,
        target: ChatRenameTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let original = match &target {
            ChatRenameTarget::Session { original, .. } | ChatRenameTarget::Tab { original, .. } => {
                original.clone()
            }
        };
        let placeholder = match &target {
            ChatRenameTarget::Session { session_id, .. }
                if self.session_entry(session_id, cx).is_some_and(|entry| {
                    Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
                }) =>
            {
                crate::i18n::message("ui.terminal_name")
            }
            ChatRenameTarget::Session { .. } => crate::i18n::message("ui.chat_name"),
            ChatRenameTarget::Tab { .. } => crate::i18n::message("ui.tab_name"),
        };
        let input = cx.new(|input_cx| {
            runner_app::ui::TextField::new(input_cx.focus_handle(), original, placeholder, false)
                .text_size(theme::text_body())
        });
        input.update(cx, |input, input_cx| input.select_all(input_cx));
        let input_focus = input.read(cx).focus_handle();
        self.chat_rename_modal = Some(ChatRenameModal {
            target,
            input,
            close_focus: cx.focus_handle(),
            cancel_focus: cx.focus_handle(),
            submit_focus: cx.focus_handle(),
            submitting: false,
            error: None,
        });
        input_focus.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn close_chat_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .chat_rename_modal
            .as_ref()
            .is_some_and(|modal| modal.submitting)
        {
            return;
        }
        self.chat_rename_modal = None;
        self.focus_active_terminal(window, cx);
        cx.notify();
    }

    pub(crate) fn submit_chat_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.submit_workspace_rename(false, window, cx);
    }

    pub(crate) fn begin_pane_rename(
        &mut self,
        session_id: String,
        value: String,
        placeholder: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.front_page_client(cx).supports(&["rename"]) {
            return;
        }
        let input = cx.new(|input_cx| {
            runner_app::ui::TextField::new(
                input_cx.focus_handle(),
                value.clone(),
                placeholder,
                false,
            )
            .text_size(theme::text_ui())
        });
        input.update(cx, |field, input_cx| {
            field.set_bare(true, input_cx);
            field.set_right_padding(0., input_cx);
            field.select_all(input_cx);
        });
        let focus = input.read(cx).focus_handle();
        let root = cx.entity();
        self._pane_rename_focus_subscription =
            Some(cx.on_focus_out(&focus, window, move |_, _, window, cx| {
                root.update(cx, |this, cx| this.submit_pane_rename(window, cx));
            }));
        self.pane_rename = Some(PaneRename {
            session_id,
            original: value,
            input,
        });
        focus.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn submit_pane_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.submit_workspace_rename(true, window, cx);
    }

    pub(crate) fn cancel_pane_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pane_rename = None;
        self._pane_rename_focus_subscription = None;
        self.focus_active_terminal(window, cx);
        cx.notify();
    }

    pub(crate) fn on_chat_rename_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "enter"
            && self
                .chat_rename_modal
                .as_ref()
                .is_some_and(|modal| !modal.input.read(cx).is_composing())
        {
            cx.stop_propagation();
            self.submit_chat_rename(window, cx);
        }
    }
}
