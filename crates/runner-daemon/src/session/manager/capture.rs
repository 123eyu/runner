use super::*;

impl SessionManager {
    pub(super) fn capture_prompt_marker(
        runtime_key: &str,
        session_id: &str,
        first_turn: Option<String>,
    ) -> (Option<String>, Option<String>) {
        crate::runtimes::for_key(runtime_key).capture_prompt(session_id, first_turn)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_runtime_capture(
        self: &Arc<Self>,
        runtime_key: &str,
        session_id: &str,
        mission_id: Option<String>,
        app_data_dir: &Path,
        row_started_at: &str,
        stop: Arc<AtomicBool>,
        pool: &Arc<DbPool>,
        events: &Arc<dyn SessionEvents>,
    ) {
        crate::runtimes::for_key(runtime_key).start_capture(crate::runtimes::CaptureContext {
            manager: Arc::downgrade(self),
            session_id: session_id.to_owned(),
            mission_id,
            app_data_dir: app_data_dir.to_path_buf(),
            row_started_at: row_started_at.to_owned(),
            stop,
            pool: Arc::clone(pool),
            events: Arc::clone(events),
        });
    }

    #[cfg(windows)]
    pub(super) fn queue_windows_batch_first_turn(
        &self,
        session_id: &str,
        role: &Role,
        plan: &router::runtime::ResumePlan,
        first_turn: Option<&str>,
        deadline: Instant,
    ) {
        if Runtime::parse(role.runtime.as_str()).is_some_and(|runtime| !runtime.is_shell())
            && !plan.resuming
            && crate::session::launch::is_windows_batch(&role.command)
        {
            if let Some(body) = first_turn.filter(|body| !body.trim().is_empty()) {
                let state = self.session_state_or_insert(session_id);
                let mut state = state.lock().unwrap();
                if let Some(handle) = state.handle.as_mut() {
                    handle.pending_first_turn = Some(PendingFirstTurn {
                        body: body.to_owned(),
                        deadline,
                        readiness: Default::default(),
                    });
                } else {
                    log::warn!(
                        "cannot queue first turn for {session_id}: session handle is missing"
                    );
                }
            }
        }
    }
}
