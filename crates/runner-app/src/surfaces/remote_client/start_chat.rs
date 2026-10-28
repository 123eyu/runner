use super::*;
use crate::surfaces::{
    front_page::model::RuntimeChoice,
    start_chat::{ChatMode, ChatTarget},
};
use runner_core::protocol::project::ProjectScope;

impl RemoteClient {
    pub(in crate::surfaces) fn open_remote_start_chat_with_role(
        &mut self,
        target: ChatTarget,
        scope: ProjectScope,
        default_role_id: Option<String>,
        window: &mut Window,
        cx: &mut Context<NativeRoot>,
    ) {
        let client = self.client(cx);
        let required = if matches!(target, ChatTarget::Pane { .. }) {
            vec!["catalog", "startChat", "layout"]
        } else {
            vec!["catalog", "startChat"]
        };
        if !client.supports(&required) {
            return;
        }
        let remote = self.workspace.as_mut().unwrap();
        if !remote.actions_pending.insert("catalog".into()) {
            return;
        }
        remote.form_generation += 1;
        let generation = remote.form_generation;
        let epoch = self.ui.epoch;
        let guard = client.clone();
        let work = cx.background_spawn(async move { client.catalog() });
        cx.spawn_in(window, async move |weak, cx| {
            let result = work.await;
            let _ = weak.update_in(cx, |root, window, cx| {
                if root.remote_client.ui.epoch != epoch || !guard.valid() {
                    return;
                }
                let Some(remote) = root.remote_client.workspace.as_mut() else {
                    return;
                };
                remote.actions_pending.remove("catalog");
                if remote.form_generation != generation {
                    cx.notify();
                    return;
                }
                match result {
                    Ok(catalog) => {
                        if default_role_id
                            .as_ref()
                            .is_some_and(|id| !catalog.roles.iter().any(|role| &role.id == id))
                        {
                            remote.action_error =
                                Some(crate::i18n::text("client.role_unavailable").to_owned());
                            cx.notify();
                            return;
                        }
                        let role_launch = default_role_id.is_some();
                        remote.catalog = Some(catalog.clone());
                        let roles = catalog.roles.clone().into_iter().map(Into::into).collect();
                        let runtimes = catalog
                            .runtimes
                            .iter()
                            .filter_map(RuntimeChoice::remote)
                            .collect();
                        root.build_start_chat_modal(
                            target,
                            default_role_id,
                            roles,
                            runtimes,
                            Vec::new(),
                            scope,
                            String::new(),
                            String::new(),
                            String::new(),
                            false,
                            None,
                            None,
                            guard,
                            Some(catalog),
                            window,
                            cx,
                        );
                        if role_launch {
                            root.switch_start_chat_mode(ChatMode::Role, window, cx);
                        }
                    }
                    Err(error) => remote.action_error = Some(error.to_string()),
                }
                root.sync_remote_state(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

pub(crate) struct ChatLaunchAction {
    client: crate::surfaces::front_page::FrontPageClient,
    epoch: u64,
    before: HashMap<String, PaneLayout>,
    navigation: u64,
    key: Option<String>,
}
impl RemoteClient {
    pub(crate) fn prepare_chat_launch(
        &mut self,
        client: crate::surfaces::front_page::FrontPageClient,
        placement: Option<&(PaneLayout, String)>,
    ) -> Option<ChatLaunchAction> {
        let remote = self.workspace.as_mut()?;
        let key = placement.map(|(layout, _)| format!("layout:{}", layout.id));
        if let Some(key) = &key {
            remote.actions_pending.insert(key.clone());
        }
        Some(ChatLaunchAction {
            client,
            epoch: self.ui.epoch,
            before: remote.layouts.clone(),
            navigation: remote.navigation_generation,
            key,
        })
    }
    pub(crate) fn settle_chat_launch(&mut self, action: &ChatLaunchAction) -> bool {
        if self.ui.epoch != action.epoch
            || !action.client.valid()
            || self
                .workspace
                .as_ref()
                .is_none_or(|remote| remote.front_page.target != action.client.target)
        {
            return false;
        }
        if let Some(key) = &action.key {
            self.finish_named_action(key);
        }
        true
    }
    pub(crate) fn complete_chat_launch(
        &mut self,
        action: ChatLaunchAction,
        session: String,
        layout: Option<PaneLayout>,
        cx: &mut Context<NativeRoot>,
    ) {
        self.apply_remote_completion(
            crate::surfaces::front_page::actions::Completion {
                session: Some(session),
                layout,
                ..Default::default()
            },
            &action.before,
            action.navigation,
            cx,
        );
    }
}

impl RemoteClient {
    pub(crate) fn begin_mission_form(
        &mut self,
    ) -> Option<(crate::surfaces::front_page::FrontPageClient, u64, u64)> {
        let remote = self.workspace.as_mut()?;
        if !remote.front_page.supports(&["catalog", "startMission"]) {
            return None;
        }
        remote.form_generation = remote.form_generation.wrapping_add(1);
        Some((
            remote.front_page.clone(),
            remote.navigation_generation,
            remote.form_generation,
        ))
    }
}
