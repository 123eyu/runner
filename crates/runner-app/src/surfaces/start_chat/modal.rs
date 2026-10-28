use super::*;

impl NativeRoot {
    pub(crate) fn open_new_tab_modal(
        &mut self,
        _: &NewTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.remote_client.is_active() {
            let target = self
                .front_page_layout()
                .and_then(|layout| {
                    layout
                        .root
                        .leaves()
                        .into_iter()
                        .find(|leaf| leaf.id == layout.focused_pane_id && leaf.session_id.is_none())
                        .map(|leaf| ChatTarget::Pane {
                            tab_id: layout.id.clone(),
                            pane_id: leaf.id.clone(),
                        })
                })
                .unwrap_or(ChatTarget::NewTab);
            let project = match &target {
                ChatTarget::Pane { .. } => self
                    .front_page_layout()
                    .and_then(|layout| layout.parent_id.clone()),
                ChatTarget::NewTab => self.active_project_id(cx),
            };
            self.open_remote_start_chat(target, ProjectScope::or_root(project), window, cx);
            return;
        }

        if self.create_modal_open() {
            return;
        }
        let focused_empty_pane = self.focused_empty_chat_pane();
        if let Some(pane_id) = focused_empty_pane {
            self.open_pane_chat_modal(&pane_id, window, cx);
            return;
        }
        let active_project_id = self.active_project_id(cx);
        let project = active_project_id
            .as_deref()
            .and_then(|id| {
                self.app_store
                    .read(cx)
                    .projects
                    .iter()
                    .find(|project| project.id == id)
            })
            .cloned();
        self.open_start_chat_modal(ChatTarget::NewTab, None, project, window, cx);
    }

    pub(crate) fn open_sidebar_chat_modal(
        &mut self,
        scope: ProjectScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.remote_client.is_active() {
            self.open_remote_start_chat(ChatTarget::NewTab, scope, window, cx);
            return;
        }

        if self.create_modal_open() {
            return;
        }
        let project = scope
            .project_id()
            .and_then(|id| {
                self.app_store
                    .read(cx)
                    .projects
                    .iter()
                    .find(|project| project.id == id)
            })
            .cloned();
        self.open_start_chat_modal(ChatTarget::NewTab, None, project, window, cx);
    }

    pub(crate) fn open_pane_chat_modal(
        &mut self,
        pane_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.remote_client.is_active() {
            if let Some(layout) = self.front_page_layout() {
                let target = ChatTarget::Pane {
                    tab_id: layout.id.clone(),
                    pane_id: pane_id.into(),
                };
                let scope = ProjectScope::or_root(layout.parent_id.clone());
                self.open_remote_start_chat(target, scope, window, cx);
            }
            return;
        }

        let Some(tab_id) = self.tabs.active_tab_id().map(str::to_owned) else {
            return;
        };
        let tab_project_id = self.active_tab_project_id(cx);
        let project = tab_project_id
            .as_deref()
            .and_then(|id| {
                self.app_store
                    .read(cx)
                    .projects
                    .iter()
                    .find(|project| project.id == id)
            })
            .cloned();
        self.open_start_chat_modal(
            ChatTarget::Pane {
                tab_id,
                pane_id: pane_id.to_owned(),
            },
            self.last_focused_role_id.clone(),
            project,
            window,
            cx,
        );
    }

    pub(super) fn open_remote_start_chat(
        &mut self,
        target: ChatTarget,
        scope: ProjectScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_remote_start_chat_with_role(target, scope, None, window, cx);
    }

    pub(crate) fn open_remote_role_chat(
        &mut self,
        role_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_remote_start_chat_with_role(
            ChatTarget::NewTab,
            ProjectScope::Root,
            Some(role_id),
            window,
            cx,
        );
    }

    pub(super) fn open_remote_start_chat_with_role(
        &mut self,
        target: ChatTarget,
        scope: ProjectScope,
        default_role_id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.create_modal_open() {
            return;
        }
        self.remote_client.open_remote_start_chat_with_role(
            target,
            scope,
            default_role_id,
            window,
            cx,
        );
    }

