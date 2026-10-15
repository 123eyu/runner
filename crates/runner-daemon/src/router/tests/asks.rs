use super::*;

#[test]
fn human_said_routes_to_target_or_lead() {
    let (router, injector, log, _dir) = fixture(
        vec![slot_with_role("lead", true), slot_with_role("impl", false)],
        &[("lead", "S-LEAD"), ("impl", "S-IMPL")],
    );

    // Targeted: lands on the worker.
    let direct = log
        .append(signal(
            "human",
            "human_said",
            serde_json::json!({ "text": "look at line 42", "target": "impl" }),
        ))
        .unwrap();
    router.handle_event(&direct);
    let impl_pushes = injector.pushes_for("S-IMPL");
    assert_eq!(impl_pushes.len(), 1);
    assert!(impl_pushes[0].contains("look at line 42"));
    assert!(injector.pushes_for("S-LEAD").is_empty());

    // Untargeted: defaults to the lead.
    let bcast = log
        .append(signal(
            "human",
            "human_said",
            serde_json::json!({ "text": "status?" }),
        ))
        .unwrap();
    router.handle_event(&bcast);
    let lead_pushes = injector.pushes_for("S-LEAD");
    assert_eq!(lead_pushes.len(), 1);
    assert!(lead_pushes[0].contains("status?"));
}

#[test]
fn old_ask_lead_events_parse_but_inject_nothing() {
    let (router, injector, log, _dir) = fixture(
        vec![slot_with_role("lead", true), slot_with_role("impl", false)],
        &[("lead", "S-LEAD"), ("impl", "S-IMPL")],
    );
    let ev = log
        .append(signal(
            "impl",
            "ask_lead",
            serde_json::json!({ "question": "use notify-debouncer-full?", "context": "Pros: …\nCons: …" }),
        ))
        .unwrap();
    router.handle_event(&ev);

    assert!(injector.pushes_for("S-LEAD").is_empty());
    assert!(injector.pushes_for("S-IMPL").is_empty());
    router.reconstruct_from_log().unwrap();
    router.handle_event(&ev);
    assert!(injector.pushes_for("S-LEAD").is_empty());
}

#[test]
fn ask_human_appends_human_question_card_and_records_pending_ask() {
    let (router, _injector, log, _dir) = fixture(
        vec![slot_with_role("lead", true), slot_with_role("impl", false)],
        &[("lead", "S-LEAD"), ("impl", "S-IMPL")],
    );
    let ev = log
        .append(signal(
            "lead",
            "ask_human",
            serde_json::json!({
                "prompt": "Approve?",
                "choices": ["yes", "no"],
                "on_behalf_of": "impl",
            }),
        ))
        .unwrap();
    router.handle_event(&ev);

    // Append a `human_question` event referencing the original ask. Per
    // arch §5.5.0, the canonical `question_id` is the card event's own
    // `id`; `triggered_by` ties it back to the originating `ask_human`.
    // The convenience-echo `payload.question_id` is intentionally absent
    // because the id isn't known until after append.
    let signals = read_signals(&log);
    let card = signals
        .iter()
        .find(|s| {
            s.signal_type
                .as_ref()
                .map(|t| t.as_str() == "human_question")
                .unwrap_or(false)
        })
        .expect("router must append human_question");
    assert_eq!(card.from, "router");
    assert_eq!(card.payload["prompt"], "Approve?");
    assert_eq!(card.payload["choices"], serde_json::json!(["yes", "no"]));
    assert_eq!(card.payload["on_behalf_of"], "impl");
    assert_eq!(card.payload["triggered_by"], ev.id);
    assert!(
        card.payload.get("question_id").is_none(),
        "question_id is the event's own id; not echoed in payload"
    );
}

