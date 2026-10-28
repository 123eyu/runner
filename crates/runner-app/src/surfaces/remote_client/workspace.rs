use super::*;
use runner_app::ui::button::spinner;
use runner_core::companion::{Overview, SidebarNode, SidebarNodeKind, Status};
use runner_core::remote::RemoteTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Item {
    Roles,
    Crews,
    Tab(String),
    Mission(String),
}
pub(super) struct RemotePane {
    _lifetime: transport::AttachmentLifetime,
    pub(super) attachment: AttachedChat,
    control_failed: bool,
}
pub(crate) struct RemoteWindow {
    pub(crate) computer: Computer,
    pub(crate) front_page: crate::surfaces::front_page::FrontPageClient,
    pub(crate) sidebar: Entity<Sidebar>,
    pub(crate) mission_workspace: Entity<MissionWorkspace>,
    pub(crate) sidebar_collapsed: bool,
    pub(crate) sidebar_width: f32,
    _follow: Option<model::Follow>,
    pub(crate) active: Option<Item>,
    pub(crate) layouts: HashMap<String, PaneLayout>,
    pub(crate) overview_revision: u64,
    pub(crate) resize_draft: Option<PaneLayout>,
    pub(crate) resizing_split: Option<String>,
    pub(crate) drawer_resizing: bool,
    pub(crate) pane_bounds: HashMap<PaneKey, gpui::Size<Pixels>>,
    pub(crate) layout_hints: HashMap<String, PaneLayout>,
    pub(crate) actions_pending: HashSet<String>,
    pub(crate) action_error: Option<String>,
    pub(crate) shell_placement: super::actions::ShellPlacement,
    pub(crate) catalog: Option<runner_core::companion::Catalog>,
    pub(crate) library_catalog: Option<runner_core::companion::Catalog>,
    pub(crate) library_request: u64,
    pub(crate) library_target: Option<crate::surfaces::front_page::FrontPageTarget>,
    pub(crate) role_surfaces: crate::surfaces::roles::RoleSurfaces,
    pub(crate) crew_surfaces: crate::surfaces::crews::CrewSurfaces,
    catalog_target: Option<crate::surfaces::front_page::FrontPageTarget>,
    pub(crate) project_picker:
        Option<Entity<crate::surfaces::front_page::folder_picker::FolderPicker>>,
    pub(crate) project_picker_subscription: Option<Subscription>,
    pub(crate) delete_project: Option<String>,
    pub(crate) delete_project_busy: bool,
    pub(crate) delete_project_force: bool,
    pub(crate) delete_project_error: Option<String>,
    pub(crate) navigation_generation: u64,
    pub(crate) activate_session: Option<String>,
    pub(crate) activate_placement: Option<(String, Option<String>)>,
    pub(crate) reopen_sessions: HashSet<String>,
    pub(crate) activate_tab: Option<String>,
    pub(crate) activate_mission: Option<String>,
    pub(crate) form_generation: u64,
    pub(super) terminals: HashMap<String, RemotePane>,
    recovery: HashSet<String>,
    terminal_epoch: u64,
    pair_epoch: u64,
    attachment_generation: u64,
    pending: HashMap<String, Arc<()>>,
    sizes: HashMap<String, (u16, u16)>,
    errors: HashSet<String>,
    connected: Arc<AtomicBool>,
    pub(super) connection: Connection,
    epoch: u64,
    wake: futures::channel::mpsc::UnboundedSender<String>,
}
impl Drop for RemoteWindow {
    fn drop(&mut self) {
        self.connected.store(false, Ordering::Release);
    }
}
impl RemoteWindow {
    fn visible_sessions(&self, _overview: &Overview, cx: &App) -> Vec<String> {
        match self.active.as_ref() {
            Some(Item::Mission(_)) => self.mission_workspace.read(cx).remote_visible_sessions(),
            Some(Item::Tab(id)) => self
                .layouts
                .get(id)
                .map(|layout| {
                    let mut ids = layout.session_ids();
                    if layout.drawer_open() {
                        if let Some(id) = layout.active_drawer_shell() {
                            ids.push(id.into());
                        }
                    }
                    ids
                })
                .unwrap_or_default(),
            None | Some(Item::Roles | Item::Crews) => Vec::new(),
        }
    }
}

impl RemoteClient {
    pub(crate) fn report_remote_input_error(
        &mut self,
        address: &str,
        message: String,
        cx: &mut Context<NativeRoot>,
    ) {
        let Some((computer, session)) = runner_core::remote::terminal_target(address) else {
            return;
        };
        let clients = state(&self.app_store, cx);
        let Some(remote) = self.workspace.as_mut().filter(|remote| {
            remote.computer.id == computer && remote.terminals.contains_key(session)
        }) else {
            return;
        };
        let visible = clients
            .read(cx)
            .overviews
            .get(computer)
            .is_some_and(|overview| {
                remote
                    .visible_sessions(overview, cx)
                    .iter()
                    .any(|id| id == session)
            });
        if visible {
            remote.action_error = Some(message);
            cx.notify();
        }
    }

    pub(crate) fn remote_terminal_interactive(&self, id: &str, cx: &App) -> bool {
        self.workspace.as_ref().is_some_and(|remote| {
            remote.connection == Connection::Connected
                && remote
                    .terminals
                    .get(id)
                    .is_some_and(|pane| pane.attachment.terminal.control().desktop())
                && self
                    .app_store
                    .read(cx)
                    .remote_clients
                    .read(cx)
                    .overviews
                    .get(&remote.computer.id)
                    .and_then(|overview| overview.sessions.get(id))
                    .is_some_and(|session| {
                        !matches!(
                            session.status.status.effective(),
                            Status::Stopped | Status::Error
                        )
                    })
        })
    }

    pub(crate) fn remote_terminal_size(&self, id: &str) -> Option<(u16, u16)> {
        let remote = self.workspace.as_ref()?;
        remote
            .terminals
            .get(id)
            .map(|pane| pane.attachment.terminal.size())
            .or_else(|| remote.sizes.get(id).copied())
    }

    pub(crate) fn remote_terminal_palette(
        &self,
        cx: &App,
    ) -> Option<runner_terminal::palette::TerminalPalette> {
        let remote = self.workspace.as_ref()?;
        state(&self.app_store, cx)
            .read(cx)
            .overviews
            .get(&remote.computer.id)?
            .terminal_palette
            .clone()
            .map(|palette| runner_terminal::terminal::palette_from_wire(palette.into()))
    }

    pub(crate) fn configure_remote_terminals(&self, cx: &App) {
        let Some(remote) = self.workspace.as_ref() else {
            return;
        };
        for pane in remote.terminals.values() {
            app_settings::configure_terminal(&pane.attachment.terminal, self.settings(cx));
        }
    }

