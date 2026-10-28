use super::*;
use runner_app::ui::TextField;

impl RemoteClient {
    pub(crate) fn open_remote_project_picker(
        &mut self,
        anchor: Option<Bounds<Pixels>>,
        window: &mut Window,
        cx: &mut Context<NativeRoot>,
    ) {
        let client = self.client(cx);
        if !client.supports(&["folders", "createProject"]) {
            return;
        }
        let Some(anchor) = anchor else {
            return;
        };
        let remote = self.workspace.as_ref().unwrap();
        let name = remote.computer.name.clone();
        let platform = remote.computer.info.platform.clone();
        let input = cx.new(|cx| TextField::new(cx.focus_handle(), "", "", false));
        let picker = cx.new(|cx| {
            crate::surfaces::front_page::folder_picker::FolderPicker::new(
                client.clone(),
                name,
                platform,
                input.clone(),
                Vec::new(),
                cx,
            )
        });
        let epoch = self.ui.epoch;
        let subscription = cx.observe(&input, move |root, input, cx| {
            if root.remote_client.ui.epoch != epoch
                || !client.valid()
                || root.front_page_client(cx).target != client.target
            {
                return;
            }
            let path = input.read(cx).text().to_owned();
            if path.is_empty() {
                return;
            }
            let name = path
                .rsplit(['/', '\\'])
                .find(|part| !part.is_empty())
                .unwrap_or(&path)
                .to_owned();
            if let Some(remote) = root.remote_client.workspace.as_mut() {
                remote.project_picker = None;
                remote.project_picker_subscription = None;
            }
            root.remote_client.remote_action(
                "createProject".into(),
                &["createProject"],
                move |client| {
                    client.create_project(name, path)?;
                    Ok(Default::default())
                },
                cx,
            );
        });
        let remote = self.workspace.as_mut().unwrap();
        remote.project_picker = Some(picker.clone());
        remote.project_picker_subscription = Some(subscription);
        let popup_width = px(320. * self.settings(cx).app_zoom);
        picker.update(cx, |picker, cx| {
            picker.set_anchor(anchor, cx);
            picker.set_popup_width(popup_width);
            picker.return_focus(window.focused(cx));
            picker.browse(window, cx);
        });
        cx.notify();
    }
}

impl RemoteClient {
    pub(crate) fn open_delete_project(&mut self, id: String, cx: &mut Context<NativeRoot>) {
        if let Some(remote) = self.workspace.as_mut() {
            remote.delete_project = Some(id);
            remote.delete_project_force = false;
            remote.delete_project_error = None;
            cx.notify();
        }
    }
    pub(crate) fn prepare_delete_project(&mut self) -> Option<(String, bool)> {
        let remote = self.workspace.as_mut()?;
        let project = remote.delete_project.clone()?;
        if remote.delete_project_busy {
            return None;
        }
        remote.delete_project_busy = true;
        remote.delete_project_error = None;
        Some((project, remote.delete_project_force))
    }
    pub(crate) fn complete_delete_project(
        &mut self,
        guard: &crate::surfaces::front_page::FrontPageClient,
        epoch: u64,
        id: &str,
        result: runner_core::protocol::ClientResult<()>,
        cx: &mut Context<NativeRoot>,
    ) {
        if self.ui.epoch != epoch || !guard.valid() {
            return;
        }
        let Some(remote) = self.workspace.as_mut().filter(|remote| {
            remote.front_page.target == guard.target && remote.delete_project.as_deref() == Some(id)
        }) else {
            return;
        };
        remote.delete_project_busy = false;
        match result {
            Ok(()) => remote.delete_project = None,
            Err(error) => {
                remote.delete_project_force =
                    error.message.to_ascii_lowercase().contains("running");
                remote.delete_project_error = Some(error.to_string());
            }
        }
        cx.notify();
    }
    pub(crate) fn cancel_delete_project(&mut self, cx: &mut Context<NativeRoot>) {
        if let Some(remote) = self.workspace.as_mut() {
            if !remote.delete_project_busy {
                remote.delete_project = None;
                cx.notify();
            }
        }
    }
    pub(crate) fn project_deletion(&self, cx: &App) -> Option<(String, bool)> {
        let remote = self.workspace.as_ref()?;
        let id = remote.delete_project.as_ref()?;
        self.app_store
            .read(cx)
            .remote_clients
            .read(cx)
            .overviews
            .get(&remote.computer.id)?
            .projects
            .iter()
            .find(|project| &project.id == id)
            .map(|project| (project.name.clone(), remote.delete_project_busy))
    }
    pub(crate) fn project_deletion_error(&self) -> Option<&str> {
        self.workspace.as_ref()?.delete_project_error.as_deref()
    }
    pub(crate) fn project_picker(
        &self,
    ) -> Option<Entity<crate::surfaces::front_page::folder_picker::FolderPicker>> {
        self.workspace.as_ref()?.project_picker.clone()
    }
}