    pub(super) fn open_start_chat_modal(
        &mut self,
        target: ChatTarget,
        default_role_id: Option<String>,
        project: Option<runner_core::protocol::project::ProjectRow>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut error = None;
        match self.core(cx).role_list() {
            Ok(roles) => self
                .app_store
                .update(cx, |store, store_cx| store.replace_roles(roles, store_cx)),
            Err(load_error) => error = Some(load_error.to_string()),
        }

        let (runtimes, agents_checking, agents_error) =
            load_selectable_runtimes(self.core(cx), self.settings(cx));
        let runtimes = runtimes
            .into_iter()
            .map(Into::into)
            .collect::<Vec<RuntimeChoice>>();
        let recent_runtimes = read_recent_runtimes(&self.app_store.read(cx).app_data_dir);
        let roles = self
            .app_store
            .read(cx)
            .roles
            .clone()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<RoleChoice>>();
        let scope = ProjectScope::or_root(project.map(|project| project.id));
        let default_runtime = self.settings(cx).default_runtime.clone();
        let default_working_dir = self.settings(cx).default_working_dir.clone();
        let front_page = self.front_page_client(cx);
        self.build_start_chat_modal(
            target,
            default_role_id,
            roles,
            runtimes,
            recent_runtimes,
            scope,
            default_runtime,
            default_working_dir,
            agents_checking,
            agents_error,
            error,
            front_page,
            None,
            window,
            cx,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::surfaces) fn build_start_chat_modal(
        &mut self,
        target: ChatTarget,
        default_role_id: Option<String>,
        roles: Vec<RoleChoice>,
        runtimes: Vec<RuntimeChoice>,
        recent_runtimes: Vec<String>,
        scope: ProjectScope,
        default_runtime: String,
        default_working_dir: String,
        agents_checking: bool,
        agents_error: Option<String>,
        mut error: Option<String>,
        front_page: crate::surfaces::front_page::FrontPageClient,
        host_catalog: Option<runner_core::companion::Catalog>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mode = ChatMode::Runtime;
        let role_id = default_role_id
            .filter(|role_id| roles.iter().any(|role| role.id == *role_id))
            .or_else(|| roles.first().map(|role| role.id.clone()));
        let runtime_name = initial_direct_runtime(&runtimes, &recent_runtimes, &default_runtime);
        let title = match mode {
            ChatMode::Role => role_id
                .as_deref()
                .and_then(|role_id| roles.iter().find(|role| role.id == role_id))
                .map(|role| default_title_for_role(&role.handle))
                .unwrap_or_default(),
            ChatMode::Runtime => runtime_name
                .as_deref()
                .and_then(|name| runtimes.iter().find(|runtime| runtime.name.key() == name))
                .map(|runtime| default_title_for_runtime(&runtime.display_name))
                .unwrap_or_default(),
        };
        let project = ProjectField::new(
            "start-chat-project",
            match &host_catalog {
                Some(catalog) => ProjectChoice::host(catalog),
                None => ProjectChoice::local(&self.app_store.read(cx).projects),
            },
            host_catalog.is_none(),
            scope.project_id().map(str::to_owned),
            matches!(target, ChatTarget::Pane { .. }),
            selection_handler(&cx.entity(), StartChatSelection::Project),
            cx,
        );
        let project_cwd = project
            .chosen()
            .map(|project| project.path.clone())
            .unwrap_or_default();
        let cwd_placeholder = if front_page.is_remote() {
            crate::i18n::text("ui.blank_starts_in_your_default_directory").to_owned()
        } else {
            cwd_placeholder(
                mode,
                role_id
                    .as_deref()
                    .and_then(|role_id| roles.iter().find(|role| role.id == role_id)),
                &default_working_dir,
            )
        };
        let title_input = cx.new(|input_cx| {
            TextField::new(input_cx.focus_handle(), "", title, false).text_size(theme::text_body())
        });
        let cwd_input = cx.new(|input_cx| {
            if front_page.is_remote() {
                TextField::new(input_cx.focus_handle(), project_cwd, cwd_placeholder, true)
                    .text_size(theme::text_ui())
            } else {
                working_dir_text_field(input_cx.focus_handle(), project_cwd, cwd_placeholder)
                    .text_size(theme::text_ui())
            }
        });
        let model_input = cx.new(|input_cx| {
            TextField::new(
                input_cx.focus_handle(),
                "",
                crate::i18n::message("word.default"),
                false,
            )
            .placeholder_as_value(true)
        });
        let root = cx.entity();
        let role_handler = selection_handler(&root, StartChatSelection::Role);
        let selected_role = role_id
            .as_deref()
            .and_then(|id| roles.iter().find(|role| role.id == id));
        let role_select = cx.new(|select_cx| {
            StyledSelect::new(
                "start-chat-role",
                select_cx.focus_handle(),
                role_id.clone().unwrap_or_default(),
                role_options(&roles),
                role_handler,
                select_cx,
            )
            .picker(true)
            .placeholder(crate::i18n::message("ui.no_roles_yet"))
            .disabled(roles.is_empty())
        });
        let role_runtime_options = role_runtime_options(&runtimes, selected_role);
        let role_runtime_handler = selection_handler(&root, StartChatSelection::RoleRuntime);
        let role_runtime_select = cx.new(|select_cx| {
            StyledSelect::new(
                "start-chat-role-runtime",
                select_cx.focus_handle(),
                selected_role
                    .map(|role| role.runtime.clone())
                    .unwrap_or_default(),
                role_runtime_options,
                role_runtime_handler,
                select_cx,
            )
            .full_width(true)
            .min_menu_width(px(200.))
        });
        let runtime_handler = selection_handler(&root, StartChatSelection::Runtime);
        let runtime_select = cx.new(|select_cx| {
            StyledSelect::new(
                "start-chat-runtime",
                select_cx.focus_handle(),
                runtime_name.clone().unwrap_or_default(),
                runtime_options(direct_runtime_order(&runtimes, &recent_runtimes)),
                runtime_handler,
                select_cx,
            )
            .picker(true)
            .disabled(runtimes.is_empty())
        });
        let effort_handler = selection_handler(&root, StartChatSelection::Effort);
        let effort_select = cx.new(|select_cx| {
            StyledSelect::new(
                "start-chat-effort",
                select_cx.focus_handle(),
                "",
                Vec::new(),
                effort_handler,
                select_cx,
            )
            .full_width(true)
            .min_menu_width(px(200.))
        });
        let speed_handler = selection_handler(&root, StartChatSelection::Speed);
        let speed_select = cx.new(|select_cx| {
            StyledSelect::new(
                "start-chat-speed",
                select_cx.focus_handle(),
                "inherit",
                speed_options(None, false),
                speed_handler,
                select_cx,
            )
            .full_width(true)
            .min_menu_width(px(160.))
        });
        let model_field = cx.new(|model_cx| ModelField::new(model_input.clone(), &[], model_cx));
        let scroll_handle = ScrollHandle::new();
        let scroll_owner = cx.entity_id();
        let scrollbar = cx.new(|_| Scrollbar::app(scroll_handle.clone(), scroll_owner));
        let role_mode_focus = cx.focus_handle();
        let direct_mode_focus = cx.focus_handle();
        let reset_focus = std::array::from_fn(|_| cx.focus_handle());
        let browse_focus = cx.focus_handle();
        let close_focus = cx.focus_handle();
        let cancel_focus = cx.focus_handle();
        let submit_focus = cx.focus_handle();
        // The model's dot, note and reset live outside the field, so the form
        // redraws with it.
        let model_subscription = cx.observe(&model_input, |this, _, cx| {
            if let Some(modal) = this.start_chat_modal.as_mut() {
                sync_model_marker(modal, cx);
                sync_effort_control(modal, cx);
            }
            cx.notify();
        });

        self.sidebar_preview_open = false;
        self.start_chat_modal = Some(StartChatModal {
            mission_role: None,
            front_page,
            folder_picker: None,
            remote_started: None,
            remote_renamed: false,
            host_account: host_catalog.as_ref().and_then(|catalog| {
                catalog
                    .accounts
                    .iter()
                    .find(|account| account.id == catalog.active_account)
                    .map(|account| account.name.clone())
            }),
            _folder_subscription: None,
            _folder_choice: None,
            _cwd_blur: None,
            _cwd_follow: None,
            target,
            project,
            mode,
            roles,
            runtimes,
            recent_runtimes,
            role_id,
            runtime_name,
            role_runtime_override: None,
            effort: String::new(),
            speed: "inherit".into(),
            title: title_input,
            cwd: cwd_input,
            model: model_input,
            model_field,
            role_select,
            role_runtime_select,
            runtime_select,
            effort_select,
            speed_select,
            scroll_handle,
            scrollbar,
            role_mode_focus,
            direct_mode_focus,
            reset_focus,
            browse_focus,
            close_focus,
            cancel_focus,
            submit_focus,
            agents_checking,
            agents_error,
            submitting: false,
            error: error.take().map(Into::into),
            _model_subscription: model_subscription,
        });
        if let Some(catalog) = host_catalog {
            let client = self.front_page_client(cx);
            let remote = self.remote_client.computer().unwrap();
            let computer = remote.name.clone();
            let platform = remote.info.platform.clone();
            let input = self.start_chat_modal.as_ref().unwrap().cwd.clone();
            let picker = cx.new(|picker_cx| {
                crate::surfaces::front_page::folder_picker::FolderPicker::new(
                    client,
                    computer,
                    platform,
                    input.clone(),
                    crate::surfaces::front_page::folder_picker::choices(&catalog),
                    picker_cx,
                )
            });
            let subscription = cx.observe(&picker, |_, _, cx| cx.notify());
            let choice = cx.subscribe(
                &picker,
                |this, _, _: &crate::surfaces::front_page::folder_picker::FolderChosen, cx| {
                    if let Some(modal) = this.start_chat_modal.as_mut() {
                        let directory = modal.cwd.read(cx).text().to_owned();
                        modal.project.picked(&directory, cx);
                        cx.notify();
                    }
                },
            );
            let blur_picker = picker.clone();
            let blur = cx.on_focus_out(
                &input.read(cx).focus_handle(),
                window,
                move |_, _, _, cx| blur_picker.update(cx, |picker, cx| picker.validate(cx)),
            );
            let modal = self.start_chat_modal.as_mut().unwrap();
            modal.folder_picker = Some(picker);
            modal._folder_subscription = Some(subscription);
            modal._folder_choice = Some(choice);
            modal._cwd_blur = Some(blur);
        }
        let cwd_focus = self
            .start_chat_modal
            .as_ref()
            .unwrap()
            .cwd
            .read(cx)
            .focus_handle();
        let follow = cx.on_blur(&cwd_focus, window, |_, window, cx| {
            // Focus callbacks run during drawing, when notifications cannot schedule a frame.
            cx.defer_in(window, |this, _, cx| this.follow_start_chat_cwd(cx));
        });
        if let Some(modal) = self.start_chat_modal.as_mut() {
            modal._cwd_follow = Some(follow);
            restore_baseline(modal, cx);
        }
        self.start_chat_modal
            .as_ref()
            .unwrap()
            .picker_focus(cx)
            .focus(window, cx);
        self.refresh_start_chat_models(cx);
        cx.notify();
    }

    pub(super) fn refresh_start_chat_models(&self, cx: &Context<Self>) {
        if self.remote_client.is_active() {
            return;
        }
        if let Some(runtime) = self
            .start_chat_modal
            .as_ref()
            .and_then(|modal| modal.active_runtime())
            .filter(|runtime| self.settings(cx).model_runtimes().contains(&runtime.name))
        {
            let _ = self.core(cx).runtime_refresh_models(&[runtime.name]);
        }
    }

    pub(crate) fn refresh_start_chat_runtimes(&mut self, cx: &mut Context<Self>) {
        if self.remote_client.is_active() {
            return;
        }
        let (runtimes, agents_checking, agents_error) =
            load_selectable_runtimes(self.core(cx), self.settings(cx));
        let runtimes = runtimes
            .into_iter()
            .map(Into::into)
            .collect::<Vec<RuntimeChoice>>();
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        let catalog_loaded = agents_error.is_none();
        let previous_runtime = modal.runtime_name.clone();
        let previous_override = modal.role_runtime_override.clone();
        modal.agents_checking = agents_checking;
        modal.agents_error = agents_error;

        if catalog_loaded {
            // A control the user has not changed follows the refreshed defaults.
            let effort_untouched = modal.effort == modal.baseline_effort();
            modal.runtimes = runtimes;
            if modal.role_runtime_override.as_ref().is_some_and(|name| {
                !modal
                    .runtimes
                    .iter()
                    .any(|runtime| runtime.name.key() == name.as_str())
            }) {
                modal.role_runtime_override = None;
            }
            if modal.runtime_name.as_ref().is_none_or(|name| {
                !modal
                    .runtimes
                    .iter()
                    .any(|runtime| runtime.name.key() == name.as_str())
            }) {
                modal.runtime_name = direct_runtime_order(&modal.runtimes, &modal.recent_runtimes)
                    .first()
                    .map(|runtime| runtime.name.to_string());
            }
            if modal.mode == ChatMode::Runtime && modal.runtime_name != previous_runtime {
                let derived = modal
                    .selected_runtime()
                    .map(|runtime| default_title_for_runtime(&runtime.display_name))
                    .unwrap_or_default();
                update_auto_title(&modal.title, derived, cx);
            }
            modal.runtime_select.update(cx, |select, select_cx| {
                select.set_options(
                    runtime_options(direct_runtime_order(
                        &modal.runtimes,
                        &modal.recent_runtimes,
                    )),
                    select_cx,
                );
                select.set_value(modal.runtime_name.clone().unwrap_or_default(), select_cx);
            });
            sync_role_runtime_select(modal, cx);
            if modal.runtime_name != previous_runtime
                || modal.role_runtime_override != previous_override
            {
                restore_baseline(modal, cx);
            } else {
                if effort_untouched {
                    modal.effort = modal.baseline_effort();
                }
                sync_runtime_controls(modal, cx);
            }
        }
        cx.notify();
    }

    pub(crate) fn sync_start_chat_default_runtime(&mut self, cx: &mut Context<Self>) {
        if self.remote_client.is_active() {
            return;
        }
        let default_runtime = self.settings(cx).default_runtime.clone();
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        let runtime_name =
            initial_direct_runtime(&modal.runtimes, &modal.recent_runtimes, &default_runtime);
        if modal.runtime_name == runtime_name {
            return;
        }
        modal.runtime_name = runtime_name;
        if modal.mode == ChatMode::Runtime {
            let derived = modal
                .selected_runtime()
                .map(|runtime| default_title_for_runtime(&runtime.display_name))
                .unwrap_or_default();
            update_auto_title(&modal.title, derived, cx);
        }
        modal.runtime_select.update(cx, |select, select_cx| {
            select.set_value(modal.runtime_name.clone().unwrap_or_default(), select_cx)
        });
        restore_baseline(modal, cx);
        cx.notify();
    }

    pub(crate) fn remember_active_role(&mut self, cx: &App) {
        let Some(session_id) = self.active_focused_session_id() else {
            return;
        };
        self.last_focused_role_id = self
            .session_entry(&session_id, cx)
            .and_then(|entry| entry.role_id.clone());
    }

    pub(super) fn close_start_chat_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .start_chat_modal
            .as_ref()
            .is_some_and(|modal| modal.submitting)
        {
            return;
        }
        self.start_chat_modal = None;
        self.focus_active_terminal(window, cx);
        cx.notify();
    }

