use std::collections::HashMap;
use std::sync::{mpsc, Weak};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use runner_core::model::{EventDraft, SignalType};
use runner_terminal::delivery_probe::{InputProbe, ProbeState};

use super::{
    DeliveryBlockedEvent, DeliveryReadiness, DeliveryReservation, DeliverySnapshot, Router,
    SessionDeliveryEvent, SUBMIT_DELAY,
};

const REDELIVERY_INTERVAL: Duration = Duration::from_secs(30);
const REDELIVERY_BACKOFF: Duration = Duration::from_secs(120);
pub(crate) const CONFIRM_WINDOW: Duration = Duration::from_millis(1500);
const INPUT_CLEAR_GRACE: Duration = Duration::from_millis(500);

pub(crate) enum Input {
    Backlog {
        handle: String,
        initial_backlog: bool,
    },
    Session {
        session_id: String,
        event: SessionDeliveryEvent,
    },
    RosterChanged,
    Stop,
}

struct Attempt {
    token: u64,
    deadline: Instant,
    snapshot: Option<DeliverySnapshot>,
    probe: InputProbe,
    submitted: bool,
    second_enter: bool,
}

struct Subscriber {
    session_id: String,
    wanted: bool,
    attempt: Option<Attempt>,
    retry: Option<Instant>,
    last_attempt: Option<Instant>,
    warned: bool,
    blocked: Option<(usize, String)>,
    paused: Option<InputProbe>,
    pause_check: Option<Instant>,
    input_cleared: bool,
    hold: Option<DeliveryReadiness>,
}

pub(crate) struct Dispatcher {
    subscribers: HashMap<String, Subscriber>,
    redelivery: Instant,
    stopped: bool,
}

