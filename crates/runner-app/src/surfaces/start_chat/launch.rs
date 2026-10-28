use super::*;

impl NativeRoot {
    pub(super) fn submit_start_chat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::surfaces::front_page::client::{ChatLaunch, ChatStart};
        let Some(modal) = self.start_chat_modal.as_ref() else {
            return;
        };
        if modal.mission_role.is_some() {
            self.submit_mission_role(window, cx);
            return;
        }
        if !modal.can_submit() || modal.is_composing(cx) {
            return;
        }
        let client = modal.front_page.clone();
        let remote = client.is_remote();
        if remote && (client.target != self.front_page_client(cx).target || !client.valid()) {
            return;
        }
        let target = modal.target.clone();
        let title = user_chat_title(modal.title.read(cx));
        let mut required = vec!["startChat"];
        if title.is_some() {
            required.push("rename");
        }
        let placement = match &target {
            ChatTarget::NewTab => None,
            ChatTarget::Pane { tab_id, pane_id } => {
                required.push("layout");
                let layout = if remote {
                    self.front_page_layout()
                } else {
                    self.tabs.active()
                };
                let Some(layout) = layout.filter(|layout| {
                    &layout.id == tab_id
                        && (!remote || layout.root.leaves().iter().any(|leaf| &leaf.id == pane_id))
                }) else {
                    if !remote {
                        self.start_chat_modal.as_mut().unwrap().error = Some(crate::i18n::message(
                            "ui.the_target_tab_is_no_longer_active",
                        ));
                        cx.notify();
                    }
                    return;
                };
                if remote && self.remote_client.layout_pending(tab_id) {
                    return;
                }
                Some((layout.clone(), pane_id.clone()))
            }
        };
        if !client.supports(&required) {
            return;
        }
        let mut size = if let Some((layout, pane)) = &placement {
            self.estimated_terminal_size(layout, pane, window, cx)
        } else if remote {
            self.estimated_terminal_size(&PaneLayout::single(None, &[]), "p1", window, cx)
        } else {
            (INITIAL_COLS, INITIAL_ROWS)
        };
        if remote {
            size = (size.0.max(20), size.1.max(8));
        }
        let cwd = if remote {
            (!modal.cwd.read(cx).text().is_empty()).then(|| modal.cwd.read(cx).text().to_owned())
        } else {
            effective_working_dir(
                modal.cwd.read(cx).text(),
                modal.mode == ChatMode::Role
                    && modal
                        .selected_role()
                        .and_then(|role| role.working_dir.as_deref())
                        .is_some_and(|path| !path.trim().is_empty()),
                &self.settings(cx).default_working_dir,
            )
        };
        let request = build_start_request(
            modal.mode,
            modal
                .selected_role()
                .map(|role| (role.id.as_str(), role.runtime.as_str())),
            modal.selected_runtime().map(|runtime| runtime.name.key()),
            modal.role_runtime_override.as_deref(),
            modal.model_override(cx),
            modal.effort_override(),
            modal.speed_override(),
            cwd,
        )
        .expect("validated start chat selection");
        let (role, mut runtime, model, effort, speed, cwd) = match request {
            StartRequest::Role {
                role_id,
                runtime,
                model,
                effort,
                speed,
                cwd,
            } => (Some(role_id), runtime, model, effort, speed, cwd),
            StartRequest::Runtime {
                runtime,
                model,
                effort,
                speed,
                cwd,
            } => (None, Some(runtime), model, effort, speed, cwd),
        };
        if remote && modal.mode == ChatMode::Role {
            runtime = modal.role_runtime_override.clone();
        }
        let recent_runtime = (!remote && modal.mode == ChatMode::Runtime)
            .then(|| runtime.clone())
            .flatten();
        let mut launch = ChatLaunch {
            input: ChatStart {
                role,
                runtime,
                model,
                effort,
                speed: speed.map(|speed| {
                    if speed == CodexSpeed::Fast {
                        "fast".into()
                    } else {
                        "standard".into()
                    }
                }),
                project: modal.scope.project_id().map(str::to_owned),
                cwd,
                size,
            },
            title,
            placement,
            started: if remote {
                modal.remote_started.clone()
            } else {
                None
            },
            renamed: remote && modal.remote_renamed,
        };
        let form_id = modal.cwd.clone();
        let remote_context = if remote {
            self.remote_client
                .prepare_chat_launch(client.clone(), launch.placement.as_ref())
        } else {
            None
        };
        let modal = self.start_chat_modal.as_mut().unwrap();
        modal.submitting = true;
        modal.error = None;
        set_start_chat_controls_disabled(modal, true, cx);
        let operation = move || {
            let result = client.launch_chat(&mut launch);
            (launch, result)
        };
        if let Some(action) = remote_context {
            let work = cx.background_spawn(async move { operation() });
            cx.spawn_in(window, async move |weak, cx| {
                let (launch, result) = work.await;
                let _ = weak.update_in(cx, |root, _, cx| {
                    if !root.remote_client.settle_chat_launch(&action) {
                        return;
                    }
                    let Some(modal) = root
                        .start_chat_modal
                        .as_mut()
                        .filter(|modal| modal.cwd == form_id)
                    else {
                        cx.notify();
                        return;
                    };
                    modal.remote_started = launch.started;
                    modal.remote_renamed = launch.renamed;
                    match result {
                        Ok(output) => {
                            root.start_chat_modal = None;
                            root.remote_client.complete_chat_launch(
                                action,
                                output.session,
                                output.layout,
                                cx,
                            );
                            root.sync_remote_state(cx);
                        }
                        Err(error) => {
                            modal.submitting = false;
                            modal.error = Some(error.to_string().into());
                            set_start_chat_controls_disabled(
                                modal,
                                modal.remote_started.is_some(),
                                cx,
                            );
                            let title_saved =
                                modal.remote_started.is_some() && modal.remote_renamed;
                            modal
                                .title
                                .update(cx, |field, cx| field.set_disabled(title_saved, cx));
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
        } else {
            let (launch, result) = operation();
            if launch.started.is_some() {
                if let Some(runtime) = recent_runtime {
                    let _ = record_recent_runtime(&self.app_store.read(cx).app_data_dir, &runtime);
                }
            }
            self.finish_local_chat_launch(launch.started, result, window, cx);
        }
        cx.notify();
    }

    pub(super) fn finish_local_chat_launch(
        &mut self,
        spawned_id: Option<String>,
        result: runner_core::protocol::ClientResult<
            crate::surfaces::front_page::client::ChatLaunched,
        >,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut rename_error = None;
        let result = result.map_err(anyhow::Error::from).and_then(|output| {
            rename_error = output.rename_error.map(|error| {
                crate::i18n::messagef(
                    "format.chat_started_but_its_title_could_not_be_saved",
                    &[("error", error)],
                )
            });
            self.refresh_sessions(cx);
            self.reload_tabs(cx)?;
            self.tabs.activate_session(&output.session);
            self.sync_active_project_from_active_tab(cx);
            self.set_route(AppRoute::Chat, cx);
            self.ensure_active_tab_attached(window, cx)?;
            Ok(output.session)
        });
        match result {
            Ok(session_id) => {
                self.start_chat_modal = None;
                self.error = rename_error;
                self.remember_active_role(cx);
                self.mark_active_tab_viewed(window, cx);
                self.sync_active_chat_detail(cx);
                self.begin_chat_transition(
                    &session_id,
                    chat_lifecycle::TransitionKind::Starting,
                    Some(0),
                    window,
                    cx,
                );
            }
            Err(start_error) => {
                if let Some(session_id) = spawned_id {
                    self.start_chat_modal = None;
                    let _ = self.reload_tabs(cx);
                    self.tabs.activate_session(&session_id);
                    self.sync_active_project_from_active_tab(cx);
                    self.set_route(AppRoute::Chat, cx);
                    let _ = self.ensure_active_tab_attached(window, cx);
                    self.remember_active_role(cx);
                    self.mark_active_tab_viewed(window, cx);
                    self.sync_active_chat_detail(cx);
                    self.begin_chat_transition(
                        &session_id,
                        chat_lifecycle::TransitionKind::Starting,
                        Some(0),
                        window,
                        cx,
                    );
                    self.error = Some(start_error.to_string().into());
                } else if let Some(modal) = self.start_chat_modal.as_mut() {
                    modal.submitting = false;
                    modal.error = Some(start_error.to_string().into());
                    set_start_chat_controls_disabled(modal, false, cx);
                }
            }
        }
        cx.notify();
    }
}

/// The launch request. A role chat with nothing overridden sends no override;
/// a model or effort changed on the role's own agent sends that agent with the
/// changed values, which the backend resolves by keeping the role's others;
/// another agent goes with its own defaults.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_start_request(
    mode: ChatMode,
    role: Option<(&str, &str)>,
    runtime_name: Option<&str>,
    role_runtime_override: Option<&str>,
    model: Option<String>,
    effort: Option<String>,
    speed: Option<CodexSpeed>,
    cwd: Option<String>,
) -> Option<StartRequest> {
    match mode {
        ChatMode::Role => role.map(|(role_id, role_runtime)| StartRequest::Role {
            role_id: role_id.to_owned(),
            runtime: role_runtime_override
                .or((model.is_some() || effort.is_some()).then_some(role_runtime))
                .map(str::to_owned),
            model,
            effort,
            speed,
            cwd,
        }),
        ChatMode::Runtime => runtime_name.map(|runtime| StartRequest::Runtime {
            runtime: runtime.to_owned(),
            model,
            effort,
            speed,
            cwd,
        }),
    }
}