    pub(super) fn set_start_chat_mode(&mut self, mode: ChatMode, cx: &mut Context<Self>) {
        let default_working_dir = self.settings(cx).default_working_dir.clone();
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        if modal.mission_role.is_some() || modal.mode == mode || modal.submitting {
            return;
        }
        modal.mode = mode;
        clear_role_overrides(modal, cx);
        let derived = match mode {
            ChatMode::Role => modal
                .selected_role()
                .map(|role| default_title_for_role(&role.handle))
                .unwrap_or_default(),
            ChatMode::Runtime => modal
                .selected_runtime()
                .map(|runtime| default_title_for_runtime(&runtime.display_name))
                .unwrap_or_default(),
        };
        update_auto_title(&modal.title, derived, cx);
        let placeholder = if modal.front_page.is_remote() {
            crate::i18n::text("ui.blank_starts_in_your_default_directory").to_owned()
        } else {
            cwd_placeholder(mode, modal.selected_role(), &default_working_dir)
        };
        modal.cwd.update(cx, |input, input_cx| {
            input.set_placeholder(placeholder, input_cx)
        });
        self.refresh_start_chat_models(cx);
        cx.notify();
    }

    pub(super) fn select_start_chat_choice(
        &mut self,
        selection: StartChatSelection,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        let refresh_models = matches!(
            selection,
            StartChatSelection::Role
                | StartChatSelection::RoleRuntime
                | StartChatSelection::Runtime
        );
        let default_working_dir = self.settings(cx).default_working_dir.clone();
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        match selection {
            StartChatSelection::Project => {
                let directory = modal.cwd.read(cx).text().to_owned();
                if let Some(directory) = modal.project.choose(value, &directory) {
                    modal
                        .cwd
                        .update(cx, |input, input_cx| input.reset(directory, input_cx));
                }
            }
            StartChatSelection::Role => {
                modal.role_id = Some(value.to_owned());
                let derived = modal
                    .selected_role()
                    .map(|role| default_title_for_role(&role.handle))
                    .unwrap_or_default();
                update_auto_title(&modal.title, derived, cx);
                let placeholder = if modal.front_page.is_remote() {
                    crate::i18n::text("ui.blank_starts_in_your_default_directory").to_owned()
                } else {
                    cwd_placeholder(modal.mode, modal.selected_role(), &default_working_dir)
                };
                modal.cwd.update(cx, |input, input_cx| {
                    input.set_placeholder(placeholder, input_cx)
                });
                // Overrides are on top of one role's setup: another role
                // starts from its own.
                clear_role_overrides(modal, cx);
            }
            StartChatSelection::RoleRuntime => {
                let own = modal
                    .selected_role()
                    .is_some_and(|role| role.runtime == value);
                let next = (!own && !value.is_empty()).then(|| value.to_owned());
                // This runs inside the Runtime select's own choose, which holds
                // the select: it already shows the pick, so it is not synced.
                if next != modal.role_runtime_override {
                    modal.role_runtime_override = next;
                    restore_baseline(modal, cx);
                }
            }
            StartChatSelection::Runtime => {
                modal.runtime_name = Some(value.to_owned());
                let derived = modal
                    .selected_runtime()
                    .map(|runtime| default_title_for_runtime(&runtime.display_name))
                    .unwrap_or_default();
                update_auto_title(&modal.title, derived, cx);
                restore_baseline(modal, cx);
            }
            StartChatSelection::Effort => modal.effort = value.to_owned(),
            StartChatSelection::Speed => modal.speed = value.to_owned(),
        }
        if selection == StartChatSelection::Role {
            self.suggest_add_role_handle(cx);
        }
        if refresh_models {
            self.refresh_start_chat_models(cx);
        }
        cx.notify();
    }