#[test]
fn human_response_routes_back_to_asker() {
    let (router, injector, log, _dir) = fixture(
        vec![slot_with_role("lead", true), slot_with_role("impl", false)],
        &[("lead", "S-LEAD"), ("impl", "S-IMPL")],
    );
    let ask = log
        .append(signal(
            "lead",
            "ask_human",
            serde_json::json!({
                "prompt": "Approve?",
                "choices": ["yes", "no"],
                "on_behalf_of": "impl",
            }),
        ))
        .unwrap();
    router.handle_event(&ask);

    // human_response.payload.question_id is the human_question.id (arch
    // §5.5.0), not the ask_human.id. Find the card the router appended.
    let card_id = read_signals(&log)
        .into_iter()
        .find(|s| {
            s.signal_type
                .as_ref()
                .map(|t| t.as_str() == "human_question")
                .unwrap_or(false)
        })
        .expect("router must append human_question")
        .id;

    let resp = log
        .append(signal(
            "human",
            "human_response",
            serde_json::json!({ "question_id": card_id, "choice": "yes" }),
        ))
        .unwrap();
    router.handle_event(&resp);

    let lead_pushes = injector.pushes_for("S-LEAD");
    assert!(
        lead_pushes
            .iter()
            .any(|p| p.contains("[human_response] yes")),
        "lead must receive the routed answer; got {lead_pushes:?}",
    );
    // The pending-ask map is consumed; a duplicate response surfaces a
    // warning rather than re-injecting.
    let dup = log
        .append(signal(
            "human",
            "human_response",
            serde_json::json!({ "question_id": card_id, "choice": "no" }),
        ))
        .unwrap();
    router.handle_event(&dup);
    let warnings: Vec<_> = read_signals(&log)
        .into_iter()
        .filter(|s| {
            s.signal_type
                .as_ref()
                .map(|t| t.as_str() == "mission_warning")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        warnings.iter().any(|w| w.payload["message"]
            .as_str()
            .unwrap()
            .contains("unknown question_id")),
        "duplicate response must produce mission_warning; got {warnings:?}",
    );
}

#[test]
fn human_response_without_matching_question_emits_mission_warning() {
    let (router, injector, log, _dir) =
        fixture(vec![slot_with_role("lead", true)], &[("lead", "S-LEAD")]);
    let resp = log
        .append(signal(
            "human",
            "human_response",
            serde_json::json!({ "question_id": "01HUNKNOWN", "choice": "yes" }),
        ))
        .unwrap();
    router.handle_event(&resp);

    assert!(injector.all_pushes().is_empty());
    let warnings: Vec<_> = read_signals(&log)
        .into_iter()
        .filter(|s| {
            s.signal_type
                .as_ref()
                .map(|t| t.as_str() == "mission_warning")
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(warnings.len(), 1);
}

#[test]
fn leadless_human_messages_and_answers_route_to_members() {
    let (router, injector, log, _dir) = fixture(
        vec![slot_with_role("one", false), slot_with_role("two", false)],
        &[("one", "S-ONE"), ("two", "S-TWO")],
    );
    assert_eq!(router.lead_handle(), None);
    let event = log
        .append(signal(
            "human",
            "human_said",
            serde_json::json!({"text": "hello team"}),
        ))
        .unwrap();
    router.handle_event(&event);
    assert!(injector.pushes_for("S-ONE")[0].contains("hello team"));
    assert!(injector.pushes_for("S-TWO")[0].contains("hello team"));
    let ask = log
        .append(signal(
            "one",
            "ask_human",
            serde_json::json!({"prompt": "Ship?", "choices": ["yes", "no"]}),
        ))
        .unwrap();
    router.handle_event(&ask);
    let card = read_signals(&log)
        .into_iter()
        .find(|event| {
            event
                .signal_type
                .as_ref()
                .is_some_and(|kind| kind.as_str() == "human_question")
        })
        .unwrap();
    let answer = log
        .append(signal(
            "human",
            "human_response",
            serde_json::json!({"question_id": card.id, "choice": "yes"}),
        ))
        .unwrap();
    router.handle_event(&answer);
    wait_until(Duration::from_secs(2), || {
        injector
            .pushes_for("S-ONE")
            .iter()
            .any(|text| text.contains("[human_response] yes"))
    });
    assert!(!injector
        .pushes_for("S-TWO")
        .iter()
        .any(|text| text.contains("human_response")));
    router.record_slot_restart("one", "S-ONE", None).unwrap();
    let notes: Vec<_> = log
        .read_from(0)
        .unwrap()
        .into_iter()
        .filter(|entry| entry.event.from == "runner")
        .collect();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].event.to.as_deref(), Some("two"));
}