impl Dispatcher {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            subscribers: HashMap::new(),
            redelivery: now + REDELIVERY_INTERVAL,
            stopped: false,
        }
    }

    /// All timing is driven by the caller's clock. Port calls never hold router state.
    pub(crate) fn advance(
        &mut self,
        router: &Router,
        input: Option<Input>,
        now: Instant,
    ) -> Option<Instant> {
        let (sessions, backlogs) = {
            let state = router.state.lock().unwrap();
            (
                state.session_by_handle.clone(),
                state.unread_by_handle.clone(),
            )
        };
        self.subscribers.retain(|handle, subscriber| {
            if sessions.get(handle) == Some(&subscriber.session_id) {
                return true;
            }
            Self::clear(router, handle, subscriber);
            false
        });
        for (handle, session_id) in sessions {
            self.subscribers
                .entry(handle.clone())
                .or_insert_with(|| Subscriber {
                    session_id,
                    // Wait for the backlog input so initial replay can seed the warning state.
                    wanted: false,
                    attempt: None,
                    retry: None,
                    last_attempt: None,
                    warned: false,
                    blocked: None,
                    paused: None,
                    pause_check: None,
                    input_cleared: false,
                    hold: None,
                });
        }
        let mut reconcile = now >= self.redelivery;
        if reconcile {
            self.redelivery = now + REDELIVERY_INTERVAL;
        }
        match input {
            Some(Input::Stop) => {
                self.stopped = true;
                for (handle, subscriber) in &mut self.subscribers {
                    Self::clear(router, handle, subscriber);
                }
                return None;
            }
            Some(Input::Backlog {
                handle,
                initial_backlog,
            }) => {
                if let Some(subscriber) = self.subscribers.get_mut(&handle) {
                    subscriber.wanted = backlogs.get(&handle).copied().unwrap_or(0) > 0;
                    if initial_backlog
                        && subscriber.wanted
                        && router.injector.delivery_readiness(&subscriber.session_id)
                            == DeliveryReadiness::Unavailable
                    {
                        subscriber.warned = true;
                    }
                }
            }
            Some(Input::Session { session_id, event }) => {
                for subscriber in self
                    .subscribers
                    .values_mut()
                    .filter(|s| s.session_id == session_id)
                {
                    match event {
                        SessionDeliveryEvent::Idle => reconcile = true,
                        SessionDeliveryEvent::InputCleared => {
                            subscriber.input_cleared = true;
                            subscriber.retry = Some(now + INPUT_CLEAR_GRACE)
                        }
                        SessionDeliveryEvent::Exited | SessionDeliveryEvent::Respawned => {
                            if let Some(attempt) = subscriber.attempt.take() {
                                log::info!("router attempt outcome=Unverified mission={} session={session_id} runtime={} reason={event:?}", router.mission_id, attempt.snapshot.as_ref().map(|s| s.runtime.as_str()).unwrap_or("unknown"));
                                router.injector.finish_delivery(&session_id, attempt.token);
                            }
                            subscriber.paused = None;
                            subscriber.pause_check = None;
                            subscriber.input_cleared = false;
                            subscriber.wanted = true;
                            subscriber.retry = None;
                            subscriber.warned = false;
                        }
                        SessionDeliveryEvent::ReadinessChanged => {
                            subscriber.retry = Some(now + INPUT_CLEAR_GRACE)
                        }
                        SessionDeliveryEvent::StartupReady
                        | SessionDeliveryEvent::InputQueueDrained => subscriber.retry = None,
                        SessionDeliveryEvent::DeliveryFinished => {}
                    }
                }
            }
            _ => {}
        }
        if self.stopped {
            return None;
        }
        for (handle, subscriber) in &mut self.subscribers {
            let unread = backlogs.get(handle).copied().unwrap_or(0);
            if subscriber.retry.is_some_and(|deadline| now >= deadline) {
                subscriber.retry = None;
            }
            if let Some(mut attempt) = subscriber.attempt.take() {
                if now < attempt.deadline {
                    subscriber.attempt = Some(attempt);
                } else if !attempt.submitted {
                    if let Some(current) = router.injector.delivery_snapshot(&subscriber.session_id)
                    {
                        attempt.probe.locate(&current.rows);
                    }
                    let submitted = router.injector.inject_reserved(
                        &subscriber.session_id,
                        attempt.token,
                        b"\r",
                    );
                    router
                        .injector
                        .finish_delivery(&subscriber.session_id, attempt.token);
                    attempt.submitted = true;
                    attempt.deadline = now + CONFIRM_WINDOW;
                    if let Err(error) = submitted {
                        log::warn!(
                            "notification Enter failed session={}: {error}",
                            subscriber.session_id
                        );
                    }
                    subscriber.attempt = Some(attempt);
                } else {
                    let current = router.injector.delivery_snapshot(&subscriber.session_id);
                    let unchanged = attempt.snapshot.as_ref().zip(current.as_ref()).is_some_and(
                        |(before, current)| {
                            before.generation == current.generation
                                && before.input_revision == current.input_revision
                        },
                    );
                    let state = current
                        .as_ref()
                        .map(|current| attempt.probe.state(&current.rows))
                        .unwrap_or(ProbeState::Unknown);
                    let turn_started = attempt.snapshot.as_ref().zip(current.as_ref()).is_some_and(
                        |(before, current)| {
                            before.generation == current.generation
                                && current.turn_started != before.turn_started
                        },
                    );
                    let outcome = if !unchanged {
                        "Unverified"
                    } else if turn_started || state == ProbeState::Cleared {
                        "Confirmed"
                    } else if state == ProbeState::Present && !attempt.second_enter {
                        let retried = attempt.snapshot.as_ref().is_some_and(|snapshot| {
                            matches!(
                                router.injector.resubmit(
                                    &subscriber.session_id,
                                    snapshot,
                                    &attempt.probe
                                ),
                                Ok(true)
                            )
                        });
                        if retried {
                            attempt.second_enter = true;
                            attempt.deadline = now + CONFIRM_WINDOW;
                            subscriber.attempt = Some(attempt);
                            continue;
                        }
                        let still_unchanged = attempt
                            .snapshot
                            .as_ref()
                            .zip(
                                router
                                    .injector
                                    .delivery_snapshot(&subscriber.session_id)
                                    .as_ref(),
                            )
                            .is_some_and(|(before, current)| {
                                before.generation == current.generation
                                    && before.input_revision == current.input_revision
                                    && attempt.probe.state(&current.rows) == ProbeState::Present
                            });
                        if still_unchanged {
                            "Unsubmitted"
                        } else {
                            "Unverified"
                        }
                    } else if state == ProbeState::Present {
                        "Unsubmitted"
                    } else {
                        "Unverified"
                    };
                    let runtime = current
                        .as_ref()
                        .or(attempt.snapshot.as_ref())
                        .map(|snapshot| snapshot.runtime.as_str())
                        .unwrap_or("unknown");
                    log::info!("router attempt outcome={outcome} mission={} session={} handle=@{handle} runtime={runtime}", router.mission_id, subscriber.session_id);
                    if outcome == "Confirmed" {
                        let draft = EventDraft::signal(
                            router.crew_id.as_deref(),
                            &router.mission_id,
                            handle,
                            SignalType::new("session_status"),
                            serde_json::json!({"state":"busy"}),
                        );
                        if let Err(error) = router
                            .injector
                            .synthesize_wake_busy(&subscriber.session_id, draft)
                        {
                            log::warn!("publish confirmed delivery busy: {error}");
                        }
                    } else if outcome == "Unsubmitted" {
                        subscriber.paused = Some(attempt.probe);
                        subscriber.input_cleared = false;
                        subscriber.pause_check = Some(now + CONFIRM_WINDOW);
                    }
                    subscriber.retry = Some(now + SUBMIT_DELAY);
                }
            }
            if let Some(probe) = &subscriber.paused {
                let probe_state = router
                    .injector
                    .delivery_snapshot(&subscriber.session_id)
                    .map(|snapshot| probe.state(&snapshot.rows))
                    .unwrap_or(ProbeState::Unknown);
                if probe_state == ProbeState::Present {
                    subscriber.input_cleared = false;
                }
                if probe_state == ProbeState::Cleared
                    || (probe_state == ProbeState::Unknown
                        && subscriber.input_cleared
                        && matches!(
                            router.injector.delivery_readiness(&subscriber.session_id),
                            DeliveryReadiness::Ready { .. }
                        ))
                {
                    subscriber.paused = None;
                    subscriber.pause_check = None;
                    subscriber.wanted = unread > 0;
                } else {
                    subscriber.pause_check = Some(now + CONFIRM_WINDOW);
                    Self::blocked(router, handle, subscriber, unread, Some("unsubmitted"));
                    continue;
                }
            }
            if unread == 0 {
                subscriber.wanted = false;
                Self::blocked(router, handle, subscriber, 0, None);
                continue;
            }
            if reconcile
                && subscriber.attempt.is_none()
                && subscriber
                    .last_attempt
                    .is_none_or(|last| now.saturating_duration_since(last) >= REDELIVERY_BACKOFF)
                && matches!(
                    router.injector.delivery_readiness(&subscriber.session_id),
                    DeliveryReadiness::Ready { idle: true }
                )
            {
                subscriber.wanted = true;
            }
            if !subscriber.wanted
                || subscriber.attempt.is_some()
                || subscriber.retry.is_some_and(|deadline| now < deadline)
            {
                continue;
            }
            subscriber.retry = None;
            let readiness = router.injector.delivery_readiness(&subscriber.session_id);
            if subscriber.hold != Some(readiness) {
                if let Some(previous) = subscriber.hold.take() {
                    log::info!(
                        "router delivery released session={} handle=@{handle} reason={previous:?}",
                        subscriber.session_id
                    );
                }
                if !matches!(readiness, DeliveryReadiness::Ready { .. }) {
                    log::info!(
                        "router delivery held session={} handle=@{handle} reason={readiness:?}",
                        subscriber.session_id
                    );
                    subscriber.hold = Some(readiness);
                }
            }
            Self::blocked(
                router,
                handle,
                subscriber,
                unread,
                match readiness {
                    DeliveryReadiness::Drafting => Some("draft"),
                    DeliveryReadiness::NeedsYou => Some("needs_you"),
                    _ => None,
                },
            );
            match readiness {
                DeliveryReadiness::Ready { .. } => {
                    match router.injector.reserve_delivery(&subscriber.session_id) {
                        Ok(DeliveryReservation::Ready(token)) => {
                            let unread = router
                                .state
                                .lock()
                                .unwrap()
                                .unread_by_handle
                                .get(handle)
                                .copied()
                                .unwrap_or(0);
                            if unread == 0 {
                                router
                                    .injector
                                    .finish_delivery(&subscriber.session_id, token);
                                subscriber.wanted = false;
                                Self::blocked(router, handle, subscriber, 0, None);
                                continue;
                            }
                            let snapshot =
                                router.injector.delivery_snapshot(&subscriber.session_id);
                            let body = format!(
                                "[inbox] {unread} new messages — run `runner msg read` to view."
                            );
                            match router.injector.inject_reserved(
                                &subscriber.session_id,
                                token,
                                body.as_bytes(),
                            ) {
                                Ok(true) => {
                                    subscriber.wanted = false;
                                    subscriber.last_attempt = Some(now);
                                    subscriber.attempt = Some(Attempt {
                                        token,
                                        deadline: now + SUBMIT_DELAY,
                                        probe: InputProbe::new(
                                            snapshot
                                                .as_ref()
                                                .map(|s| s.rows.clone())
                                                .unwrap_or_default(),
                                            &body,
                                        ),
                                        snapshot,
                                        submitted: false,
                                        second_enter: false,
                                    });
                                }
                                _ => {
                                    router
                                        .injector
                                        .finish_delivery(&subscriber.session_id, token);
                                    subscriber.retry = Some(now + REDELIVERY_INTERVAL);
                                }
                            }
                        }
                        _ => subscriber.retry = Some(now + REDELIVERY_INTERVAL),
                    }
                }
                DeliveryReadiness::RecentlyTyping(delay) => subscriber.retry = Some(now + delay),
                DeliveryReadiness::Unavailable if !subscriber.warned => {
                    subscriber.warned = true;
                    router.warn(format!(
                        "delivery to @{handle} queued until the session resumes"
                    ));
                }
                _ => {}
            }
        }
        Some(
            self.subscribers
                .values()
                .flat_map(|subscriber| {
                    subscriber
                        .retry
                        .into_iter()
                        .chain(subscriber.attempt.as_ref().map(|a| a.deadline))
                        .chain(subscriber.pause_check)
                })
                .fold(self.redelivery, Instant::min),
        )
    }

    fn blocked(
        router: &Router,
        handle: &str,
        subscriber: &mut Subscriber,
        unread: usize,
        reason: Option<&str>,
    ) {
        let next = reason.map(|reason| (unread, reason.to_owned()));
        if subscriber.blocked == next {
            return;
        }
        subscriber.blocked = next.clone();
        router.ui_notifier.delivery_blocked(&DeliveryBlockedEvent {
            mission_id: router.mission_id.clone(),
            session_id: subscriber.session_id.clone(),
            handle: handle.to_owned(),
            unread_count: unread,
            blocked: next.is_some(),
            reason: next.map(|(_, reason)| reason),
        });
    }

    fn clear(router: &Router, handle: &str, subscriber: &mut Subscriber) {
        if let Some(attempt) = subscriber.attempt.take() {
            log::info!("router attempt outcome=Unverified mission={} session={} handle=@{handle} runtime={} reason=Unmounted", router.mission_id, subscriber.session_id, attempt.snapshot.as_ref().map(|s| s.runtime.as_str()).unwrap_or("unknown"));
            router
                .injector
                .finish_delivery(&subscriber.session_id, attempt.token);
        }
        Self::blocked(router, handle, subscriber, 0, None);
    }
}

pub(super) struct Driver {
    tx: mpsc::Sender<Input>,
    thread: JoinHandle<()>,
}

impl Driver {
    pub(super) fn start(
        router: Weak<Router>,
        tx: mpsc::Sender<Input>,
        rx: mpsc::Receiver<Input>,
    ) -> Self {
        let thread = std::thread::Builder::new()
            .name("mission-dispatch".into())
            .spawn(move || {
                let mut dispatcher = Dispatcher::new(Instant::now());
                let mut input = Some(Input::RosterChanged);
                loop {
                    let Some(router) = router.upgrade() else {
                        return;
                    };
                    let deadline = dispatcher.advance(&router, input, Instant::now());
                    drop(router);
                    let Some(deadline) = deadline else {
                        return;
                    };
                    input =
                        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                            Ok(input) => Some(input),
                            Err(mpsc::RecvTimeoutError::Timeout) => None,
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        };
                }
            })
            .expect("start mission dispatcher");
        Self { tx, thread }
    }
    pub(super) fn stop(self) {
        let _ = self.tx.send(Input::Stop);
        if self.thread.thread().id() != std::thread::current().id() {
            let _ = self.thread.join();
        }
    }
}
