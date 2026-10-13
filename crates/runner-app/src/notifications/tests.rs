use super::*;
use runner_core::protocol::status::HumanInteraction;

fn wait(reason: WaitReason, id: &str) -> AgentStatus {
    AgentStatus {
        lifecycle: Lifecycle::Running,
        observation: runner_core::protocol::status::AgentObservation {
            interactions: vec![HumanInteraction {
                id: id.into(),
                reason,
                owners: Vec::new(),
                since: 1,
            }],
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn status_edges_baseline_new_sessions_repeats_and_priority() {
    let empty = BTreeMap::new();
    for reason in [
        WaitReason::Approval,
        WaitReason::Answer,
        WaitReason::Unknown,
    ] {
        let next = BTreeMap::from([("chat".into(), wait(reason, "first"))]);
        assert!(detect_status_triggers(None, &next).is_empty());
        assert_eq!(
            detect_status_triggers(Some(&empty), &next),
            [("chat".into(), Reason::Input(reason))]
        );
        assert!(detect_status_triggers(Some(&next), &next).is_empty());
        let newer = BTreeMap::from([("chat".into(), wait(reason, "second"))]);
        assert_eq!(
            detect_status_triggers(Some(&next), &newer),
            [("chat".into(), Reason::Input(reason))]
        );
    }
    let cases = [
        (
            AgentStatus {
                unread_since: Some(1),
                ..Default::default()
            },
            Reason::Finished,
        ),
        (
            AgentStatus {
                failed_since: Some(1),
                unread_since: Some(1),
                ..Default::default()
            },
            Reason::Failed,
        ),
        (
            AgentStatus {
                lifecycle: Lifecycle::Error,
                exit_code: Some(2),
                failed_since: Some(1),
                unread_since: Some(1),
                ..Default::default()
            },
            Reason::Error(Some(2)),
        ),
    ];
    for (status, reason) in cases {
        let next = BTreeMap::from([("chat".into(), status)]);
        assert_eq!(
            detect_status_triggers(Some(&empty), &next),
            [("chat".into(), reason)]
        );
        assert!(detect_status_triggers(Some(&next), &next).is_empty());
    }
    let mut priority = wait(WaitReason::Approval, "new");
    priority.lifecycle = Lifecycle::Error;
    priority.failed_since = Some(1);
    priority.unread_since = Some(1);
    assert_eq!(
        detect_status_triggers(Some(&empty), &BTreeMap::from([("chat".into(), priority)])),
        [("chat".into(), Reason::Input(WaitReason::Approval))]
    );
}

#[test]
fn content_matches_each_direct_chat_trigger() {
    for (reason, text) in [
        (Reason::Input(WaitReason::Approval), "Approval needed"),
        (Reason::Input(WaitReason::Answer), "Answer needed"),
        (Reason::Input(WaitReason::Unknown), "Needs you"),
        (Reason::Finished, "Finished"),
        (Reason::Failed, "Response failed"),
        (Reason::Error(Some(9)), "Error · exit code 9"),
        (Reason::Error(None), "Error"),
    ] {
        let content = notification_content("chat", "Fix the test".into(), Some("runner"), reason);
        assert_eq!(content.tag.as_ref(), "chat");
        assert_eq!(content.title.as_ref(), "Fix the test");
        assert_eq!(content.body.as_ref(), format!("{text} · runner"));
        assert!(content.actions.is_empty());
        assert_eq!(
            notification_content("chat", "Title".into(), None, reason)
                .body
                .as_ref(),
            text
        );
    }
}

#[test]
fn switches_and_visibility_include_every_pane_of_the_active_tab() {
    let mut settings = NotificationSettings::default();
    settings.chats.input = false;
    settings.chats.finished = false;
    settings.chats.failed = false;
    for reason in [
        Reason::Input(WaitReason::Approval),
        Reason::Input(WaitReason::Answer),
        Reason::Input(WaitReason::Unknown),
        Reason::Finished,
        Reason::Failed,
        Reason::Error(None),
    ] {
        assert!(!reason.enabled(&settings));
    }
    let visible = Visible {
        chats: vec!["focused".into(), "unfocused".into()],
        ..Default::default()
    };
    assert!(target_visible(&Target::Chat("unfocused".into()), &visible));
    assert!(!target_visible(
        &Target::Chat("another-tab".into()),
        &visible
    ));
    assert!(!target_visible(
        &Target::Chat("focused".into()),
        &Visible::default()
    ));
}

#[test]
fn every_withdrawal_rule() {
    let settings = NotificationSettings::default();
    for (reason, status) in [
        (
            Reason::Input(WaitReason::Approval),
            wait(WaitReason::Approval, "wait"),
        ),
        (
            Reason::Finished,
            AgentStatus {
                unread_since: Some(1),
                ..Default::default()
            },
        ),
        (
            Reason::Failed,
            AgentStatus {
                failed_since: Some(1),
                ..Default::default()
            },
        ),
        (
            Reason::Error(None),
            AgentStatus {
                lifecycle: Lifecycle::Error,
                ..Default::default()
            },
        ),
    ] {
        let posted = BTreeMap::from([(
            "chat".into(),
            Posted {
                target: Target::Chat("chat".into()),
                reason,
            },
        )]);
        let statuses = BTreeMap::from([("chat".into(), status)]);
        assert!(
            stale_tags(&posted, &statuses, &settings, &Visible::default(), |_| true).is_empty()
        );
        assert_eq!(
            stale_tags(
                &posted,
                &BTreeMap::new(),
                &settings,
                &Visible::default(),
                |_| true
            ),
            ["chat"]
        );
        assert_eq!(
            stale_tags(
                &posted,
                &BTreeMap::from([("chat".into(), AgentStatus::default())]),
                &settings,
                &Visible::default(),
                |_| true
            ),
            ["chat"]
        );
        assert_eq!(
            stale_tags(&posted, &statuses, &settings, &Visible::default(), |_| {
                false
            }),
            ["chat"]
        );
        assert_eq!(
            stale_tags(
                &posted,
                &statuses,
                &settings,
                &Visible {
                    chats: vec!["chat".into()],
                    ..Default::default()
                },
                |_| true
            ),
            ["chat"]
        );
        let off = NotificationSettings {
            chats: crate::app_settings::ChatNotifications {
                input: false,
                finished: false,
                failed: false,
            },
            ..Default::default()
        };
        assert_eq!(
            stale_tags(&posted, &statuses, &off, &Visible::default(), |_| true),
            ["chat"]
        );
    }
}

#[cfg(unix)]
#[test]
fn controller_posts_once_replaces_clears_archives_switches_and_quits() {
    let temp = tempfile::tempdir().unwrap();
    let mut cx = gpui::TestAppContext::single();
    let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
    store.update(&mut cx, |store, _| {
        crate::app_store::seed_mixed_working_sessions(store)
    });
    cx.update(|cx| {
        cx.set_app_identity("com.wycstudios.runner-dev", "Runner");
        install(cx);
    });
    assert!(cx.shown_system_notifications().is_empty());
    let change = |cx: &mut gpui::TestAppContext, state: AgentStatus| {
        store.update(cx, |store, cx| {
            store
                .session_statuses
                .insert("direct-agent".into(), state.clone());
            store.session_statuses.insert("busy-shell".into(), state);
            store.revisions.activity += 1;
            cx.notify();
        });
        cx.update(|_| ());
    };
    change(&mut cx, wait(WaitReason::Approval, "first"));
    assert_eq!(cx.shown_system_notifications().len(), 1);
    change(&mut cx, wait(WaitReason::Approval, "first"));
    assert_eq!(cx.shown_system_notifications().len(), 1);
    change(&mut cx, wait(WaitReason::Answer, "second"));
    assert_eq!(cx.shown_system_notifications().len(), 2);
    assert_eq!(cx.delivered_system_notifications().len(), 1);
    change(&mut cx, AgentStatus::default());
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
    change(&mut cx, wait(WaitReason::Unknown, "third"));
    store.update(&mut cx, |store, cx| {
        store.settings.notifications.chats.input = false;
        store.revisions.settings += 1;
        cx.notify();
    });
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
    change(&mut cx, wait(WaitReason::Unknown, "fourth"));
    assert_eq!(cx.shown_system_notifications().len(), 3);
    store.update(&mut cx, |store, cx| {
        store.settings.notifications.chats.input = true;
        store.revisions.settings += 1;
        cx.notify();
    });
    change(&mut cx, wait(WaitReason::Unknown, "fifth"));
    cx.update(dismiss_all);
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
    change(&mut cx, wait(WaitReason::Unknown, "sixth"));
    store.update(&mut cx, |store, cx| {
        store.session_details.remove("direct-agent");
        store.revisions.sessions += 1;
        cx.notify();
    });
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
}

#[cfg(unix)]
#[test]
fn click_activates_the_window_tab_and_pane_and_view_changes_defer() {
    use gpui::{SystemNotificationResponse, VisualTestContext};
    use runner_app::pane_layout::{PaneLayout, TabSet};
    use runner_core::protocol::node::{NodeRow, NodeType};

    let temp = tempfile::tempdir().unwrap();
    let mut cx = gpui::TestAppContext::single();
    let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
    store.update(&mut cx, |store, _| {
        crate::app_store::seed_mixed_working_sessions(store)
    });
    let rows = ["direct-agent", "busy-shell"].map(|id| NodeRow {
        id: if id == "direct-agent" {
            "01ARZ3NDEKTSV4RRFFQ69G5FAV"
        } else {
            "01ARZ3NDEKTSV4RRFFQ69G5FAW"
        }
        .into(),
        parent_id: None,
        position: 0,
        node_type: NodeType::Tab,
        name: None,
        ref_id: None,
        layout: Some(PaneLayout::single(Some(id), &[]).serialize().unwrap()),
        pinned_position: None,
        last_completed_at: None,
        last_viewed_at: None,
        created_at: String::new(),
    });
    let client = store.read_with(&cx, |store, _| store.client.clone());
    for row in &rows {
        client
            .node_tab_upsert(runner_core::protocol::node::NodeTabUpsertInput {
                id: row.id.clone(),
                parent_id: None,
                name: String::new(),
                layout: row.layout.clone().unwrap(),
            })
            .unwrap();
    }
    store.update(&mut cx, |store, _| store.nodes = rows.to_vec());
    cx.update(|cx| {
        cx.set_app_identity("com.wycstudios.runner-dev", "Runner");
        install(cx);
    });
    let host = cx.add_window(|window, cx| {
        let mut root = NativeRoot::new(
            "main".into(),
            temp.path().join("logs"),
            None,
            None,
            store.clone(),
            window,
            cx,
        );
        root.tabs = TabSet::from_rows(&rows);
        root.tabs.activate(&rows[1].id);
        root.route = AppRoute::Chat;
        root
    });
    let mut visual = VisualTestContext::from_window(host.into(), &cx);
    visual.update(|window, _| window.activate_window());
    cx.run_until_parked();
    visual.deactivate_window();
    cx.run_until_parked();
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            store
                .session_statuses
                .insert("direct-agent".into(), wait(WaitReason::Approval, "new"));
            store.revisions.activity += 1;
            cx.notify();
        })
    });
    assert_eq!(cx.delivered_system_notifications().len(), 1);
    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: "direct-agent".into(),
        action_id: None,
    });
    cx.run_until_parked();
    host.read_with(&cx, |root, _| {
        assert_eq!(root.route, AppRoute::Chat);
        assert_eq!(root.tabs.active_tab_id(), Some(rows[0].id.as_str()));
        assert_eq!(
            root.tabs.active().unwrap().focused_session_id(),
            Some("direct-agent")
        );
    })
    .unwrap();
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
    host.update(&mut visual, |root, _, cx| root.report_current_subjects(cx))
        .unwrap();
    cx.update(|_| ());
    assert!(cx.delivered_system_notifications().is_empty());
    host.update(&mut visual, |root, _, _| {
        root.tabs.activate(&rows[1].id);
    })
    .unwrap();
    let secondary = cx.add_window(|window, cx| {
        let mut root = NativeRoot::new(
            "secondary".into(),
            temp.path().join("logs"),
            None,
            None,
            store.clone(),
            window,
            cx,
        );
        root.tabs = TabSet::from_rows(&rows);
        root.tabs.activate(&rows[0].id);
        root.route = AppRoute::Chat;
        root
    });
    visual.update(|window, _| window.activate_window());
    cx.run_until_parked();
    visual.deactivate_window();
    cx.run_until_parked();
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            store.session_statuses.insert(
                "direct-agent".into(),
                wait(WaitReason::Approval, "second-window"),
            );
            store.revisions.activity += 1;
            cx.notify();
        })
    });
    assert_eq!(cx.delivered_system_notifications().len(), 1);
    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: "direct-agent".into(),
        action_id: None,
    });
    cx.run_until_parked();
    secondary
        .read_with(&cx, |root, _| {
            assert_eq!(root.tabs.active_tab_id(), Some(rows[0].id.as_str()))
        })
        .unwrap();
    host.read_with(&cx, |root, _| {
        assert_eq!(root.tabs.active_tab_id(), Some(rows[1].id.as_str()))
    })
    .unwrap();
    assert_eq!(
        cx.read(|cx| cx.active_window().unwrap().window_id()),
        secondary.window_id()
    );
}

