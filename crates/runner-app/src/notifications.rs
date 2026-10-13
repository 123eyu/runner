use std::collections::BTreeMap;

use gpui::{App, AppContext, Context, Entity, Global, Subscription, SystemNotification};
use runner_app::ui::agent_status::{StatusKind, StatusPresentation};
use runner_core::protocol::status::{AgentStatus, Lifecycle, WaitReason};
use runner_core::Runtime;

use crate::app_settings::NotificationSettings;
use crate::app_store::{AppStore, StoreRevisions};
use crate::{global_app_store, AppRoute, NativeRoot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reason {
    Input(WaitReason),
    Error(Option<i32>),
    Failed,
    Finished,
}

impl Reason {
    fn label(self) -> String {
        let kind = match self {
            Self::Input(WaitReason::Approval) => StatusKind::Approval,
            Self::Input(WaitReason::Answer) => StatusKind::Answer,
            Self::Input(WaitReason::Unknown) => StatusKind::NeedsYou,
            Self::Error(code) => {
                return code.map_or_else(
                    || "Error".into(),
                    |code| format!("Error · exit code {code}"),
                )
            }
            Self::Failed => StatusKind::ResponseFailed,
            Self::Finished => return "Finished".into(),
        };
        StatusPresentation {
            kind,
            estimated: false,
            exit_code: None,
            outcome: None,
            waiting_since: None,
            detail: None,
        }
        .label()
        .into()
    }

    fn enabled(self, settings: &NotificationSettings) -> bool {
        match self {
            Self::Input(_) => settings.chats.input,
            Self::Finished => settings.chats.finished,
            Self::Error(_) | Self::Failed => settings.chats.failed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Chat(String),
}

#[derive(Clone)]
struct Posted {
    target: Target,
    reason: Reason,
}

#[derive(Default)]
struct Visible {
    app_active: bool,
    chats: Vec<String>,
}

fn target_visible(target: &Target, visible: &Visible) -> bool {
    match target {
        Target::Chat(id) => visible.chats.contains(id),
    }
}

fn detect_status_triggers(
    previous: Option<&BTreeMap<String, AgentStatus>>,
    next: &BTreeMap<String, AgentStatus>,
) -> Vec<(String, Reason)> {
    let Some(previous) = previous else {
        return Vec::new();
    };
    next.iter()
        .filter_map(|(id, status)| {
            let default = AgentStatus::default();
            let old = previous.get(id).unwrap_or(&default);
            let wait = status.observation.interactions.iter().find(|wait| {
                !old.observation
                    .interactions
                    .iter()
                    .any(|old| old.id == wait.id)
            });
            let reason = if let Some(wait) = wait {
                Reason::Input(wait.reason)
            } else if status.lifecycle == Lifecycle::Error && old.lifecycle != Lifecycle::Error {
                Reason::Error(status.exit_code)
            } else if status.failed_since.is_some() && old.failed_since.is_none() {
                Reason::Failed
            } else if status.unread_since.is_some() && old.unread_since.is_none() {
                Reason::Finished
            } else {
                return None;
            };
            Some((id.clone(), reason))
        })
        .collect()
}

fn notification_content(
    tag: &str,
    title: String,
    project: Option<&str>,
    reason: Reason,
) -> SystemNotification {
    let label = reason.label();
    SystemNotification {
        tag: tag.to_owned().into(),
        title: title.into(),
        body: project
            .map_or(label.clone(), |project| format!("{label} · {project}"))
            .into(),
        actions: Vec::new(),
    }
}

fn status_snapshot(store: &AppStore) -> BTreeMap<String, AgentStatus> {
    store
        .session_statuses
        .iter()
        .filter(|(id, _)| {
            store
                .session_details
                .get(*id)
                .is_some_and(|entry| entry.archived_at.is_none())
        })
        .map(|(id, status)| (id.clone(), status.clone()))
        .collect()
}

fn stale_tags(
    posted: &BTreeMap<String, Posted>,
    statuses: &BTreeMap<String, AgentStatus>,
    settings: &NotificationSettings,
    visible: &Visible,
    exists: impl Fn(&Target) -> bool,
) -> Vec<String> {
    posted
        .iter()
        .filter_map(|(tag, posted)| {
            let Target::Chat(id) = &posted.target;
            let holds = statuses.get(id).is_some_and(|status| match posted.reason {
                Reason::Input(_) => !status.observation.interactions.is_empty(),
                Reason::Finished => status.unread_since.is_some(),
                Reason::Failed => status.failed_since.is_some(),
                Reason::Error(_) => status.lifecycle == Lifecycle::Error,
            });
            (!exists(&posted.target)
                || !holds
                || !posted.reason.enabled(settings)
                || target_visible(&posted.target, visible))
            .then(|| tag.clone())
        })
        .collect()
}

fn visible(cx: &mut App) -> Visible {
    for window in cx.windows() {
        let Some(window) = window.downcast::<NativeRoot>() else {
            continue;
        };
        if let Ok(Some(visible)) = window.update(cx, |root, window, _| {
            window.is_window_active().then(|| Visible {
                app_active: true,
                chats: if root.route == AppRoute::Chat {
                    root.tabs
                        .active()
                        .map(|tab| tab.session_ids())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                },
            })
        }) {
            return visible;
        }
    }
    Visible::default()
}

struct NotificationController {
    store: Entity<AppStore>,
    revisions: StoreRevisions,
    previous: Option<BTreeMap<String, AgentStatus>>,
    posted: BTreeMap<String, Posted>,
    _subscription: Subscription,
}

struct GlobalNotifications(Entity<NotificationController>);
impl Global for GlobalNotifications {}

impl NotificationController {
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let store = self.store.read(cx);
        let revisions = store.revisions;
        if self.revisions == revisions {
            return;
        }
        let statuses_changed = self.revisions.activity != revisions.activity
            || self.revisions.missions != revisions.missions;
        let relevant = statuses_changed
            || self.revisions.sessions != revisions.sessions
            || self.revisions.settings != revisions.settings;
        self.revisions = revisions;
        if !relevant {
            return;
        }
        let triggers = if statuses_changed {
            let next = status_snapshot(store);
            let triggers = detect_status_triggers(self.previous.as_ref(), &next);
            self.previous = Some(next);
            triggers
        } else {
            Vec::new()
        };
        let visible = visible(cx);
        self.withdraw(&visible, cx);
        for (id, reason) in triggers {
            let store = self.store.read(cx);
            let Some(entry) = store
                .session_details
                .get(&id)
                .filter(|entry| entry.archived_at.is_none())
            else {
                continue;
            };
            let target = Target::Chat(id.clone());
            if Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell)
                || !reason.enabled(&store.settings.notifications)
                || visible.app_active
            {
                continue;
            }
            let title = crate::surfaces::session_label_live(
                entry,
                store
                    .bridge
                    .session(&id)
                    .map(|terminal| terminal.title())
                    .as_deref(),
            );
            let project = entry
                .project_id
                .as_ref()
                .and_then(|id| store.projects.iter().find(|project| &project.id == id))
                .map(|project| project.name.as_str());
            cx.show_system_notification(notification_content(&id, title, project, reason));
            self.posted.insert(id, Posted { target, reason });
        }
    }

    fn withdraw(&mut self, visible: &Visible, cx: &mut Context<Self>) {
        let store = self.store.read(cx);
        let stale = stale_tags(
            &self.posted,
            &status_snapshot(store),
            &store.settings.notifications,
            visible,
            |target| {
                let Target::Chat(id) = target;
                store
                    .session_details
                    .get(id)
                    .is_some_and(|entry| entry.archived_at.is_none())
            },
        );
        for tag in stale {
            cx.dismiss_system_notification(&tag);
            self.posted.remove(&tag);
        }
    }
}

pub(crate) fn install(cx: &mut App) {
    let store = global_app_store(cx);
    let controller = cx.new(|cx: &mut Context<NotificationController>| {
        let snapshot = store.read(cx);
        NotificationController {
            revisions: snapshot.revisions,
            previous: Some(status_snapshot(snapshot)),
            posted: BTreeMap::new(),
            _subscription: cx.observe(&store, |this, _, cx| this.refresh(cx)),
            store,
        }
    });
    cx.set_global(GlobalNotifications(controller));
    cx.on_system_notification_response(|response, cx| open(response.tag.as_ref(), cx));
}

pub(crate) fn view_changed(cx: &mut App) {
    cx.defer(|cx| {
        let Some(controller) = cx
            .try_global::<GlobalNotifications>()
            .map(|global| global.0.clone())
        else {
            return;
        };
        let visible = visible(cx);
        controller.update(cx, |this, cx| this.withdraw(&visible, cx));
    });
}

pub(crate) fn dismiss_all(cx: &mut App) {
    let Some(controller) = cx
        .try_global::<GlobalNotifications>()
        .map(|global| global.0.clone())
    else {
        return;
    };
    controller.update(cx, |this, cx| {
        for (tag, _) in std::mem::take(&mut this.posted) {
            cx.dismiss_system_notification(&tag);
        }
    });
}

pub(crate) fn open(tag: &str, cx: &mut App) {
    let target = cx
        .try_global::<GlobalNotifications>()
        .and_then(|global| global.0.read(cx).posted.get(tag))
        .map(|posted| posted.target.clone())
        .or_else(|| {
            global_app_store(cx)
                .read(cx)
                .session_details
                .contains_key(tag)
                .then(|| Target::Chat(tag.into()))
        });
    let mut windows = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<NativeRoot>())
        .collect::<Vec<_>>();
    if windows.is_empty() {
        crate::handle_reopen(cx);
        windows = cx
            .windows()
            .into_iter()
            .filter_map(|window| window.downcast::<NativeRoot>())
            .collect();
    }
    let entries = global_app_store(cx).read(cx).window_entries.clone();
    let displaying = target.as_ref().and_then(|target| {
        windows
            .iter()
            .copied()
            .filter(|window| {
                window.read(cx).is_ok_and(|root| match target {
                    Target::Chat(id) => {
                        root.route == AppRoute::Chat
                            && root
                                .tabs
                                .active()
                                .is_some_and(|tab| tab.session_ids().contains(id))
                    }
                })
            })
            .max_by_key(|window| {
                window
                    .read(cx)
                    .ok()
                    .and_then(|root| {
                        entries
                            .iter()
                            .find(|entry| entry.label == root.window_label)
                    })
                    .map(|entry| entry.focused_at)
            })
    });
    let holding = target.as_ref().and_then(|target| {
        windows.iter().copied().find(|window| {
            window.read(cx).is_ok_and(|root| match target {
                Target::Chat(id) => root
                    .tabs
                    .tabs()
                    .iter()
                    .any(|tab| tab.session_ids().contains(id)),
            })
        })
    });
    let window = displaying
        .or(holding)
        .or_else(|| {
            cx.active_window()
                .and_then(|window| window.downcast::<NativeRoot>())
        })
        .or_else(|| windows.first().copied());
    if let Some(window) = window {
        let _ = window.update(cx, |root, window, cx| {
            window.activate_window();
            if let Some(Target::Chat(id)) = target {
                if !root.open_chat_session(&id, window, cx) {
                    match root.reload_tabs(cx) {
                        Ok(()) => {
                            root.open_chat_session(&id, window, cx);
                        }
                        Err(error) => root.chat_error = Some(error.to_string()),
                    }
                }
            }
        });
    }
    cx.activate(true);
    view_changed(cx);
}

#[cfg(test)]
mod tests;