    pub(crate) fn focus_remote_terminal(&self, id: &str, window: &mut Window, cx: &mut App) {
        if let Some(pane) = self
            .workspace
            .as_ref()
            .and_then(|remote| remote.terminals.get(id))
        {
            pane.attachment.terminal_focus.focus(window, cx);
        }
    }

    pub(crate) fn visit_computer(
        &mut self,
        computer: Computer,
        window_label: &str,
        window: &mut Window,
        cx: &mut Context<NativeRoot>,
    ) -> bool {
        if !self.app_store.read(cx).remote_access_enabled() {
            return false;
        }
        if let Some(remote) = self
            .workspace
            .as_mut()
            .filter(|remote| remote.computer.id == computer.id)
        {
            remote.computer = computer.clone();
            self.ui.picker_open = false;
            if !matches!(
                remote.connection,
                Connection::Unreachable | Connection::Revoked | Connection::Forgotten
            ) {
                cx.notify();
                return false;
            }
            remote.terminals.clear();
            remote.pending.clear();
            remote.recovery.clear();
            remote.attachment_generation = remote.attachment_generation.wrapping_add(1);
            remote.errors.clear();
            state(&self.app_store, cx)
                .read(cx)
                .send(Command::Retry(computer.id));
            return false;
        }
        self.remember_remote_selection(cx);
        if let Some(remote) = &self.workspace {
            remote
                .mission_workspace
                .update(cx, |workspace, cx| workspace.release_window(cx));
        }
        self.ui.epoch += 1;
        let epoch = self.ui.epoch;
        let clients = state(&self.app_store, cx);
        let follow = clients.read(cx).follow(computer.id.clone());
        let (wake, mut wakes) = futures::channel::mpsc::unbounded();
        cx.spawn(async move |weak, cx| {
            while let Some(id) = wakes.next().await {
                if weak
                    .update(cx, |root, cx| {
                        if root
                            .remote_client
                            .workspace
                            .as_ref()
                            .is_some_and(|remote| remote.epoch == epoch)
                        {
                            root.app_store
                                .read(cx)
                                .terminal_updates
                                .clone()
                                .update(cx, |_, cx| cx.emit(id));
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        let pair_epoch = clients
            .read(cx)
            .pair_epochs
            .get(&computer.id)
            .copied()
            .unwrap_or_default();
        let access = clients.update(cx, |state, cx| state.front_page_access(&computer, cx));
        let front_page =
            crate::surfaces::front_page::FrontPageClient::remote(self.core(cx).clone(), access);
        let selection = self
            .ui
            .selections
            .remove(&computer.id)
            .filter(|_| clients.read(cx).overviews.contains_key(&computer.id))
            .unwrap_or_default();
        let shell = cx.weak_entity();
        let sidebar =
            cx.new(|sidebar_cx| Sidebar::new(shell, self.app_store.clone(), None, sidebar_cx));
        sidebar.update(cx, |sidebar, cx| {
            sidebar.remote_settings = selection.sidebar_settings.map(|saved| AppSettings {
                sidebar_active_only: saved.sidebar_active_only,
                sidebar_collapsed_projects: saved.sidebar_collapsed_projects,
                sidebar_collapsed_active_groups: saved.sidebar_collapsed_active_groups,
                sidebar_projects_open: saved.sidebar_projects_open,
                sidebar_chats_open: saved.sidebar_chats_open,
                ..self.settings(cx).clone()
            });
            sidebar.set_front_page_target(front_page.clone(), cx)
        });
        let shell = cx.weak_entity();
        let mission_workspace = cx.new(|workspace_cx| {
            MissionWorkspace::new_remote(
                window_label.to_owned(),
                shell,
                self.app_store.clone(),
                front_page.clone(),
                window,
                workspace_cx,
            )
        });
        self.workspace = Some(RemoteWindow {
            mission_workspace,
            sidebar,
            sidebar_collapsed: false,
            sidebar_width: self.settings(cx).sidebar_width,
            front_page,
            computer,
            _follow: Some(follow),
            active: selection.active,
            layouts: selection.layouts,
            overview_revision: 0,
            resize_draft: None,
            resizing_split: None,
            drawer_resizing: false,
            pane_bounds: HashMap::new(),
            layout_hints: HashMap::new(),
            actions_pending: HashSet::new(),
            action_error: None,
            shell_placement: Arc::new(Mutex::new(None)),
            catalog: None,
            library_catalog: None,
            library_request: 0,
            library_target: None,
            role_surfaces: crate::surfaces::roles::RoleSurfaces::new_remote(
                cx.entity(),
                self.settings(cx).roles_page_size,
                epoch,
                cx,
            ),
            crew_surfaces: crate::surfaces::crews::CrewSurfaces::new_remote(
                cx.entity(),
                self.settings(cx).crews_page_size,
                epoch,
                cx,
            ),
            catalog_target: None,
            project_picker: None,
            project_picker_subscription: None,
            delete_project: None,
            delete_project_busy: false,
            delete_project_force: false,
            delete_project_error: None,
            navigation_generation: 0,
            activate_session: None,
            activate_placement: None,
            reopen_sessions: HashSet::new(),
            activate_tab: None,
            activate_mission: None,
            form_generation: 0,
            terminals: HashMap::new(),
            recovery: HashSet::new(),
            terminal_epoch: clients.read(cx).terminal_epoch,
            pair_epoch,
            attachment_generation: 0,
            pending: HashMap::new(),
            sizes: HashMap::new(),
            errors: HashSet::new(),
            connected: Arc::new(AtomicBool::new(false)),
            connection: Connection::Opening,
            epoch,
            wake,
        });
        self.ui.picker_open = false;
        self.root_focus.focus(window, cx);
        cx.notify();
        true
    }

    pub(crate) fn return_to_this_computer(
        &mut self,
        window: &mut Window,
        cx: &mut Context<NativeRoot>,
    ) {
        self.remember_remote_selection(cx);
        if let Some(remote) = &self.workspace {
            remote
                .mission_workspace
                .update(cx, |workspace, cx| workspace.release_window(cx));
        }
        self.workspace = None;
        self.ui.picker_open = false;
        self.root_focus.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn remember_remote_selection(&mut self, cx: &App) {
        if let Some(remote) = &self.workspace {
            self.ui.selections.insert(
                remote.computer.id.clone(),
                RemoteSelection {
                    active: remote.active.clone(),
                    layouts: remote.layouts.clone(),
                    sidebar_settings: remote.sidebar.read(cx).remote_settings.clone(),
                },
            );
        }
    }

    pub(crate) fn sync_remote_state(&mut self, cx: &mut Context<NativeRoot>) -> bool {
        self.sync_computer_pair(cx);
        let clients = state(&self.app_store, cx);
        let Some(remote) = self.workspace.as_mut() else {
            cx.notify();
            return false;
        };
        if let Some(computer) = clients
            .read(cx)
            .computers
            .iter()
            .find(|computer| computer.id == remote.computer.id)
        {
            remote.computer = computer.clone();
        }
        let pair_epoch = clients
            .read(cx)
            .pair_epochs
            .get(&remote.computer.id)
            .copied()
            .unwrap_or_default();
        if remote.pair_epoch != pair_epoch {
            remote.terminals.clear();
            remote.pending.clear();
            remote.recovery.clear();
            remote.errors.clear();
            remote.pair_epoch = pair_epoch;
            remote.attachment_generation = remote.attachment_generation.wrapping_add(1);
        }
        let terminal_epoch = clients.read(cx).terminal_epoch;
        if remote.terminal_epoch != terminal_epoch {
            for (id, pane) in &remote.terminals {
                if let Some(size) = pane.attachment.terminal_interaction.read(cx).pane_size {
                    remote
                        .sizes
                        .insert(id.clone(), (size.cols.max(20), size.rows.max(8)));
                }
            }
            remote.recovery.extend(remote.terminals.keys().cloned());
            remote.recovery.extend(remote.pending.keys().cloned());
            remote.terminals.clear();
            remote.pending.clear();
            remote.errors.clear();
            remote.terminal_epoch = terminal_epoch;
            remote.attachment_generation = remote.attachment_generation.wrapping_add(1);
        }
        let connection = clients.read(cx).connection(&remote.computer.id);
        let connection = if !remote.computer.paired && connection == Connection::Connected {
            Connection::Revoked
        } else {
            connection
        };
        let trusted = clients.read(cx).computers.iter().any(|computer| {
            computer.id == remote.computer.id
                && computer.fingerprint == remote.computer.fingerprint
                && computer.paired
        }) && !matches!(connection, Connection::Revoked | Connection::Forgotten);
        if !trusted {
            remote._follow = None;
        } else if remote._follow.is_none() {
            remote._follow = Some(clients.read(cx).follow(remote.computer.id.clone()));
        }

        if connection == Connection::Forgotten && remote.connection != connection {
            remote.terminals.clear();
            remote.pending.clear();
            remote.recovery.clear();
            remote.errors.clear();
            remote.attachment_generation = remote.attachment_generation.wrapping_add(1);
        }
        let revision = clients
            .read(cx)
            .overview_revisions
            .get(&remote.computer.id)
            .copied()
            .unwrap_or_default();
        let pushed = revision != remote.overview_revision;
        remote.overview_revision = revision;
        if let Some(overview) = clients.read(cx).overviews.get(&remote.computer.id) {
            if pushed {
                remote.layout_hints.retain(|id, _| {
                    !remote.layouts.contains_key(id)
                        || overview.tabs.iter().any(|tab| &tab.id == id)
                });
            }
            remote
                .layouts
                .retain(|id, _| overview.tabs.iter().any(|tab| &tab.id == id));
            for tab in &overview.tabs {
                let layout = crate::surfaces::front_page::model::reconcile_tab(
                    tab,
                    remote.layouts.get(&tab.id),
                    remote.layout_hints.get(&tab.id),
                );
                if remote.layout_hints.get(&tab.id).is_some_and(|hint| {
                    crate::surfaces::front_page::model::matches_layout(hint, tab)
                        || (pushed
                            && remote.layouts.get(&tab.id).is_some_and(|old| {
                                !crate::surfaces::front_page::model::matches_layout(old, tab)
                            }))
                }) {
                    remote.layout_hints.remove(&tab.id);
                }
                remote.layouts.insert(tab.id.clone(), layout);
            }
        }
        if let Some(overview) = clients.read(cx).overviews.get(&remote.computer.id) {
            let live = remote
                .reopen_sessions
                .iter()
                .filter(|id| {
                    overview.sessions.get(*id).is_some_and(|session| {
                        !matches!(
                            session.status.status.effective(),
                            Status::Stopped | Status::Error
                        )
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            for id in live {
                remote.reopen_sessions.remove(&id);
                remote.terminals.remove(&id);
                remote.pending.remove(&id);
                remote.recovery.remove(&id);
            }
        }
        if let Some(session) = &remote.activate_session {
            let destination = if let Some((tab, pane)) = &remote.activate_placement {
                remote
                    .layouts
                    .get(tab)
                    .filter(|layout| match pane {
                        Some(pane) => layout.root.leaves().iter().any(|leaf| {
                            &leaf.id == pane && leaf.session_id.as_ref() == Some(session)
                        }),
                        None => layout.drawer_shells().contains(session),
                    })
                    .map(|_| tab.clone())
            } else {
                remote
                    .layouts
                    .iter()
                    .find(|(_, layout)| layout.all_session_ids().contains(session))
                    .map(|(id, _)| id.clone())
            };
            if let Some(id) = destination {
                let layout = remote.layouts.get_mut(&id).unwrap();
                if let Some((_, pane)) = &remote.activate_placement {
                    if let Some(pane) = pane {
                        layout.focus_pane(pane);
                    } else {
                        layout.activate_drawer_shell(session);
                        layout.set_drawer_open(true);
                    }
                }
                remote.active = Some(Item::Tab(id));
                remote.activate_session = None;
                remote.activate_placement = None;
            }
        }
        if let Some(tab) = &remote.activate_tab {
            if remote.layouts.contains_key(tab) {
                remote.active = Some(Item::Tab(tab.clone()));
                remote.activate_tab = None;
            }
        }
        if let Some(mission) = &remote.activate_mission {
            if clients
                .read(cx)
                .overviews
                .get(&remote.computer.id)
                .is_some_and(|overview| overview.missions.iter().any(|item| &item.id == mission))
            {
                remote.active = Some(Item::Mission(mission.clone()));
                remote.activate_mission = None;
            }
        }
        if let Some(overview) = clients.read(cx).overviews.get(&remote.computer.id) {
            let valid = match &remote.active {
                Some(Item::Tab(id)) => remote.layouts.contains_key(id),
                Some(Item::Mission(id)) => {
                    overview.missions.iter().any(|mission| &mission.id == id)
                }
                Some(Item::Roles | Item::Crews) | None => true,
            };
            if !valid {
                remote.active = None;
            }
        }
        if remote.active.is_none() {
            if let Some(overview) = clients.read(cx).overviews.get(&remote.computer.id) {
                remote.active = overview
                    .tabs
                    .first()
                    .map(|tab| Item::Tab(tab.id.clone()))
                    .or_else(|| {
                        overview
                            .missions
                            .first()
                            .map(|mission| Item::Mission(mission.id.clone()))
                    });
            }
        }
        remote
            .connected
            .store(connection == Connection::Connected, Ordering::Release);
        if connection != Connection::Connected {
            remote.catalog = None;
            remote.library_catalog = None;
            remote.library_target = None;
            remote.role_surfaces.invalidate_library();
            remote.crew_surfaces.invalidate_library();
            remote.catalog_target = None;
            remote.project_picker = None;
            remote.project_picker_subscription = None;
            remote.delete_project = None;
            *remote.shell_placement.lock().unwrap() = None;
            remote.actions_pending.clear();
            remote.layout_hints.clear();
            remote.reopen_sessions.clear();
            remote.activate_session = None;
            remote.activate_placement = None;
            remote.activate_tab = None;
            remote.activate_mission = None;
            for pane in remote.terminals.values() {
                pane.attachment.terminal_input.update(cx, |input, cx| {
                    input.cancel_composition();
                    cx.notify();
                });
            }
        }
        if connection == Connection::Connected && remote.connection != connection {
            remote.errors.clear();
        }
        remote.connection = connection;
        let access = clients.update(cx, |state, cx| {
            state.front_page_access(&remote.computer, cx)
        });
        remote.front_page = crate::surfaces::front_page::FrontPageClient::remote(
            self.app_store.read(cx).client.clone(),
            access,
        );
        let sessions = clients
            .read(cx)
            .overviews
            .get(&remote.computer.id)
            .map(|overview| {
                overview
                    .sessions
                    .iter()
                    .map(|(id, session)| {
                        let native = remote.catalog.as_ref().is_some_and(|catalog| {
                            catalog.runtimes.iter().any(|runtime| {
                                Some(&runtime.name) == session.runtime.as_ref()
                                    && runtime.native_fork
                            })
                        });
                        let pinned = overview.tabs.iter().any(|tab| {
                            tab.pinned
                                && remote
                                    .layouts
                                    .get(&tab.id)
                                    .is_some_and(|layout| layout.contains_session(id))
                        });
                        (
                            id.clone(),
                            crate::surfaces::front_page::model::SessionDisplay::remote(
                                id,
                                session,
                                pinned,
                                native,
                                remote.front_page.supports(&["resume"]),
                            ),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        remote.sidebar.update(cx, |sidebar, cx| {
            sidebar.set_front_page_target(remote.front_page.clone(), cx);
            sidebar.set_remote_presentation(
                remote.layouts.clone(),
                sessions,
                remote.active.clone(),
                cx,
            );
        });
        let workspace = remote.mission_workspace.clone();
        let mission = match &remote.active {
            Some(Item::Mission(id)) => Some(id.clone()),
            _ => None,
        };
        let overview = clients.read(cx).overviews.get(&remote.computer.id).cloned();
        let client = remote.front_page.clone();
        workspace.update(cx, |workspace, cx| {
            workspace.sync_remote(client, mission, overview, clients, cx)
        });
        self.ensure_remote_library(cx);
        self.load_remote_metadata(cx);
        self.attach_remote_panes(cx);
        cx.notify();
        connection != Connection::Connected
    }

    fn load_remote_metadata(&mut self, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        if !remote.front_page.supports(&["catalog"])
            || remote.catalog_target.as_ref() == Some(&remote.front_page.target)
        {
            return;
        }
        remote.catalog = None;
        remote.catalog_target = Some(remote.front_page.target.clone());
        let client = remote.front_page.clone();
        let guard = client.clone();
        let epoch = self.ui.epoch;
        let work = cx.background_spawn(async move { client.catalog() });
        cx.spawn(async move |weak, cx| {
            let result = work.await;
            let _ = weak.update(cx, |root, cx| {
                if root.remote_client.ui.epoch != epoch || !guard.valid() {
                    return;
                }
                if let Some(remote) = root
                    .remote_client
                    .workspace
                    .as_mut()
                    .filter(|remote| remote.front_page.target == guard.target)
                {
                    if let Ok(catalog) = result {
                        remote.catalog = Some(catalog);
                    }
                    root.sync_remote_state(cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(crate) fn select_remote_item(&mut self, item: RemoteItem) -> bool {
        let mut changed = false;
        if let Some(remote) = self.workspace.as_mut() {
            if remote.active.as_ref() != Some(&item) {
                changed = true;
                remote.form_generation = remote.form_generation.wrapping_add(1);
                remote.navigation_generation = remote.navigation_generation.wrapping_add(1);
                remote.activate_session = None;
                remote.activate_placement = None;
                remote.activate_tab = None;
                remote.activate_mission = None;
                for (id, hint) in &mut remote.layout_hints {
                    if let Some(previous) = remote.layouts.get(id) {
                        crate::surfaces::front_page::model::retain_presentation(hint, previous);
                    }
                }
                *remote.shell_placement.lock().unwrap() = None;
                remote.action_error = None;
                remote.recovery.clear();
            }
            remote.active = Some(item);
            remote.errors.clear();
        }
        changed
    }

    pub(crate) fn mark_remote_stopped_sessions_viewed(&mut self, cx: &mut Context<NativeRoot>) {
        let clients = state(&self.app_store, cx);
        let Some(remote) = self.workspace.as_ref() else {
            return;
        };
        if remote.connection != Connection::Connected
            || !remote.front_page.supports(&["markViewed"])
        {
            return;
        }
        let Some(overview) = clients.read(cx).overviews.get(&remote.computer.id) else {
            return;
        };
        let sessions = remote
            .visible_sessions(overview, cx)
            .into_iter()
            .filter(|id| {
                overview.sessions.get(id).is_some_and(|session| {
                    let status = &session.status;
                    matches!(status.status, Status::Stopped | Status::Error)
                        && (status.unread
                            || (status.status == Status::Error && !status.error_acknowledged))
                })
            })
            .collect::<Vec<_>>();
        for session in sessions {
            self.remote_action(
                format!("mark-viewed:{session}"),
                &["markViewed"],
                move |client| {
                    client.mark_viewed(session)?;
                    Ok(Default::default())
                },
                cx,
            );
        }
    }

    pub(crate) fn attach_remote_panes(&mut self, cx: &mut Context<NativeRoot>) {
        let clients = state(&self.app_store, cx);
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        let overview = clients.read(cx).overviews.get(&remote.computer.id).cloned();
        let ids = overview
            .as_ref()
            .map(|overview| remote.visible_sessions(overview, cx))
            .unwrap_or_default();
        let running = overview
            .as_ref()
            .map(|overview| {
                overview
                    .sessions
                    .iter()
                    .filter(|(_, session)| {
                        !matches!(
                            session.status.status.effective(),
                            Status::Stopped | Status::Error
                        )
                    })
                    .map(|(id, _)| id)
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        remote.terminals.retain(|id, _| ids.contains(id));
        remote
            .pending
            .retain(|id, _| ids.contains(id) && running.contains(id));
        remote.errors.retain(|id| running.contains(id));
        if remote.connection != Connection::Connected {
            return;
        }
        for id in ids {
            if !running.contains(&id)
                || remote.terminals.contains_key(&id)
                || remote.pending.contains_key(&id)
                || remote.errors.contains(&id)
            {
                continue;
            }
            let Some(&(cols, rows)) = remote.sizes.get(&id) else {
                continue;
            };
            let request = Arc::new(());
            remote.pending.insert(id.clone(), request.clone());
            let config = RemoteTerminal {
                computer: remote.computer.id.clone(),
                session: id.clone(),
                cols,
                rows,
                take_control: !remote.recovery.contains(&id),
            };
            let address = config.address();
            let title = overview
                .as_ref()
                .and_then(|overview| overview.sessions.get(&id))
                .map(|session| session.title.clone())
                .unwrap_or_default();
            let (client, lifetime) = transport::RemoteTransport::client(
                self.app_store.read(cx).client.clone(),
                config,
                title,
                remote.connected.clone(),
            );
            let epoch = remote.epoch;
            let attachment_generation = remote.attachment_generation;
            let wake = remote.wake.clone();
            let wake_id = address.clone();
            let task = cx.background_spawn(async move {
                TerminalMirror::attach(
                    client,
                    address,
                    Arc::new(move || {
                        let _ = wake.unbounded_send(wake_id.clone());
                    }),
                )
            });
            cx.spawn(async move |weak, cx| {
                let result = task.await;
                let _ = weak.update(cx, |root, cx| {
                    let style = root.terminal_style(cx);
                    let settings = root.app_store.read(cx).settings.clone();
                    let Some(remote) = root.remote_client.workspace.as_mut().filter(|remote| {
                        remote.epoch == epoch
                            && remote.attachment_generation == attachment_generation
                    }) else {
                        return;
                    };
                    if !remote
                        .pending
                        .get(&id)
                        .is_some_and(|pending| Arc::ptr_eq(pending, &request))
                    {
                        return;
                    }
                    remote.pending.remove(&id);
                    let clients = state(&root.app_store, cx);
                    let wanted = clients
                        .read(cx)
                        .overviews
                        .get(&remote.computer.id)
                        .is_some_and(|overview| {
                            remote.visible_sessions(overview, cx).contains(&id)
                                && overview.sessions.get(&id).is_some_and(|session| {
                                    !matches!(
                                        session.status.status.effective(),
                                        Status::Stopped | Status::Error
                                    )
                                })
                        });
                    if !wanted {
                        return;
                    }
                    match result {
                        Ok(terminal) => {
                            let input_address =
                                runner_core::remote::terminal_address(&remote.computer.id, &id);
                            let input_id = id.clone();
                            let input_terminal = terminal.clone();
                            let attachment = AttachedChat::new(
                                terminal,
                                &style,
                                &settings,
                                cx,
                                move |root, result, cx| {
                                    if let Err(error) = result {
                                        let current = root
                                            .remote_client
                                            .workspace
                                            .as_ref()
                                            .is_some_and(|remote| {
                                                remote.epoch == epoch
                                                    && remote.attachment_generation
                                                        == attachment_generation
                                                    && remote.terminals.get(&input_id).is_some_and(
                                                        |pane| {
                                                            Arc::ptr_eq(
                                                                &pane.attachment.terminal,
                                                                &input_terminal,
                                                            )
                                                        },
                                                    )
                                            });
                                        if current {
                                            root.remote_client.report_remote_input_error(
                                                &input_address,
                                                error,
                                                cx,
                                            );
                                        }
                                    }
                                },
                            );
                            let pane = RemotePane {
                                _lifetime: lifetime,
                                attachment,
                                control_failed: false,
                            };
                            remote.recovery.remove(&id);
                            remote.terminals.insert(id.clone(), pane);
                        }
                        Err(_) => {
                            remote.errors.insert(id.clone());
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
        }
    }

    pub(crate) fn render_remote_connection(&self, _cx: &mut Context<NativeRoot>) -> AnyElement {
        let remote = self.workspace.as_ref().unwrap();
        let (key, glyph, color) = match remote.connection {
            Connection::Connected => ("client.connection_live", "shield-check.svg", theme::muted()),
            Connection::Opening => (
                "client.connection_opening",
                "refresh-cw.svg",
                theme::muted(),
            ),
            Connection::Reconnecting => (
                "client.connection_reconnecting",
                "refresh-cw.svg",
                theme::warning(),
            ),
            Connection::Unreachable => (
                "client.connection_unreachable",
                "cloud-off.svg",
                theme::warning(),
            ),
            Connection::Revoked => ("client.connection_revoked", "lock.svg", theme::danger()),
            Connection::Forgotten => ("client.connection_forgotten", "unplug.svg", theme::muted()),
        };
        div()
            .debug_selector(|| "REMOTE_CONNECTION_ROW".into())
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap_2()
            .text_size(theme::text_meta())
            .text_color(color)
            .child(icon(glyph, color))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .child(label(key, &remote.computer.name)),
            )
            .into_any_element()
    }

    pub(crate) fn mission_surface(&self, cx: &mut Context<NativeRoot>) -> Entity<MissionWorkspace> {
        let remote = self.workspace.as_ref().unwrap();
        let mut sizes = remote.sizes.clone();
        sizes.extend(
            remote
                .terminals
                .iter()
                .map(|(id, pane)| (id.clone(), pane.attachment.terminal.size())),
        );
        remote.mission_workspace.update(cx, |workspace, _| {
            workspace.set_remote_terminal_sizes(sizes)
        });
        remote.mission_workspace.clone()
    }

    pub(crate) fn render_remote_workspace(
        &self,
        body: AnyElement,
        cx: &mut Context<NativeRoot>,
    ) -> AnyElement {
        let remote = self.workspace.as_ref().unwrap();
        let retry_shell = remote
            .shell_placement
            .lock()
            .unwrap()
            .as_ref()
            .map(|(_, layout)| layout.clone());
        let retry_root = cx.entity();
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(body)
            .children(remote.action_error.clone().map(|error| {
                runner_app::ui::notice_banner(error, runner_app::ui::Tone::Danger)
                    .debug_selector(|| "REMOTE_ACTION_ERROR".into())
            }))
            .children(
                retry_shell
                    .filter(|_| {
                        remote.action_error.is_some() && remote.front_page.supports(&["layout"])
                    })
                    .map(|draft| {
                        Button::new("retry-shell-placement", crate::i18n::text("ui.retry"))
                            .on_press(move |_, cx| {
                                retry_root.update(cx, |root, cx| {
                                    root.remote_start_shell(
                                        draft.clone(),
                                        None,
                                        (INITIAL_COLS, INITIAL_ROWS),
                                        cx,
                                    )
                                })
                            })
                    }),
            )
            .children((remote.connection == Connection::Reconnecting).then(|| {
                div()
                    .absolute()
                    .top(rems(74. / 16.))
                    .left_0()
                    .right_0()
                    .px_4()
                    .py_2()
                    .bg(theme::with_alpha(theme::warning(), 0.12))
                    .text_color(theme::warning())
                    .text_size(theme::text_body())
                    .child(label("client.reconnecting_banner", &remote.computer.name))
            }))
            .children(
                matches!(
                    remote.connection,
                    Connection::Unreachable | Connection::Revoked | Connection::Forgotten
                )
                .then(|| self.remote_connection_card(cx)),
            )
            .into_any_element()
    }

    pub(crate) fn render_remote_terminal_content(
        &self,
        id: &str,
        cx: &mut Context<NativeRoot>,
    ) -> AnyElement {
        let remote = self.workspace.as_ref().unwrap();
        let clients = state(&self.app_store, cx);
        let running = clients
            .read(cx)
            .overviews
            .get(&remote.computer.id)
            .and_then(|overview| overview.sessions.get(id))
            .is_some_and(|session| {
                !matches!(
                    session.status.status.effective(),
                    Status::Stopped | Status::Error
                )
            });
        let mut surface = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0();
        let Some(pane) = remote.terminals.get(id) else {
            let failed = remote.errors.contains(id);
            let owner = cx.weak_entity();
            let session = id.to_owned();
            let epoch = remote.epoch;
            let style = self.terminal_style(cx);
            let measure = gpui::canvas(
                move |bounds, window, cx| {
                    let font = window.text_system().resolve_font(&style.font);
                    let cell = window
                        .text_system()
                        .em_advance(font, px(style.font_size))
                        .unwrap_or(px(style.font_size * 0.6));
                    if let Some(size) = runner_app::terminal_resize::terminal_grid_size(
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                        f32::from(cell),
                        (style.font_size * crate::terminal::element::LINE_HEIGHT_FACTOR).round(),
                    ) {
                        let owner = owner.clone();
                        let session = session.clone();
                        cx.defer(move |cx| {
                            let _ = owner.update(cx, |root, cx| {
                                if let Some(remote) = root
                                    .remote_client
                                    .workspace
                                    .as_mut()
                                    .filter(|remote| remote.epoch == epoch)
                                {
                                    let measured = (size.cols.max(20), size.rows.max(8));
                                    if remote.sizes.insert(session, measured) != Some(measured) {
                                        root.remote_client.attach_remote_panes(cx);
                                        cx.notify();
                                    }
                                }
                            });
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0();
            if !running {
                return surface.child(measure).into_any_element();
            }
            return surface
                .child(
                    div()
                        .when(failed, |pane| {
                            pane.debug_selector(|| "REMOTE_TERMINAL_FAILED".into())
                        })
                        .relative()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_col()
                        .gap_3()
                        .children(
                            (!failed).then(|| {
                                spinner("remote-terminal-connecting", 24., theme::muted())
                            }),
                        )
                        .child(measure)
                        .child(label(
                            if failed {
                                "client.terminal_failed"
                            } else {
                                "client.terminal_connecting"
                            },
                            &remote.computer.name,
                        )),
                )
                .into_any_element();
        };
        let connected = remote.connection == Connection::Connected;
        let controlled = pane.attachment.terminal.control().desktop();
        let style = self.terminal_style(cx);
        if pane.attachment.terminal.palette() != style.palette {
            pane.attachment.terminal.set_palette(style.palette);
        }
        let terminal = pane.attachment.terminal.clone();
        let key_terminal = terminal.clone();
        let copy_terminal = terminal.clone();
        let paste_terminal = terminal.clone();
        let input = pane.attachment.terminal_input.clone();
        let take = cx.weak_entity();
        let take_id = id.to_owned();
        if !controlled {
            let owner_name =
                match &terminal.control().controller {
                    runner_core::protocol::terminal::TerminalController::Browser {
                        name, ..
                    } if name != "Host desktop" => name.clone(),
                    _ => remote.computer.name.clone(),
                };
            surface = surface.child(
                div()
                    .debug_selector(|| "REMOTE_TAKEN_BACK".into())
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .py_2()
                    .bg(theme::panel())
                    .text_size(theme::text_meta())
                    .child(div().flex_1().min_w_0().child(
                        if terminal.control().controller
                            == runner_core::protocol::terminal::TerminalController::None
                        {
                            crate::i18n::text("client.no_controller").to_string()
                        } else {
                            label("client.taken_back", &owner_name)
                        },
                    ))
                    .child(
                        Button::new(
                            SharedString::from(format!("take-remote-{id}")),
                            crate::i18n::text("client.take_control"),
                        )
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .disabled(!connected)
                        .on_press(move |_, cx| {
                            let _ = take.update(cx, |root, cx| {
                                root.remote_client.take_remote_control(&take_id, cx)
                            });
                        }),
                    ),
            );
        }
        if pane.control_failed {
            surface = surface.child(
                div()
                    .debug_selector(|| "REMOTE_CONTROL_FAILED".into())
                    .px_3()
                    .py_2()
                    .text_size(theme::text_meta())
                    .text_color(theme::danger())
                    .child(crate::i18n::text("client.take_control_failed")),
            );
        }
        let interactive = connected && controlled && running;
        let option_as_meta = self.settings(cx).option_as_meta;
        surface
            .child(
                div().relative().flex_1().min_h_0().min_w_0().child(
                    crate::surfaces::panes::TerminalSurface {
                        terminal,
                        interaction: pane.attachment.terminal_interaction.clone(),
                        input: pane.attachment.terminal_input.clone(),
                        focus: pane.attachment.terminal_focus.clone(),
                        scrollbar: pane.attachment.terminal_scrollbar.clone(),
                        style,
                        interactive,
                        resize_owner: interactive,
                        scrollable: true,
                    }
                    .render()
                    .id(SharedString::from(format!("remote-terminal-{id}")))
                    .opacity(if interactive { 1. } else { 0.45 })
                    .on_key_down(move |event: &KeyDownEvent, _, cx| {
                        if !interactive {
                            cx.stop_propagation();
                            return;
                        }
                        let option_is_meta = option_as_meta.is_meta(cx);
                        let _ = crate::terminal::input::handle_key(
                            &key_terminal,
                            input.read(cx).is_composing(),
                            &event.keystroke,
                            option_is_meta,
                            cx,
                        );
                    })
                    .on_action(move |_: &Copy, _, cx| {
                        if !crate::terminal::input::copy_selection(&copy_terminal, cx) {
                            #[cfg(any(windows, target_os = "linux"))]
                            cx.propagate();
                        }
                    })
                    .on_action(move |_: &Paste, _, cx| {
                        if interactive {
                            let _ = crate::terminal::input::paste_text_from_clipboard(
                                &paste_terminal,
                                cx,
                            );
                        }
                        cx.stop_propagation();
                    }),
                ),
            )
            .into_any_element()
    }

    pub(super) fn take_remote_control(&mut self, id: &str, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        let Some(pane) = remote.terminals.get_mut(id) else {
            return;
        };
        let terminal = pane.attachment.terminal.clone();
        let (cols, rows) = pane
            .attachment
            .terminal_interaction
            .read(cx)
            .pane_size
            .map(|size| (size.cols, size.rows))
            .unwrap_or_else(|| terminal.size());
        let epoch = remote.epoch;
        pane.control_failed = false;
        let taking = terminal.clone();
        let task = cx.background_spawn(async move { taking.take_back(cols, rows) });
        let id = id.to_owned();
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |root, cx| {
                let Some(remote) = root
                    .remote_client
                    .workspace
                    .as_mut()
                    .filter(|remote| remote.epoch == epoch)
                else {
                    return;
                };
                let Some(pane) = remote
                    .terminals
                    .get_mut(&id)
                    .filter(|pane| Arc::ptr_eq(&pane.attachment.terminal, &terminal))
                else {
                    return;
                };
                pane.control_failed = result.is_err();
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn remote_connection_card(&self, cx: &mut Context<NativeRoot>) -> AnyElement {
        let remote = self.workspace.as_ref().unwrap();
        let revoked = remote.connection == Connection::Revoked;
        let forgotten = remote.connection == Connection::Forgotten;
        let pair = revoked || forgotten;
        let retry = cx.weak_entity();
        let close = cx.weak_entity();
        let computer = remote.computer.clone();
        div()
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme::with_alpha(theme::bg(), 0.7))
            .p_6()
            .child(
                div()
                    .debug_selector(|| "REMOTE_CONNECTION_CARD".into())
                    .w_full()
                    .max_w(rems(440. / 16.))
                    .rounded_xl()
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::panel())
                    .p_6()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .text_center()
                    .child(icon(
                        if revoked {
                            "shield-x.svg"
                        } else {
                            "cloud-off.svg"
                        },
                        if revoked {
                            theme::danger()
                        } else {
                            theme::muted()
                        },
                    ))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(theme::text_lead())
                            .child(label(
                                if forgotten {
                                    "client.forgotten_title"
                                } else if revoked {
                                    "client.revoked_title"
                                } else {
                                    "client.unreachable_title"
                                },
                                &computer.name,
                            )),
                    )
                    .child(
                        div()
                            .text_size(theme::text_body())
                            .text_color(theme::muted())
                            .child(label(
                                if forgotten {
                                    "client.forgotten_detail"
                                } else if revoked {
                                    "client.revoked_detail"
                                } else {
                                    "client.unreachable_detail"
                                },
                                &computer.name,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::new(
                                    "remote-reconnect",
                                    crate::i18n::text(if forgotten {
                                        "client.pair"
                                    } else if revoked {
                                        "client.pair_again"
                                    } else {
                                        "client.reconnect"
                                    }),
                                )
                                .on_press(move |window, cx| {
                                    let _ = retry.update(cx, |root, cx| {
                                        if pair {
                                            root.remote_client.open_computer_pair(
                                                computer.clone(),
                                                pairing::Purpose::Open { new_window: false },
                                                window,
                                                cx,
                                            );
                                        } else {
                                            let clients = state(&root.app_store, cx);
                                            clients.update(cx, |clients, cx| {
                                                clients.connections.insert(
                                                    computer.id.clone(),
                                                    Connection::Opening,
                                                );
                                                clients.send(Command::Retry(computer.id.clone()));
                                                cx.notify();
                                            });
                                        }
                                    });
                                }),
                            )
                            .child(
                                Button::new("remote-close-view", crate::i18n::text("ui.close_tab"))
                                    .variant(ButtonVariant::Ghost)
                                    .on_press(move |window, cx| {
                                        let _ = close.update(cx, |root, cx| {
                                            root.return_to_this_computer(window, cx)
                                        });
                                    }),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn remote_mission_workspace(&self) -> Option<Entity<MissionWorkspace>> {
        let remote = self.workspace.as_ref()?;
        matches!(
            remote.active,
            Some(crate::surfaces::remote_client::RemoteItem::Mission(_))
        )
        .then(|| remote.mission_workspace.clone())
    }

    pub(crate) fn refresh_remote_palette(
        &self,
        palette: &Entity<crate::surfaces::command_palette::CommandPaletteState>,
        cx: &mut Context<NativeRoot>,
    ) {
        let data = self.workspace.as_ref().map(|remote| {
            let overview = self
                .app_store
                .read(cx)
                .remote_clients
                .read(cx)
                .overviews
                .get(&remote.computer.id)
                .cloned();
            (overview, remote.front_page.clone())
        });
        palette.update(cx, |palette, cx| palette.set_remote_data(data, cx));
    }
}

pub(crate) fn sidebar_nodes(overview: &Overview) -> Vec<SidebarNode> {
    let nodes = overview.sidebar.clone().unwrap_or_else(|| {
        let mut nodes = overview
            .projects
            .iter()
            .enumerate()
            .map(|(position, project)| SidebarNode {
                kind: SidebarNodeKind::Project,
                id: project.id.clone(),
                parent: None,
                position: position as i64,
                pinned_position: None,
            })
            .collect::<Vec<_>>();
        for (position, tab) in overview.tabs.iter().enumerate() {
            nodes.push(SidebarNode {
                kind: SidebarNodeKind::Tab,
                id: tab.id.clone(),
                parent: tab.project.clone(),
                position: position as i64,
                pinned_position: tab.pinned.then_some(position as i64),
            });
        }
        for (position, mission) in overview.missions.iter().enumerate() {
            let position = (overview.tabs.len() + position) as i64;
            nodes.push(SidebarNode {
                kind: SidebarNodeKind::Mission,
                id: mission.id.clone(),
                parent: mission.project.clone(),
                position,
                pinned_position: mission.pinned.then_some(position),
            });
        }
        nodes
    });
    nodes
}

impl RemoteClient {
    pub(crate) fn layout(&self) -> Option<&PaneLayout> {
        let remote = self.workspace.as_ref()?;
        let Some(Item::Tab(id)) = &remote.active else {
            return None;
        };
        remote
            .resize_draft
            .as_ref()
            .filter(|layout| &layout.id == id)
            .or_else(|| remote.layouts.get(id))
    }
    pub(crate) fn layout_mut(&mut self) -> Option<&mut PaneLayout> {
        let remote = self.workspace.as_mut()?;
        let Some(Item::Tab(id)) = &remote.active else {
            return None;
        };
        remote.layouts.get_mut(id)
    }
    pub(crate) fn pane_bounds_mut(&mut self) -> Option<&mut HashMap<PaneKey, gpui::Size<Pixels>>> {
        self.workspace
            .as_mut()
            .map(|remote| &mut remote.pane_bounds)
    }
    pub(crate) fn resize_split(&mut self, id: &str, ratio: f32, cx: &mut Context<NativeRoot>) {
        let draft = self.layout().cloned();
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        if remote.resize_draft.is_none() {
            remote.resize_draft = draft;
        }
        if let Some(draft) = &mut remote.resize_draft {
            draft.set_split_sizes(id, [ratio, 100. - ratio]);
        }
        remote.resizing_split = Some(id.into());
        cx.notify();
    }
    pub(crate) fn finish_split_resize(&mut self) -> Option<PaneLayout> {
        let remote = self.workspace.as_mut()?;
        remote.resizing_split = None;
        remote.resize_draft.take()
    }
    pub(crate) fn resize_drawer(&mut self, height: f32, cx: &mut Context<NativeRoot>) {
        if let Some(layout) = self.layout_mut() {
            layout.set_drawer_height(height);
        }
        self.workspace.as_mut().unwrap().drawer_resizing = true;
        cx.notify();
    }
    pub(crate) fn finish_drawer_resize(&mut self, cx: &mut Context<NativeRoot>) {
        if self
            .workspace
            .as_mut()
            .is_some_and(|remote| std::mem::take(&mut remote.drawer_resizing))
        {
            cx.notify();
        }
    }
    pub(crate) fn set_sidebar_width(&mut self, width: f32, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        remote.sidebar_width = width;
        remote.mission_workspace.update(cx, |workspace, cx| {
            workspace.set_remote_sidebar_width(width, cx)
        });
        cx.notify();
    }
    pub(crate) fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<NativeRoot>) {
        let Some(remote) = self.workspace.as_mut() else {
            return;
        };
        remote.sidebar_collapsed = collapsed;
        remote.mission_workspace.update(cx, |workspace, cx| {
            workspace.set_sidebar_collapsed(collapsed, cx)
        });
        cx.notify();
    }
    pub(crate) fn focus_session_layout(&mut self, session: &str) -> Option<String> {
        let remote = self.workspace.as_mut()?;
        let tab = remote
            .layouts
            .values()
            .find(|layout| layout.all_session_ids().iter().any(|id| id == session))
            .map(|layout| layout.id.clone())?;
        if let Some(layout) = remote.layouts.get_mut(&tab) {
            if layout.drawer_shells().iter().any(|id| id == session) {
                layout.activate_drawer_shell(session);
            } else {
                layout.focus_session(session);
            }
        }
        Some(tab)
    }
    pub(crate) fn set_error(&mut self, error: String) {
        if let Some(remote) = self.workspace.as_mut() {
            remote.action_error = Some(error);
        }
    }
    pub(crate) fn activate_mission(&mut self, id: String) {
        if let Some(remote) = self.workspace.as_mut() {
            remote.activate_mission = Some(id);
        }
    }
    pub(crate) fn reopen_sessions(
        &mut self,
        target: &crate::surfaces::front_page::FrontPageTarget,
        ids: Vec<String>,
    ) -> bool {
        let Some(remote) = self
            .workspace
            .as_mut()
            .filter(|remote| &remote.front_page.target == target)
        else {
            return false;
        };
        remote.reopen_sessions.extend(ids);
        true
    }
}

impl RemoteClient {
    pub(crate) fn session(
        &self,
        id: &str,
        cx: &App,
    ) -> Option<crate::surfaces::front_page::model::SessionDisplay> {
        let remote = self.workspace.as_ref()?;
        let state = self.app_store.read(cx).remote_clients.read(cx);
        let overview = state.overviews.get(&remote.computer.id)?;
        let session = overview.sessions.get(id)?;
        let pinned = overview.tabs.iter().any(|tab| {
            tab.pinned
                && remote
                    .layouts
                    .get(&tab.id)
                    .is_some_and(|layout| layout.contains_session(id))
        });
        let native_fork = remote.catalog.as_ref().is_some_and(|catalog| {
            catalog.runtimes.iter().any(|runtime| {
                Some(&runtime.name) == session.runtime.as_ref() && runtime.native_fork
            })
        });
        Some(crate::surfaces::front_page::model::SessionDisplay::remote(
            id,
            session,
            pinned,
            native_fork,
            remote.front_page.supports(&["resume"]),
        ))
    }
}

impl RemoteClient {
    pub(crate) fn computer(&self) -> Option<&Computer> {
        self.workspace.as_ref().map(|remote| &remote.computer)
    }
    pub(crate) fn active_item(&self) -> Option<&Item> {
        self.workspace.as_ref()?.active.as_ref()
    }
    pub(crate) fn mission_workspace(&self) -> Option<Entity<MissionWorkspace>> {
        self.workspace
            .as_ref()
            .map(|remote| remote.mission_workspace.clone())
    }
    pub(crate) fn sidebar(&self) -> Option<Entity<Sidebar>> {
        self.workspace.as_ref().map(|remote| remote.sidebar.clone())
    }
    pub(crate) fn tab_layout(&self, id: &str) -> Option<&PaneLayout> {
        self.workspace.as_ref()?.layouts.get(id)
    }
    pub(crate) fn session_tab(&self, session: &str) -> Option<String> {
        self.workspace
            .as_ref()?
            .layouts
            .values()
            .find(|layout| layout.contains_session(session))
            .map(|layout| layout.id.clone())
    }
    pub(crate) fn session_pending(&self, id: &str) -> bool {
        self.workspace
            .as_ref()
            .is_some_and(|remote| remote.actions_pending.contains(&format!("session:{id}")))
    }
    pub(crate) fn pane_bounds(&self) -> Option<&HashMap<PaneKey, gpui::Size<Pixels>>> {
        self.workspace.as_ref().map(|remote| &remote.pane_bounds)
    }
    pub(crate) fn resizing_split(&self) -> Option<Option<&str>> {
        self.workspace
            .as_ref()
            .map(|remote| remote.resizing_split.as_deref())
    }
    pub(crate) fn drawer_resizing(&self) -> Option<bool> {
        self.workspace.as_ref().map(|remote| remote.drawer_resizing)
    }
    pub(crate) fn sidebar_collapsed(&self) -> Option<bool> {
        self.workspace
            .as_ref()
            .map(|remote| remote.sidebar_collapsed)
    }
    pub(crate) fn sidebar_width(&self) -> Option<f32> {
        self.workspace.as_ref().map(|remote| remote.sidebar_width)
    }
    pub(crate) fn mission_form_current(
        &self,
        target: &crate::surfaces::front_page::FrontPageTarget,
        navigation: u64,
        form: u64,
    ) -> bool {
        self.workspace.as_ref().is_some_and(|remote| {
            remote.front_page.target == *target
                && remote.front_page.valid()
                && remote.navigation_generation == navigation
                && remote.form_generation == form
        })
    }
    pub(crate) fn status(
        &self,
        id: &str,
        cx: &App,
    ) -> Option<runner_app::ui::agent_status::StatusPresentation> {
        let remote = self.workspace.as_ref()?;
        let state = self.app_store.read(cx).remote_clients.read(cx);
        let session = state.overviews.get(&remote.computer.id)?.sessions.get(id)?;
        let status = model::agent_status(&session.status);
        let mut display = runner_app::ui::agent_status::StatusPresentation::new(&status);
        display.waiting_since = None;
        display.estimated = false;
        Some(display)
    }
    pub(crate) fn archive_targets(&self, ids: Vec<String>) -> Vec<String> {
        let layouts = self
            .workspace
            .as_ref()
            .map(|remote| remote.layouts.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        crate::surfaces::sidebar::archive_targets_for_chats(ids, &layouts)
    }
}