#[cfg(unix)]
#[test]
fn controller_accepts_an_empty_loaded_store_as_the_baseline() {
    let temp = tempfile::tempdir().unwrap();
    let mut cx = gpui::TestAppContext::single();
    let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
    assert!(store.read_with(&cx, |store, _| store.session_statuses.is_empty()));
    cx.update(|cx| {
        cx.set_app_identity("com.wycstudios.runner-dev", "Runner");
        install(cx);
    });
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            crate::app_store::seed_mixed_working_sessions(store);
            store.session_statuses.insert(
                "direct-agent".into(),
                wait(WaitReason::Approval, "first-session"),
            );
            store.revisions.activity += 1;
            cx.notify();
        })
    });
    assert_eq!(cx.shown_system_notifications().len(), 1);
    assert_eq!(
        cx.shown_system_notifications()[0].tag.as_ref(),
        "direct-agent"
    );
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            store.revisions.activity += 1;
            cx.notify();
        })
    });
    assert_eq!(cx.shown_system_notifications().len(), 1);
}

#[cfg(unix)]
#[test]
fn chat_finished_defaults_on_and_reenabling_does_not_replay() {
    let temp = tempfile::tempdir().unwrap();
    let mut cx = gpui::TestAppContext::single();
    let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
    store.update(&mut cx, |store, _| {
        crate::app_store::seed_mixed_working_sessions(store)
    });
    cx.update(|cx| {
        cx.set_app_identity("com.wycstudios.runner-dev", "Runner");
        install(cx);
    });
    let change = |cx: &mut gpui::TestAppContext, unread| {
        cx.update(|cx| {
            store.update(cx, |store, cx| {
                store.session_statuses.insert(
                    "direct-agent".into(),
                    AgentStatus {
                        unread_since: unread,
                        ..Default::default()
                    },
                );
                store.revisions.activity += 1;
                cx.notify();
            })
        });
    };
    change(&mut cx, Some(1));
    assert_eq!(cx.delivered_system_notifications().len(), 1);
    assert_eq!(
        cx.delivered_system_notifications()[0].body.as_ref(),
        "Finished"
    );
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            store.settings.notifications.chats.finished = false;
            store.revisions.settings += 1;
            cx.notify();
        })
    });
    assert!(cx.delivered_system_notifications().is_empty());
    change(&mut cx, None);
    change(&mut cx, Some(2));
    assert_eq!(cx.shown_system_notifications().len(), 1);
    assert!(cx.delivered_system_notifications().is_empty());
    cx.update(|cx| {
        store.update(cx, |store, cx| {
            store.settings.notifications.chats.finished = true;
            store.revisions.settings += 1;
            cx.notify();
        })
    });
    change(&mut cx, Some(2));
    assert_eq!(cx.shown_system_notifications().len(), 1);
    assert!(cx.delivered_system_notifications().is_empty());
    change(&mut cx, None);
    change(&mut cx, Some(3));
    assert_eq!(cx.shown_system_notifications().len(), 2);
    assert_eq!(cx.delivered_system_notifications().len(), 1);
    assert_eq!(
        cx.delivered_system_notifications()[0].body.as_ref(),
        "Finished"
    );
}

