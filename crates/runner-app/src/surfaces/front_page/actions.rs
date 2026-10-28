use super::FrontPageClient;
use crate::*;
use runner_core::companion::{ActionKind, ActionTarget};
use runner_core::protocol::ClientResult;

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum SessionAction {
    Resume,
    Fork,
}

#[derive(Default)]
pub(crate) struct Completion {
    pub(crate) session: Option<String>,
    pub(crate) reopened_sessions: Vec<String>,
    pub(crate) layout: Option<PaneLayout>,
    pub(crate) layouts: Vec<PaneLayout>,
    pub(crate) activate_tab: Option<String>,
}

pub(crate) struct WorkspaceAction {
    pub(crate) client: FrontPageClient,
    key: String,
    epoch: u64,
    remote: Option<(HashMap<String, PaneLayout>, u64)>,
}

pub(crate) struct ShellStart {
    pub(crate) draft: PaneLayout,
    pub(crate) pane: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) size: (u16, u16),
}

pub(crate) struct ArchiveCompletion {
    pub(crate) removed: Vec<String>,
    pub(crate) error: Option<String>,
    pub(crate) completion: Completion,
}

impl WorkspaceAction {
    pub(crate) fn prepare(
        root: &mut NativeRoot,
        key: String,
        required: &[&str],
        cx: &mut Context<NativeRoot>,
    ) -> Option<WorkspaceAction> {
        let client = root.front_page_client(cx);
        if !client.supports(required) {
            return None;
        }
        let remote = if let Some(remote) = root.remote_window.as_mut() {
            if key.starts_with("layout:") && remote.actions_pending.contains("archive-sessions") {
                return None;
            }
            if !remote.actions_pending.insert(key.clone()) {
                return None;
            }
            remote.action_error = None;
            cx.notify();
            Some((remote.layouts.clone(), remote.navigation_generation))
        } else {
            None
        };
        Some(WorkspaceAction {
            client,
            key,
            epoch: root.client_ui.epoch,
            remote,
        })
    }

    pub(crate) fn complete<T>(
        root: &mut NativeRoot,
        action: WorkspaceAction,
        result: Result<T, String>,
        remote_completion: impl FnOnce(T) -> Completion,
        cx: &mut Context<NativeRoot>,
    ) -> Option<Result<T, String>> {
        let Some((before, navigation_generation)) = action.remote else {
            return Some(result);
        };
        if root.client_ui.epoch != action.epoch
            || root.front_page_client(cx).target != action.client.target
            || !action.client.valid()
        {
            return None;
        }
        let remote = root.remote_window.as_mut()?;
        remote.actions_pending.remove(&action.key);
        match result {
            Ok(done) => root.apply_remote_completion(
                remote_completion(done),
                &before,
                navigation_generation,
                cx,
            ),
            Err(error) => {
                if remote.navigation_generation == navigation_generation {
                    remote.action_error = Some(error);
                }
                if let Some(tab) = action.key.strip_prefix("layout:") {
                    remote.layout_hints.remove(tab);
                    if remote.navigation_generation != navigation_generation {
                        let mut staged = remote.shell_placement.lock().unwrap();
                        if staged.as_ref().is_some_and(|(_, layout)| layout.id == tab) {
                            *staged = None;
                        }
                    }
                }
            }
        }
        root.sync_remote_state(cx);
        cx.notify();
        None
    }

    pub(crate) fn close(
        root: &mut NativeRoot,
        session: &str,
        pane: Option<&str>,
        drawer: bool,
        cx: &mut Context<NativeRoot>,
    ) -> Option<(
        WorkspaceAction,
        impl FnOnce() -> Result<Completion, String> + Send + 'static,
    )> {
        let client = root.front_page_client(cx);
        let (draft, shell) = if client.is_remote() {
            let mut draft = root.front_page_layout()?.clone();
            if root
                .remote_window
                .as_ref()
                .is_some_and(|remote| remote.layout_hints.contains_key(&draft.id))
            {
                return None;
            }
            let shell = root.front_page_session(session, cx)?.agent_runtime == "shell";
            if drawer {
                draft.remove_drawer_shell(session);
            } else if let Some(pane) = pane {
                draft.close_pane(pane);
                draft.remove_session(session);
            } else {
                draft.remove_session(session);
            }
            (Some(draft), shell)
        } else {
            (None, true)
        };
        let mut required = vec![if shell { "close" } else { "archive" }];
        if draft
            .as_ref()
            .is_some_and(|draft| !draft.all_session_ids().is_empty())
        {
            required.push("layout");
        }
        let key = draft
            .as_ref()
            .map(|draft| format!("layout:{}", draft.id))
            .unwrap_or_else(|| format!("session:{session}"));
        let action = WorkspaceAction::prepare(root, key, &required, cx)?;
        let session = session.to_owned();
        let operation = move || {
            if shell {
                client.close(&session)
            } else {
                client.archive(ActionTarget {
                    kind: ActionKind::Session,
                    id: session,
                })
            }
            .map_err(|error| error.to_string())?;
            if let Some(draft) = &draft {
                if !draft.all_session_ids().is_empty() {
                    client
                        .layout(draft, draft.parent_id.clone())
                        .map_err(|error| error.to_string())?;
                }
            }
            Ok(Completion {
                layout: draft,
                ..Default::default()
            })
        };
        Some((action, operation))
    }

