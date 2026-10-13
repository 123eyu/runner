use super::*;
use crate::AppCore;
use runner_core::protocol::mission::MissionAddRoleInput;

#[test]
fn added_slot_records_overrides_resumes_restarts_and_removal_stops_it() {
    let data = tempfile::tempdir().unwrap();
    let fake = fake_runtime();
    let manager = mgr_with_fake(None, fake.clone());
    let mut core = crate::test_support::test_core_in(data.path().into());
    core.sessions = manager.clone();
    {
        let conn = core.db.get().unwrap();
        conn.execute_batch("INSERT INTO crews (id,name,created_at,updated_at) VALUES ('c','Team','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z'); INSERT INTO missions (id,crew_id,title,status,started_at) VALUES ('m','c','Test mission','running','2026-10-01T00:00:00Z');").unwrap();
        crate::test_support::insert_test_role(&conn, "lead", "lead", "shell", "fixture-shell");
        crate::test_support::insert_test_slot(&conn, "lead", "c", "lead", "lead", 0, true);
        let mut lead_row = crate::test_support::test_session_row(
            "lead-session",
            crate::model::SessionStatus::Stopped,
        );
        lead_row.mission_id = Some("m".into());
        lead_row.slot_id = Some("lead".into());
        lead_row.role_id = Some("lead".into());
        crate::test_support::insert_session_row(&conn, &lead_row).unwrap();
        crate::test_support::insert_test_role(&conn, "added", "coder", "codex", "fixture-codex");
        conn.execute("UPDATE roles SET system_prompt='ORIGINAL_BRIEF',model='role-model',effort='high' WHERE id='added'",[]).unwrap();
    }
    let change = crate::ops::mission::roles::add_role(
        &core,
        MissionAddRoleInput {
            mission_id: "m".into(),
            role_id: "added".into(),
            handle: None,
            runtime: Some(Runtime::Codex),
            model: Some("override-model".into()),
            effort: Some("xhigh".into()),
            speed: Some(crate::model::CodexSpeed::Fast),
            task: Some("small task".into()),
        },
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while manager.has_pending_mission_spawns("m") && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!manager.has_pending_mission_spawns("m"));
    assert!(manager.live_session_ids().contains(&change.session_id));
    let initial = crate::repo::session::get_row(&core.db.get().unwrap(), &change.session_id)
        .unwrap()
        .unwrap();
    let definition = initial.launch_definition.unwrap();
    assert_eq!(definition.model.as_deref(), Some("override-model"));
    assert_eq!(definition.effort.as_deref(), Some("xhigh"));
    assert_eq!(definition.codex_speed, Some(crate::model::CodexSpeed::Fast));
    let log = open_mission_event_log(data.path(), "c", "m").unwrap();
    let (entries, _) = log.read_from_lossy(0).unwrap();
    let joined = entries
        .iter()
        .find(|entry| {
            entry
                .event
                .signal_type
                .as_ref()
                .is_some_and(|kind| kind.as_str() == "role_joined")
        })
        .unwrap();
    assert_eq!(
        joined.event.payload["model"].as_str(),
        definition.model.as_deref()
    );
    assert_eq!(
        joined.event.payload["effort"].as_str(),
        definition.effort.as_deref()
    );
    assert_eq!(
        joined.event.payload["speed"],
        serde_json::to_value(definition.codex_speed).unwrap()
    );
    assert!(fake
        .last_spawn_spec()
        .unwrap()
        .args
        .iter()
        .any(|arg| arg.contains("ORIGINAL_BRIEF") && arg.contains("team can change")));
    let roster = crate::ops::mission::mission_roster(&core, "m").unwrap();
    let slot = &roster
        .iter()
        .find(|member| member.slot.slot_handle == change.handle)
        .unwrap()
        .slot;
    assert_eq!(slot.model_override.as_deref(), Some("override-model"));
    assert_eq!(slot.runtime_override.as_deref(), Some("codex"));
    core.db.get().unwrap().execute("UPDATE roles SET system_prompt='EDITED_BRIEF',model='edited-model',effort='medium' WHERE id='added'",[]).unwrap();
    manager.kill(&change.session_id).unwrap();
    crate::ops::session::session_resume(&core, &change.session_id, None, None).unwrap();
    let resumed = crate::repo::session::get_row(&core.db.get().unwrap(), &change.session_id)
        .unwrap()
        .unwrap();
    assert_eq!(resumed.launch_definition.unwrap(), definition);
    crate::ops::session::session_restart(&core, &change.session_id, None, None).unwrap();
    let restarted = crate::repo::session::get_row(&core.db.get().unwrap(), &change.session_id)
        .unwrap()
        .unwrap();
    let definition = restarted.launch_definition.unwrap();
    assert_eq!(definition.system_prompt.as_deref(), Some("EDITED_BRIEF"));
    assert_eq!(definition.model.as_deref(), Some("override-model"));
    assert_eq!(definition.effort.as_deref(), Some("xhigh"));
    assert_eq!(definition.codex_speed, Some(crate::model::CodexSpeed::Fast));
    let count = fake.spawn_count();
    crate::ops::mission::roles::remove_role(&core, "m", &change.handle).unwrap();
    assert!(manager.live_session_ids().is_empty());
    let removed = crate::repo::session::get_row(&core.db.get().unwrap(), &change.session_id)
        .unwrap()
        .unwrap();
    assert!(removed.archived_at.is_none());
    assert_eq!(removed.status, crate::model::SessionStatus::Stopped);
    let error = crate::ops::session::session_resume(&core, &change.session_id, None, None)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("@coder was removed from this mission"),
        "{error}"
    );
    assert_eq!(fake.spawn_count(), count);
    core.buses.unmount("m");
    core.routers.unregister("m");
    crate::daemon::block_on(crate::ops::mission::ensure_mission_router_mounted(
        &core, "m",
    ))
    .unwrap();
    let listed = crate::ops::session::session_list(&core, "m").unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].session.id, "lead-session");
    let resumed = crate::ops::mission::mission_resume(&core, "m").unwrap();
    assert_eq!(resumed.resumed_session_ids, vec!["lead-session"]);
    assert_eq!(fake.spawn_count(), count + 1);
    manager.kill("lead-session").unwrap();
    let error = crate::ops::role::delete(&mut core.db.get().unwrap(), "added")
        .unwrap_err()
        .to_string();
    assert!(error.contains("unarchived missions"), "{error}");
    core.buses.unmount("m");
    core.routers.unregister("m");
}

