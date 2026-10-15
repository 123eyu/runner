use super::*;

// A startup paint can go quiet before the first answer. Give output-only runtimes
// a separate startup grace, then require the normal idle window after their last byte.
const STARTUP_OUTPUT_GRACE: Duration = Duration::from_secs(10);

pub(super) struct StartupDelivery {
    hooks: bool,
    pending_turn: bool,
    started: Instant,
    first_output: Option<Instant>,
    last_output: Option<Instant>,
    working: bool,
}

impl StartupDelivery {
    pub(super) fn new(runtime: &str, pending_turn: bool, now: Instant) -> Self {
        let hooks = crate::runtimes::for_key(runtime).startup_delivery_uses_hooks(pending_turn);
        Self {
            hooks,
            pending_turn,
            started: now,
            first_output: None,
            last_output: None,
            working: false,
        }
    }

    pub(super) fn bridge_failed(&mut self) {
        self.hooks = false;
        self.working = false;
    }

    pub(super) fn observation(
        &mut self,
        observation: &super::super::status::AgentObservation,
    ) -> bool {
        if observation.source != ObservationSource::Hook {
            return false;
        }
        self.working = observation.activity == Activity::Working || observation.needs_you();
        self.hooks
            && !self.working
            && matches!(observation.activity, Activity::Idle | Activity::Ready)
            && (!self.pending_turn || observation.outcome.is_some())
    }

    pub(super) fn output(&mut self, bytes: &[u8], now: Instant) {
        if !bytes.is_empty() {
            self.first_output.get_or_insert(now);
            self.last_output = Some(now);
        }
    }

    pub(super) fn output_settled(&self, now: Instant) -> bool {
        !self.hooks
            && !self.working
            && self.first_output.is_some_and(|first| {
                now.duration_since(first.max(self.started)) >= STARTUP_OUTPUT_GRACE
            })
            && self.last_output.is_some_and(|last| {
                now.duration_since(last) >= super::super::pty_runtime::DEFAULT_IDLE_THRESHOLD
            })
    }
}

impl SessionManager {
    pub(super) fn startup_output(
        &self,
        session_id: &str,
        launch: &Arc<AtomicBool>,
        bytes: &[u8],
        now: Instant,
    ) {
        let Some(session) = self.session_state(session_id) else {
            return;
        };
        let mut session = session.lock().unwrap();
        if let Some(handle) = session
            .handle
            .as_mut()
            .filter(|handle| Arc::ptr_eq(launch, &handle.stop))
        {
            if let Some(startup) = handle.startup.as_mut() {
                startup.output(bytes, now);
            }
        }
    }

    pub(super) fn release_settled_startup(
        &self,
        session_id: &str,
        launch: &Arc<AtomicBool>,
        now: Instant,
    ) {
        let Some(session) = self.session_state(session_id) else {
            return;
        };
        let ready = {
            let mut session = session.lock().unwrap();
            if session.killed || launch.load(Ordering::Acquire) {
                return;
            }
            let Some(handle) = session
                .handle
                .as_mut()
                .filter(|handle| Arc::ptr_eq(launch, &handle.stop))
            else {
                return;
            };
            if handle
                .startup
                .as_ref()
                .is_some_and(|startup| startup.output_settled(now))
            {
                handle.startup = None;
                true
            } else {
                false
            }
        };
        if ready {
            self.notify_delivery_event(session_id, router::SessionDeliveryEvent::StartupReady);
        }
    }
}