    pub(crate) fn shell(
        root: &mut NativeRoot,
        mut input: ShellStart,
        cx: &mut Context<NativeRoot>,
    ) -> Option<(
        WorkspaceAction,
        impl FnOnce() -> Result<Completion, String> + Send + 'static,
    )> {
        let client = root.front_page_client(cx);
        let remote_target = client.is_remote();
        let staged = root
            .remote_window
            .as_ref()
            .map(|remote| remote.shell_placement.clone());
        let pending = staged
            .as_ref()
            .and_then(|staged| staged.lock().unwrap().clone());
        if let Some(remote) = &root.remote_window {
            if remote.actions_pending.contains("archive-sessions")
                || remote
                    .actions_pending
                    .iter()
                    .any(|key| key.starts_with("layout:"))
            {
                return None;
            }
            if let Some((_, draft)) = &pending {
                input.draft = draft.clone();
            }
            if remote.layout_hints.contains_key(&input.draft.id) {
                return None;
            }
            input.project = input.draft.parent_id.clone();
            input.size = (input.size.0.max(20), input.size.1.max(8));
        }
        let required = if remote_target && pending.is_some() {
            vec!["layout"]
        } else if remote_target {
            vec!["startShell", "layout"]
        } else {
            vec!["startShell"]
        };
        let action =
            WorkspaceAction::prepare(root, format!("layout:{}", input.draft.id), &required, cx)?;
        let operation = move || {
            let session = if let Some((session, _)) = pending {
                session
            } else {
                let session = client
                    .start_shell(input.project, input.cwd, input.size)
                    .map_err(|error| error.to_string())?;
                if remote_target {
                    if let Some(pane) = input.pane {
                        input
                            .draft
                            .assign_session(&pane, &session)
                            .map_err(|error| error.to_string())?;
                    } else {
                        input.draft.add_drawer_shell(session.clone());
                    }
                    *staged.as_ref().unwrap().lock().unwrap() =
                        Some((session.clone(), input.draft.clone()));
                }
                session
            };
            let layout = if remote_target {
                client
                    .layout(&input.draft, input.draft.parent_id.clone())
                    .map_err(|error| error.to_string())?;
                Some(input.draft)
            } else {
                None
            };
            Ok(Completion {
                session: Some(session),
                layout,
                ..Default::default()
            })
        };
        Some((action, operation))
    }