fn removal_race_fixture() -> (tempfile::TempDir, AppCore, Arc<FakeRuntime>) {
    let data = tempfile::tempdir().unwrap();
    let fake = fake_runtime();
    let mut core = crate::test_support::test_core_in(data.path().into());
    core.sessions = mgr_with_fake(None, fake.clone());
    {
        let conn = core.db.get().unwrap();
        conn.execute_batch("INSERT INTO crews (id,name,created_at,updated_at) VALUES ('c','Team','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z'); INSERT INTO missions (id,crew_id,title,status,started_at) VALUES ('m','c','Test mission','running','2026-10-01T00:00:00Z');").unwrap();
        for (handle, lead) in [("lead", true), ("worker", false)] {
            crate::test_support::insert_test_role(&conn, handle, handle, "shell", "fixture-shell");
            crate::test_support::insert_test_slot(
                &conn,
                handle,
                "c",
                handle,
                handle,
                i64::from(!lead),
                lead,
            );
            let mut row =
                crate::test_support::test_session_row(handle, crate::model::SessionStatus::Stopped);
            row.mission_id = Some("m".into());
            row.slot_id = Some(handle.into());
            row.role_id = Some(handle.into());
            crate::test_support::insert_session_row(&conn, &row).unwrap();
        }
    }
    crate::daemon::block_on(crate::ops::mission::ensure_mission_router_mounted(
        &core, "m",
    ))
    .unwrap();
    (data, core, fake)
}

fn run_worker_lifecycle(core: &AppCore, restart: bool) -> Result<SpawnedSession> {
    if restart {
        crate::ops::session::session_restart(core, "worker", None, None)
    } else {
        crate::ops::session::session_resume(core, "worker", None, None)
    }
}

#[test]
fn removal_refuses_resume_and_restart_during_spawn_then_stops_the_completed_spawn() {
    for restart in [false, true] {
        let (_data, core, fake) = removal_race_fixture();
        let (entered_tx, entered) = std::sync::mpsc::channel();
        let (release, release_rx) = std::sync::mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        *fake.spawn_hook.lock().unwrap() = Some(Box::new(move || {
            entered_tx.send(()).unwrap();
            release_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
        }));
        let removal = thread::scope(|scope| {
            let task = scope.spawn(|| run_worker_lifecycle(&core, restart));
            entered.recv_timeout(Duration::from_secs(5)).unwrap();
            let removal = crate::ops::mission::roles::remove_role(&core, "m", "worker");
            release.send(()).unwrap();
            task.join().unwrap().unwrap();
            removal
        });
        let error = removal.unwrap_err().to_string();
        assert!(
            error.contains("@worker is being resumed or restarted"),
            "{error}"
        );
        assert!(core.sessions.live_session_ids().contains(&"worker".into()));
        assert!(crate::ops::mission::mission_roster(&core, "m")
            .unwrap()
            .iter()
            .any(|member| member.slot.slot_handle == "worker"));
        assert!(fake.stops.lock().unwrap().is_empty());
        *fake.spawn_hook.lock().unwrap() = None;
        crate::ops::mission::roles::remove_role(&core, "m", "worker").unwrap();
        assert!(core.sessions.live_session_ids().is_empty());
        for fresh in [false, true] {
            let error = run_worker_lifecycle(&core, fresh).unwrap_err().to_string();
            assert!(
                error.contains("@worker was removed from this mission"),
                "{error}"
            );
        }
        assert_eq!(fake.spawn_count(), 1);
        core.buses.unmount("m");
        core.routers.unregister("m");
    }
}

#[test]
fn removal_claim_excludes_resume_and_restart_from_stop_through_publication() {
    let (_data, core, fake) = removal_race_fixture();
    run_worker_lifecycle(&core, false).unwrap();
    let (entered, release) = fake.arm_stop_gate();
    let errors = thread::scope(|scope| {
        let removal = scope.spawn(|| crate::ops::mission::roles::remove_role(&core, "m", "worker"));
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        let results = [
            run_worker_lifecycle(&core, false),
            run_worker_lifecycle(&core, true),
        ];
        release.send(()).unwrap();
        removal.join().unwrap().unwrap();
        results
    });
    for result in errors {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is already being resumed"));
    }
    assert!(core.sessions.live_session_ids().is_empty());
    assert_eq!(
        crate::ops::mission::mission_roster(&core, "m")
            .unwrap()
            .len(),
        1
    );
    for restart in [false, true] {
        let error = run_worker_lifecycle(&core, restart)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("@worker was removed from this mission"),
            "{error}"
        );
    }
    assert_eq!(fake.spawn_count(), 1);
    let stops = fake.stops.lock().unwrap();
    assert!(!stops.is_empty());
    assert!(stops.iter().all(|id| id == "worker"));
    drop(stops);
    core.buses.unmount("m");
    core.routers.unregister("m");
}
