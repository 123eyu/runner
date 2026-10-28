use super::*;

impl RemoteClient {
    pub(crate) fn invalidate_library(&mut self) {
        if let Some(remote) = self.workspace.as_mut() {
            remote.library_target = None;
        }
    }
    pub(crate) fn ensure_remote_library(&mut self, cx: &mut Context<NativeRoot>) {
        if self.workspace.as_ref().is_some_and(|remote| {
            matches!(remote.active, Some(RemoteItem::Roles | RemoteItem::Crews))
                && remote.library_target.as_ref() != Some(&remote.front_page.target)
                && remote.front_page.supports(&["catalog"])
        }) {
            self.refresh_remote_library(cx);
        }
    }
    pub(crate) fn refresh_remote_library(&mut self, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        if !matches!(remote.active, Some(RemoteItem::Roles | RemoteItem::Crews))
            || !remote.front_page.supports(&["catalog"])
        {
            return;
        }
        remote.library_request = remote.library_request.wrapping_add(1);
        let request = remote.library_request;
        let navigation = remote.navigation_generation;
        let client = remote.front_page.clone();
        remote.library_target = Some(client.target.clone());
        remote.role_surfaces.set_library_load(None);
        remote.crew_surfaces.set_library_load(None);
        let epoch = self.ui.epoch;
        let guard = client.clone();
        let task = cx.background_spawn(async move { client.catalog() });
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |root, cx| {
                if root.remote_client.ui.epoch != epoch || !guard.valid() {
                    return;
                }
                let Some(remote) = root.remote_client.workspace.as_mut().filter(|remote| {
                    remote.front_page.target == guard.target
                        && remote.library_request == request
                        && remote.navigation_generation == navigation
                }) else {
                    return;
                };
                match result {
                    Ok(catalog) => {
                        remote.library_catalog = Some(catalog);
                        root.load_role_page(cx);
                        root.load_crew_page(cx);
                    }
                    Err(error) => {
                        let error = error.to_string();
                        remote.role_surfaces.set_library_load(Some(error.clone()));
                        remote.crew_surfaces.set_library_load(Some(error));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

impl RemoteClient {
    pub(crate) fn library_view_current(
        &self,
        client: &crate::surfaces::front_page::FrontPageClient,
        epoch: u64,
        cx: &App,
    ) -> bool {
        if client.is_remote() {
            self.is_active()
                && self.ui.epoch == epoch
                && self.client(cx).target == client.target
                && client.valid()
        } else {
            self.is_local()
        }
    }
    pub(crate) fn render_library_refresh(&self, cx: &Context<NativeRoot>) -> gpui::AnyElement {
        use gpui::prelude::*;
        let root = cx.entity();
        let client = self.client(cx);
        let epoch = self.ui.epoch;
        gpui::div()
            .when(cfg!(test), |entry| {
                entry.debug_selector(|| "REMOTE_LIBRARY_REFRESH".into())
            })
            .child(
                runner_app::ui::Button::new(
                    "refresh-host-library",
                    crate::i18n::text("client.refresh_library"),
                )
                .disabled(!self.client(cx).supports(&["catalog"]))
                .on_press(move |_, cx| {
                    root.update(cx, |root, cx| {
                        if root.remote_client.library_view_current(&client, epoch, cx) {
                            root.remote_client.refresh_remote_library(cx);
                        }
                    })
                }),
            )
            .into_any_element()
    }
}

impl RemoteClient {
    pub(crate) fn role_surfaces(&self) -> Option<&crate::surfaces::roles::RoleSurfaces> {
        self.workspace.as_ref().map(|remote| &remote.role_surfaces)
    }
    pub(crate) fn role_surfaces_mut(
        &mut self,
    ) -> Option<&mut crate::surfaces::roles::RoleSurfaces> {
        self.workspace
            .as_mut()
            .map(|remote| &mut remote.role_surfaces)
    }
    pub(crate) fn load_role_page(&mut self, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        let Some(catalog) = remote.library_catalog.as_ref() else {
            return;
        };
        if remote
            .role_surfaces
            .load_remote_page(catalog, &remote.front_page.target, self.ui.epoch)
        {
            self.load_role_page(cx);
        }
        cx.notify();
    }
    pub(crate) fn crew_surfaces(&self) -> Option<&crate::surfaces::crews::CrewSurfaces> {
        self.workspace.as_ref().map(|remote| &remote.crew_surfaces)
    }
    pub(crate) fn crew_surfaces_mut(
        &mut self,
    ) -> Option<&mut crate::surfaces::crews::CrewSurfaces> {
        self.workspace
            .as_mut()
            .map(|remote| &mut remote.crew_surfaces)
    }
    pub(crate) fn load_crew_page(&mut self, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        let Some(catalog) = remote.library_catalog.as_ref() else {
            return;
        };
        if remote
            .crew_surfaces
            .load_remote_page(catalog, &remote.front_page.target, self.ui.epoch)
        {
            self.load_crew_page(cx);
        }
        cx.notify();
    }
}