    /// The working directory was committed: let it choose its project.
    pub(super) fn follow_start_chat_cwd(&mut self, cx: &mut Context<Self>) {
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        let directory = modal.cwd.read(cx).text().to_owned();
        modal.project.follow(&directory, cx);
        cx.notify();
    }

    pub(super) fn browse_start_chat_cwd(&mut self, cx: &mut Context<Self>) {
        if self.remote_client.is_active() {
            return;
        }

        if self
            .start_chat_modal
            .as_ref()
            .is_some_and(|modal| modal.submitting)
        {
            return;
        }
        let Some(cwd_input) = self
            .start_chat_modal
            .as_ref()
            .map(|modal| modal.cwd.clone())
        else {
            return;
        };
        let selected = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(crate::i18n::text("ui.pick_a_working_directory").into()),
        });
        cx.spawn(async move |weak, cx| {
            let result = selected
                .await
                .map_err(|error| error.to_string())
                .and_then(|result| result.map_err(|error| error.to_string()));
            let _ = weak.update(cx, |this, cx| {
                let Some(modal) = this.start_chat_modal.as_mut() else {
                    return;
                };
                if modal.cwd != cwd_input {
                    return;
                }
                match result {
                    Ok(Some(paths)) => {
                        if let Some(path) = paths.into_iter().next() {
                            modal.cwd.update(cx, |input, input_cx| {
                                input.reset(path.to_string_lossy().into_owned(), input_cx)
                            });
                            let directory = modal.cwd.read(cx).text().to_owned();
                            modal.project.picked(&directory, cx);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => modal.error = Some(error.into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn on_start_chat_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                self.close_start_chat_modal(window, cx);
            }
            "enter"
                if self.start_chat_modal.as_ref().is_some_and(|modal| {
                    !modal.is_composing(cx)
                        && (modal.mission_role.is_none() || event.keystroke.modifiers.platform)
                }) =>
            {
                cx.stop_propagation();
                self.submit_start_chat(window, cx);
            }
            _ => {}
        }
    }
}