    pub(crate) fn archive(
        root: &mut NativeRoot,
        ids: Vec<String>,
        local_plan: Option<Vec<(String, super::client::ArchiveSessionOperation)>>,
        pane_close: Option<PendingPaneClose>,
        cx: &mut Context<NativeRoot>,
    ) -> Option<(
        WorkspaceAction,
        impl FnOnce() -> Result<ArchiveCompletion, String> + Send + 'static,
    )> {
        use super::client::ArchiveSessionOperation;
        let client = root.front_page_client(cx);
        let remote_target = client.is_remote();
        let mut layouts = root
            .remote_window
            .as_ref()
            .map(|remote| {
                remote
                    .layouts
                    .values()
                    .filter(|layout| layout.all_session_ids().iter().any(|id| ids.contains(id)))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(remote) = &root.remote_window {
            if layouts.iter().any(|layout| {
                remote.layout_hints.contains_key(&layout.id)
                    || remote
                        .actions_pending
                        .contains(&format!("layout:{}", layout.id))
            }) {
                return None;
            }
        }
        let plan = if let Some(plan) = local_plan {
            plan
        } else {
            let mut plan = ids
                .iter()
                .filter_map(|id| {
                    root.front_page_session(id, cx).map(|session| {
                        (
                            id.clone(),
                            if session.agent_runtime == "shell" {
                                ArchiveSessionOperation::CloseTerminal
                            } else {
                                ArchiveSessionOperation::ArchiveChat {
                                    running: session.status == SessionStatus::Running,
                                }
                            },
                        )
                    })
                })
                .collect::<Vec<_>>();
            plan.sort_by_key(|(id, operation)| {
                (
                    *operation == ArchiveSessionOperation::CloseTerminal,
                    root.active_focused_session_id().as_ref() == Some(id),
                )
            });
            plan
        };
        if let Some(close) = &pane_close {
            for layout in &mut layouts {
                if layout.id == close.tab_id {
                    layout.close_pane(&close.pane_id);
                }
            }
        }
        let mut required = vec!["archive"];
        if pane_close.is_none()
            || layouts.iter().any(|layout| {
                layout
                    .all_session_ids()
                    .iter()
                    .any(|session| !ids.contains(session))
            })
        {
            required.push("layout");
        }
        if plan
            .iter()
            .any(|(_, operation)| *operation == ArchiveSessionOperation::CloseTerminal)
        {
            required.push("close");
        }
        let key = pane_close
            .as_ref()
            .map(|close| format!("layout:{}", close.tab_id))
            .unwrap_or_else(|| "archive-sessions".into());
        let action = WorkspaceAction::prepare(root, key, &required, cx)?;
        let operation = move || {
            let (removed, error) = client.archive_sessions(plan);
            if remote_target {
                if let Some(error) = &error {
                    return Err(error.clone());
                }
            }
            let mut completed = Vec::new();
            for mut layout in layouts {
                for id in &ids {
                    layout.remove_session(id);
                    layout.remove_drawer_shell(id);
                }
                if !layout.all_session_ids().is_empty() {
                    client
                        .layout(&layout, layout.parent_id.clone())
                        .map_err(|error| error.to_string())?;
                }
                completed.push(layout);
            }
            Ok(ArchiveCompletion {
                removed,
                error,
                completion: Completion {
                    layouts: completed,
                    ..Default::default()
                },
            })
        };
        Some((action, operation))
    }
}

impl NativeRoot {
    pub(crate) fn remote_action(
        &mut self,
        key: String,
        required: &[&str],
        operation: impl FnOnce(FrontPageClient) -> ClientResult<Completion> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.remote_window.is_none() {
            return;
        }
        let Some(action) = WorkspaceAction::prepare(self, key, required, cx) else {
            return;
        };
        let client = action.client.clone();
        let required = required.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let work = cx.background_spawn(async move {
            if !client.supports(&required.iter().map(String::as_str).collect::<Vec<_>>()) {
                return Err("The selected computer changed.".to_owned());
            }
            operation(client).map_err(|error| error.to_string())
        });
        cx.spawn(async move |weak, cx| {
            let result = work.await;
            let _ = weak.update(cx, |root, cx| {
                WorkspaceAction::complete(root, action, result, |done| done, cx);
            });
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn apply_remote_completion(
        &mut self,
        done: Completion,
        before: &HashMap<String, PaneLayout>,
        navigation_generation: u64,
        cx: &mut Context<Self>,
    ) {
        let Some(remote) = self.remote_window.as_mut() else {
            return;
        };
        let mut activate = done.layout.is_none();
        let mut placement = None;
        for mut layout in done.layout.into_iter().chain(done.layouts) {
            {
                let mut staged = remote.shell_placement.lock().unwrap();
                if staged
                    .as_ref()
                    .is_some_and(|(_, pending)| pending.id == layout.id)
                {
                    *staged = None;
                }
            }
            let current = self
                .app_store
                .read(cx)
                .remote_clients
                .read(cx)
                .overviews
                .get(&remote.computer.id)
                .and_then(|overview| overview.tabs.iter().find(|tab| tab.id == layout.id));
            let superseded = match current {
                Some(tab) => {
                    !super::model::matches_layout(&layout, tab)
                        && before
                            .get(&layout.id)
                            .is_none_or(|previous| !super::model::matches_layout(previous, tab))
                }
                None => before.contains_key(&layout.id),
            };
            if !superseded {
                if let Some(session) = &done.session {
                    if let Some(leaf) = layout
                        .root
                        .leaves()
                        .into_iter()
                        .find(|leaf| leaf.session_id.as_ref() == Some(session))
                    {
                        placement = Some((layout.id.clone(), Some(leaf.id.clone())));
                        activate = true;
                    } else if layout.drawer_shells().contains(session) {
                        placement = Some((layout.id.clone(), None));
                        activate = true;
                    }
                }
                if remote.navigation_generation != navigation_generation {
                    if let Some(previous) = remote.layouts.get(&layout.id) {
                        super::model::retain_presentation(&mut layout, previous);
                    }
                }
                remote.layout_hints.insert(layout.id.clone(), layout);
            }
        }
        remote.reopen_sessions.extend(done.reopened_sessions);
        if let Some(session) = done.session {
            remote.reopen_sessions.insert(session.clone());
            if activate && remote.navigation_generation == navigation_generation {
                remote.activate_session = Some(session);
                remote.activate_placement = placement;
            }
        }
        if let Some(tab) = done
            .activate_tab
            .filter(|_| remote.navigation_generation == navigation_generation)
        {
            remote.activate_tab = Some(tab);
        }
    }

    #[cfg(test)]
    pub(crate) fn remote_session_action(
        &mut self,
        session: &str,
        action: SessionAction,
        size: (u16, u16),
        cx: &mut Context<Self>,
    ) {
        let size = (size.0.max(20), size.1.max(8));
        let feature = match action {
            SessionAction::Resume => "resume",
            SessionAction::Fork => "fork",
        };
        let session = session.to_owned();
        self.remote_action(
            format!("session:{session}"),
            &[feature],
            move |client| {
                let next = match action {
                    SessionAction::Resume => Some(client.resume(&session, size)?),
                    SessionAction::Fork => Some(client.fork(&session, None, size)?),
                };
                Ok(Completion {
                    session: next,
                    ..Default::default()
                })
            },
            cx,
        );
    }

    pub(crate) fn save_workspace_layout(
        &mut self,
        draft: PaneLayout,
        activate: bool,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let remote_target = self.front_page_client(cx).is_remote();
        if let Some(remote) = &self.remote_window {
            let key = format!("layout:{}", draft.id);
            if remote.actions_pending.contains("archive-sessions")
                || remote.layout_hints.contains_key(&draft.id)
                || remote.actions_pending.contains(&key)
                || !remote.front_page.supports(&["layout"])
            {
                return Ok(());
            }
        }
        let Some(action) =
            WorkspaceAction::prepare(self, format!("layout:{}", draft.id), &["layout"], cx)
        else {
            return Ok(());
        };
        if let Some(remote) = &mut self.remote_window {
            remote.layout_hints.insert(draft.id.clone(), draft.clone());
        }
        let client = action.client.clone();
        let operation = move || {
            client
                .layout(&draft, draft.parent_id.clone())
                .map_err(|error| error.to_string())?;
            Ok(Completion {
                activate_tab: activate.then(|| draft.id.clone()),
                layout: Some(draft),
                ..Default::default()
            })
        };
        if remote_target {
            let task = cx.background_spawn(async move { operation() });
            cx.spawn(async move |weak, cx| {
                let result = task.await;
                let _ = weak.update(cx, |root, cx| {
                    WorkspaceAction::complete(root, action, result, |done| done, cx);
                });
            })
            .detach();
            cx.notify();
            Ok(())
        } else {
            operation().map(drop).map_err(anyhow::Error::msg)
        }
    }

    pub(crate) fn remote_layout(
        &mut self,
        draft: PaneLayout,
        activate: bool,
        cx: &mut Context<Self>,
    ) {
        if self.remote_window.is_some() {
            let _ = self.save_workspace_layout(draft, activate, cx);
        }
    }

    #[cfg(test)]
    pub(crate) fn remote_close_session(
        &mut self,
        session: &str,
        pane: Option<&str>,
        drawer: bool,
        cx: &mut Context<Self>,
    ) {
        if self.remote_window.is_none() {
            return;
        }
        let Some((action, operation)) = WorkspaceAction::close(self, session, pane, drawer, cx)
        else {
            return;
        };
        let task = cx.background_spawn(async move { operation() });
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |root, cx| {
                WorkspaceAction::complete(root, action, result, |done| done, cx);
            });
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn remote_start_shell(
        &mut self,
        draft: PaneLayout,
        pane: Option<String>,
        size: (u16, u16),
        cx: &mut Context<Self>,
    ) {
        let cwd = draft
            .focused_session_id()
            .map(str::to_owned)
            .or_else(|| draft.session_ids().first().cloned())
            .and_then(|id| self.front_page_session(&id, cx))
            .and_then(|session| session.cwd);
        self.remote_start_shell_in(draft, pane, cwd, size, cx);
    }

    pub(crate) fn launch_workspace_shell(
        &mut self,
        input: ShellStart,
        window: &mut Window,
        cx: &mut Context<Self>,
        finish_local: impl FnOnce(&mut Self, Result<String>, &mut Window, &mut Context<Self>),
    ) {
        let Some((action, operation)) = WorkspaceAction::shell(self, input, cx) else {
            return;
        };
        if action.client.is_remote() {
            let task = cx.background_spawn(async move { operation() });
            cx.spawn(async move |weak, cx| {
                let result = task.await;
                let _ = weak.update(cx, |root, cx| {
                    WorkspaceAction::complete(root, action, result, |done| done, cx);
                });
            })
            .detach();
            cx.notify();
        } else {
            let result = operation()
                .map(|done| done.session.unwrap())
                .map_err(anyhow::Error::msg);
            finish_local(self, result, window, cx);
        }
    }

    pub(crate) fn remote_start_shell_in(
        &mut self,
        draft: PaneLayout,
        pane: Option<String>,
        cwd: Option<String>,
        size: (u16, u16),
        cx: &mut Context<Self>,
    ) {
        if self.remote_window.is_none() {
            return;
        }
        let input = ShellStart {
            project: draft.parent_id.clone(),
            draft,
            pane,
            cwd,
            size,
        };
        let Some((action, operation)) = WorkspaceAction::shell(self, input, cx) else {
            return;
        };
        let task = cx.background_spawn(async move { operation() });
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |root, cx| {
                WorkspaceAction::complete(root, action, result, |done| done, cx);
            });
        })
        .detach();
        cx.notify();
    }
}

impl NativeRoot {
    pub(crate) fn submit_workspace_rename(
        &mut self,
        pane: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (input, target, original) = if pane {
            let Some(rename) = &self.pane_rename else {
                return;
            };
            (
                rename.input.clone(),
                ActionTarget {
                    kind: ActionKind::Session,
                    id: rename.session_id.clone(),
                },
                rename.original.clone(),
            )
        } else {
            let Some(modal) = &self.chat_rename_modal else {
                return;
            };
            if modal.submitting {
                return;
            }
            let (target, original) = match &modal.target {
                ChatRenameTarget::Session {
                    session_id,
                    original,
                } => (
                    ActionTarget {
                        kind: ActionKind::Session,
                        id: session_id.clone(),
                    },
                    original.clone(),
                ),
                ChatRenameTarget::Tab { tab_id, original } => (
                    ActionTarget {
                        kind: ActionKind::Tab,
                        id: tab_id.clone(),
                    },
                    original.clone(),
                ),
            };
            (modal.input.clone(), target, original)
        };
        if input.read(cx).is_composing() {
            return;
        }
        let client = self.front_page_client(cx);
        if !client.supports(&["rename"]) {
            return;
        }
        let remote = client.is_remote();
        let name = input.read(cx).text().trim().to_owned();
        let (unchanged, title) = if pane && !remote {
            match crate::surfaces::chat::pane_rename_change(&original, &name) {
                crate::surfaces::chat::PaneRenameChange::Unchanged => (true, None),
                crate::surfaces::chat::PaneRenameChange::Persist(title) => (false, title),
            }
        } else {
            (!remote && name == original.trim(), Some(name))
        };
        if unchanged && !pane {
            self.close_chat_rename(window, cx);
            return;
        }
        let key = format!("rename:{}", target.id);
        if let Some(remote) = self.remote_window.as_mut() {
            if !remote.actions_pending.insert(key.clone()) {
                return;
            }
        }
        if let Some(modal) = self
            .chat_rename_modal
            .as_mut()
            .filter(|modal| modal.input == input)
        {
            modal.submitting = true;
            modal.error = None;
        }
        let epoch = self.client_ui.epoch;
        let guard = client.clone();
        let operation = move || {
            if unchanged {
                Ok(())
            } else {
                client
                    .rename_title(target, title)
                    .map_err(|error| error.to_string())
            }
        };
        if pane && !remote {
            let result = operation();
            self.finish_workspace_rename(pane, input, remote.then_some(key), result, window, cx);
        } else {
            let task = cx.background_spawn(async move { operation() });
            cx.spawn_in(window, async move |weak, cx| {
                let result = task.await;
                let _ = weak.update_in(cx, |root, window, cx| {
                    if root.client_ui.epoch != epoch
                        || (remote
                            && (!guard.valid()
                                || root.front_page_client(cx).target != guard.target))
                    {
                        return;
                    }
                    root.finish_workspace_rename(
                        pane,
                        input,
                        remote.then_some(key),
                        result,
                        window,
                        cx,
                    );
                });
            })
            .detach();
        }
        cx.notify();
    }

    fn finish_workspace_rename(
        &mut self,
        pane: bool,
        input: Entity<runner_app::ui::TextField>,
        remote_key: Option<String>,
        result: Result<(), String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let remote = remote_key.is_some();
        if let Some(key) = remote_key {
            self.remote_window
                .as_mut()
                .unwrap()
                .actions_pending
                .remove(&key);
        }
        match result {
            Ok(()) => {
                if pane
                    && (!remote
                        || self
                            .pane_rename
                            .as_ref()
                            .is_some_and(|rename| rename.input == input))
                {
                    self.pane_rename = None;
                    self._pane_rename_focus_subscription = None;
                }
                if !pane
                    && (!remote
                        || self
                            .chat_rename_modal
                            .as_ref()
                            .is_some_and(|modal| modal.input == input))
                {
                    self.chat_rename_modal = None;
                }
                if !remote {
                    self.refresh_sessions(cx);
                    if let Err(error) = self.reload_tabs(cx) {
                        self.chat_error = Some(error.to_string().into());
                    }
                    self.sync_active_chat_detail(cx);
                }
                self.focus_active_terminal(window, cx);
            }
            Err(error) => {
                if let Some(modal) = self
                    .chat_rename_modal
                    .as_mut()
                    .filter(|modal| (remote && modal.input == input) || (!remote && !pane))
                {
                    modal.submitting = false;
                    modal.error = Some(error);
                } else if remote {
                    self.remote_window.as_mut().unwrap().action_error = Some(error);
                } else {
                    self.chat_error = Some(error.into());
                    if let Some(rename) = &self.pane_rename {
                        rename.input.read(cx).focus_handle().focus(window, cx);
                    }
                }
            }
        }
        cx.notify();
    }
}

impl NativeRoot {
    pub(crate) fn remote_archive_sessions(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        if self.remote_window.is_none() {
            return;
        }
        let Some((action, operation)) = WorkspaceAction::archive(self, ids, None, None, cx) else {
            return;
        };
        let task = cx.background_spawn(async move { operation() });
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |root, cx| {
                WorkspaceAction::complete(root, action, result, |done| done.completion, cx);
            });
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn close_remote_front_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(workspace) = self.remote_mission_workspace() {
            workspace.update(cx, |workspace, cx| workspace.close_visible_tab(window, cx));
            return;
        }
        let Some(layout) = self.front_page_layout().cloned() else {
            return;
        };
        if layout.drawer_open() && self.drawer_focus.contains_focused(window, cx) {
            self.hide_terminal_drawer(window, cx);
            return;
        }
        if layout.root.leaves().len() > 1 {
            let pane = layout.focused_pane_id.clone();
            match layout
                .focused_session_id()
                .and_then(|id| self.front_page_session(id, cx))
            {
                Some(session) if session.agent_runtime == "shell" => {
                    self.request_close_terminal_pane(&pane, &session.session_id, window, cx)
                }
                Some(session) => {
                    self.request_close_chat_pane(&pane, &session.session_id, window, cx)
                }
                None => self.close_pane(&pane, window, cx),
            }
        } else if !layout.all_session_ids().is_empty() {
            self.request_archive_all(
                Some(&layout.id),
                layout.all_session_ids(),
                ArchiveAllSource::Chat,
                window,
                cx,
            );
        } else {
            let id = layout.id;
            self.remote_action(
                format!("archive-tab:{id}"),
                &["archive"],
                move |client| {
                    client.archive(ActionTarget {
                        kind: ActionKind::Tab,
                        id,
                    })?;
                    Ok(Default::default())
                },
                cx,
            );
        }
    }
}