#[cfg(unix)]
#[test]
fn foreground_routes_suppress_all_kinds_without_background_replay() {
    use gpui::VisualTestContext;
    let temp = tempfile::tempdir().unwrap();
    let mut cx = gpui::TestAppContext::single();
    let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
    store.update(&mut cx, |store, _| {
        crate::app_store::seed_mixed_working_sessions(store)
    });
    cx.update(|cx| {
        cx.set_app_identity("com.wycstudios.runner-dev", "Runner");
        install(cx);
    });
    let _host = cx.add_window(|window, cx| {
        NativeRoot::new(
            "main".into(),
            temp.path().join("logs"),
            None,
            None,
            store.clone(),
            window,
            cx,
        )
    });
    let secondary = cx.add_window(|window, cx| {
        NativeRoot::new(
            "secondary".into(),
            temp.path().join("logs"),
            None,
            None,
            store.clone(),
            window,
            cx,
        )
    });
    let mut visual = VisualTestContext::from_window(secondary.into(), &cx);
    visual.update(|window, _| window.activate_window());
    cx.run_until_parked();
    let change = |cx: &mut gpui::TestAppContext, state| {
        cx.update(|cx| {
            store.update(cx, |store, cx| {
                store.session_statuses.insert("direct-agent".into(), state);
                store.revisions.activity += 1;
                cx.notify();
            })
        });
    };
    for (index, route) in [
        AppRoute::Chat,
        AppRoute::Settings,
        AppRoute::Mission("other".into()),
    ]
    .into_iter()
    .enumerate()
    {
        secondary
            .update(&mut visual, |root, _, _| root.route = route)
            .unwrap();
        change(&mut cx, AgentStatus::default());
        change(
            &mut cx,
            wait(WaitReason::Approval, &format!("foreground-{index}")),
        );
        change(
            &mut cx,
            AgentStatus {
                lifecycle: Lifecycle::Error,
                ..Default::default()
            },
        );
        change(
            &mut cx,
            AgentStatus {
                failed_since: Some(1),
                ..Default::default()
            },
        );
        change(
            &mut cx,
            AgentStatus {
                unread_since: Some(1),
                ..Default::default()
            },
        );
    }
    assert!(cx.shown_system_notifications().is_empty());
    visual.deactivate_window();
    cx.run_until_parked();
    cx.update(view_changed);
    assert!(cx.shown_system_notifications().is_empty());
    change(&mut cx, AgentStatus::default());
    change(&mut cx, wait(WaitReason::Answer, "fresh-background"));
    assert_eq!(cx.delivered_system_notifications().len(), 1);
}
