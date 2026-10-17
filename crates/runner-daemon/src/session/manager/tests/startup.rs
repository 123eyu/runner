use super::*;
use crate::router::{DeliveryReservation, Router, RouterUiNotifier};
use crate::session::state::agent::AgentEvent;

struct NoUi;
impl RouterUiNotifier for NoUi {
    fn delivery_blocked(&self, _: &crate::router::DeliveryBlockedEvent) {}
}

struct StartupFixture {
    manager: Arc<SessionManager>,
    fake: Arc<FakeRuntime>,
    router: Arc<Router>,
    log: Arc<EventLog>,
    _dir: tempfile::TempDir,
}

impl StartupFixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let log = Arc::new(EventLog::open(dir.path()).unwrap());
        let fake = fake_runtime();
        let manager = mgr_with_fake(None, fake.clone());
        let role = role("fake", &[]);
        let router = Router::new(
            "mission".into(),
            None,
            String::new(),
            &[crate::model::SlotWithRole {
                slot: slot_for(&role),
                role,
            }],
            vec![],
            None,
            log.clone(),
            manager.clone(),
            Arc::new(NoUi),
        )
        .unwrap();
        router.register_pending_sessions(&[("tester".into(), "session".into())]);
        Self {
            manager,
            fake,
            router,
            log,
            _dir: dir,
        }
    }

    fn queue(&self, text: &str) {
        let event = self
            .log
            .append(EventDraft::message(
                None,
                "mission",
                "human",
                Some("tester".into()),
                text,
            ))
            .unwrap();
        self.router.handle_event(&event);
    }

    fn launch(&self, runtime: &str, pending_turn: bool, now: Instant) -> OutputStream {
        let (runtime_session, output) = self
            .fake
            .spawn(SpawnSpec {
                session_id: "session".into(),
                ..Default::default()
            })
            .unwrap();
        self.manager.install_handle_with_size_persistence(
            "session",
            SessionHandle {
                startup: Some(super::super::startup::StartupDelivery::new(
                    runtime,
                    pending_turn,
                    now,
                )),
                #[cfg(windows)]
                pending_first_turn: None,
                id: "session".into(),
                mission_id: Some("mission".into()),
                role_id: None,
                runtime_session,
                codex_capture: None,
                forwarder: None,
                stop: output.stop_flag(),
            },
            Some(ForwarderEmitCtx {
                crew_id: None,
                mission_id: "mission".into(),
                handle: "tester".into(),
                event_log: self.log.clone(),
            }),
            Some(DEFAULT_PTY_SIZE),
            |_, _| true,
            &Capture::default(),
        );
        output
    }

    fn event(&self, runtime: Runtime, event: AgentEvent) {
        self.manager.publish_agent_event(
            "session",
            AgentEvent::Batch {
                runtime,
                events: vec![event],
            },
            &Capture::default(),
        );
    }

    fn held(&self) {
        assert_eq!(
            self.manager.reserve_delivery("session").unwrap(),
            DeliveryReservation::Starting
        );
        assert!(self.fake.inputs.lock().unwrap().is_empty());
    }

    fn submitted(&self) {
        let deadline = Instant::now() + ci_scaled_budget(Duration::from_secs(3));
        loop {
            if self
                .fake
                .inputs
                .lock()
                .unwrap()
                .iter()
                .any(|input| matches!(input, FakeInput::Key { key, .. } if key == "Enter"))
            {
                break;
            }
            assert!(Instant::now() < deadline, "held delivery never submitted");
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for StartupFixture {
    fn drop(&mut self) {
        self.manager.kill("session").unwrap();
    }
}

#[test]
fn router_holds_startup_deliveries_until_hook_turn_completion_and_resets_on_resume() {
    for runtime in [
        Runtime::ClaudeCode,
        Runtime::Codex,
        Runtime::Copilot,
        Runtime::Pi,
        Runtime::Antigravity,
    ] {
        let fixture = StartupFixture::new();
        fixture.queue("join broadcast before process launch");
        fixture.queue("task before process launch");
        let now = Instant::now();
        let output = fixture.launch(runtime.key(), true, now);
        fixture.held();
        fixture.event(runtime, AgentEvent::StartupReady);
        fixture.held();
        fixture.manager.startup_output(
            "session",
            &output.stop_flag(),
            b"Starting MCP servers (6/7)",
            now,
        );
        fixture.manager.release_settled_startup(
            "session",
            &output.stop_flag(),
            now + Duration::from_secs(60),
        );
        fixture.held();
        fixture.event(runtime, AgentEvent::TurnStarted);
        fixture.manager.release_settled_startup(
            "session",
            &output.stop_flag(),
            now + Duration::from_secs(3600),
        );
        fixture.held();
        fixture.event(
            runtime,
            AgentEvent::TurnEnded {
                outcome: TurnOutcome::Completed,
            },
        );
        fixture.submitted();
        assert!(fixture
            .fake
            .bytes_writes()
            .iter()
            .any(|(_, bytes)| String::from_utf8_lossy(bytes).contains("2 new messages")));
        fixture.manager.kill("session").unwrap();
        fixture.fake.inputs.lock().unwrap().clear();
        fixture.queue("task before resume");
        let resume = fixture.launch(runtime.key(), false, now);
        fixture.held();
        if runtime == Runtime::Antigravity {
            fixture
                .manager
                .startup_output("session", &resume.stop_flag(), b"resumed prompt", now);
            fixture.manager.release_settled_startup(
                "session",
                &resume.stop_flag(),
                now + Duration::from_secs(10),
            );
        } else {
            fixture.event(runtime, AgentEvent::StartupReady);
        }
        fixture.submitted();
    }
}

#[test]
fn router_output_fallback_requires_output_startup_grace_and_idle_including_bridge_failure() {
    for runtime in [
        Runtime::OpenCode,
        Runtime::Cursor,
        Runtime::Grok,
        Runtime::Trae,
        Runtime::Shell,
        Runtime::Codex,
    ] {
        let fixture = StartupFixture::new();
        fixture.queue("task");
        let now = Instant::now();
        let output = fixture.launch(runtime.key(), true, now);
        let launch = output.stop_flag();
        if runtime == Runtime::Codex {
            fixture
                .manager
                .status_bridge_failed("session", &Capture::default());
        }
        fixture
            .manager
            .release_settled_startup("session", &launch, now + Duration::from_secs(60));
        fixture.held();
        fixture
            .manager
            .startup_output("session", &launch, b"startup screen", now);
        fixture
            .manager
            .release_settled_startup("session", &launch, now + Duration::from_secs(2));
        fixture.held();
        fixture.manager.startup_output(
            "session",
            &launch,
            b"first answer",
            now + Duration::from_secs(11),
        );
        fixture
            .manager
            .release_settled_startup("session", &launch, now + Duration::from_secs(12));
        fixture.held();
        fixture
            .manager
            .release_settled_startup("session", &launch, now + Duration::from_secs(13));
        fixture.submitted();
    }
}

#[test]
fn codex_conversation_recovery_in_first_turn_falls_back_to_output_release() {
    let fixture = StartupFixture::new();
    fixture.queue("answer after recovery");
    let now = Instant::now();
    let output = fixture.launch("codex", true, now);
    let launch = output.stop_flag();
    fixture.event(Runtime::Codex, AgentEvent::TurnStarted);
    fixture
        .manager
        .startup_output("session", &launch, b"first turn", now);
    fixture.event(Runtime::Codex, AgentEvent::ConversationRecovered);
    fixture
        .manager
        .release_settled_startup("session", &launch, now + Duration::from_secs(1));
    fixture.held();
    fixture
        .manager
        .release_settled_startup("session", &launch, now + Duration::from_secs(12));
    fixture.submitted();
}

#[test]
fn codex_interrupted_first_turn_releases_the_startup_hold_once_the_abort_settles() {
    let fixture = StartupFixture::new();
    fixture.queue("task after interrupt");
    let now = Instant::now();
    let _output = fixture.launch("codex", true, now);
    fixture.event(Runtime::Codex, AgentEvent::TurnStarted);
    fixture.event(
        Runtime::Codex,
        AgentEvent::TurnEnded {
            outcome: crate::session::status::TurnOutcome::Interrupted,
        },
    );
    fixture.held();
    fixture.manager.publish_agent_event(
        "session",
        AgentEvent::Transcript {
            runtime: Runtime::Codex,
            events: vec![AgentEvent::AbortSettled],
        },
        &Capture::default(),
    );
    fixture.submitted();
}

#[test]
fn codex_failed_first_turn_releases_the_startup_hold() {
    let fixture = StartupFixture::new();
    fixture.queue("task after failure");
    let now = Instant::now();
    let _output = fixture.launch("codex", true, now);
    fixture.event(Runtime::Codex, AgentEvent::TurnStarted);
    fixture.held();
    fixture.manager.publish_agent_event(
        "session",
        AgentEvent::Transcript {
            runtime: Runtime::Codex,
            events: vec![AgentEvent::TurnEnded {
                outcome: crate::session::status::TurnOutcome::Failed,
            }],
        },
        &Capture::default(),
    );
    fixture.submitted();
}

#[test]
fn router_startup_exit_keeps_existing_drop_warning_and_does_not_deliver_on_resume() {
    let fixture = StartupFixture::new();
    fixture.queue("task before exit");
    let now = Instant::now();
    let output = fixture.launch("codex", true, now);
    fixture.held();
    fixture.manager.kill("session").unwrap();
    let entries = fixture.log.read_from(0).unwrap();
    assert!(entries.iter().any(|entry| entry
        .event
        .payload
        .to_string()
        .contains("dropped 1 deferred deliveries")));
    let _resume = fixture.launch("codex", false, now);
    fixture.event(Runtime::Codex, AgentEvent::StartupReady);
    fixture.manager.release_settled_startup(
        "session",
        &output.stop_flag(),
        now + Duration::from_secs(60),
    );
    assert!(fixture.fake.inputs.lock().unwrap().is_empty());
}

#[test]
fn completed_startup_keeps_a_human_draft_held_until_input_clears() {
    let fixture = StartupFixture::new();
    fixture.queue("task");
    let now = Instant::now();
    let _output = fixture.launch("codex", true, now);
    fixture.manager.report_input_state(
        "session",
        InputObservation {
            composing: false,
            state: InputState::Drafting,
            since: now,
            composer_visible: true,
        },
    );
    fixture.event(Runtime::Codex, AgentEvent::TurnStarted);
    fixture.event(
        Runtime::Codex,
        AgentEvent::TurnEnded {
            outcome: TurnOutcome::Completed,
        },
    );
    assert_eq!(
        fixture.manager.reserve_delivery("session").unwrap(),
        DeliveryReservation::Drafting {
            composer_visible: true
        }
    );
    assert!(fixture.fake.inputs.lock().unwrap().is_empty());
    fixture.manager.report_input_state(
        "session",
        InputObservation {
            composing: false,
            state: InputState::Idle,
            since: now,
            composer_visible: true,
        },
    );
    fixture.submitted();
}
