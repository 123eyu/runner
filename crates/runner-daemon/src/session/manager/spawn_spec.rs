use super::*;

const LOCALE_VARS: [&str; 3] = ["LANG", "LC_ALL", "LC_CTYPE"];

/// Dock-launched GUI apps inherit no locale, so children run in the
/// POSIX C locale and macOS clipboard tools (claude's `pbpaste`)
/// decode UTF-8 as Mac Roman — CJK text arrives as mojibake (#461).
/// Mirror alacritty's minimal macOS fallback: set `LC_CTYPE=UTF-8`
/// only when no layer supplies a locale (a full `LANG` would also
/// switch tool message languages). Children inherit the app's own
/// process env on top of the spec, so a locale there counts too.
pub(super) fn ensure_utf8_locale(env: &mut BTreeMap<String, String>, process_has_locale: bool) {
    if process_has_locale || LOCALE_VARS.iter().any(|var| env.contains_key(*var)) {
        return;
    }
    env.insert("LC_CTYPE".into(), "UTF-8".into());
}

/// The environment an agent process starts from. Login-shell vars (the
/// captured configuration and proxy variables) are the bottom layer; a role row can override
/// any of them, being the most specific configuration surface. System vars
/// layer on top so the user can't accidentally shadow them — PATH comes from
/// the composed path and a role.env PATH is filtered by
/// `launch::is_reserved_env_name` — then `extra_env`, the runtime's quiet
/// flags (#475), and a UTF-8 locale when no layer supplies one.
pub(super) fn agent_env(
    shell_vars: BTreeMap<String, String>,
    role_env: &HashMap<String, String>,
    extra_env: BTreeMap<String, String>,
    runtime_key: &str,
) -> BTreeMap<String, String> {
    let mut env = shell_vars;
    for (k, v) in role_env {
        env.insert(k.clone(), v.clone());
    }
    env.insert("TERM".into(), "xterm-256color".into());
    env.insert("COLORTERM".into(), "truecolor".into());
    if extra_env.contains_key("RUNNER_MISSION_ID") && !extra_env.contains_key("RUNNER_CREW_ID") {
        env.remove("RUNNER_CREW_ID");
    }
    env.extend(extra_env);
    for &(name, value) in crate::runtimes::for_key(runtime_key).launch_env() {
        env.insert(name.into(), value.into());
    }
    let process_has_locale = LOCALE_VARS
        .iter()
        .any(|var| std::env::var_os(var).is_some());
    ensure_utf8_locale(&mut env, process_has_locale);
    env
}

impl SessionManager {
    pub(super) fn spawn_runtime(
        &self,
        mut spec: SpawnSpec,
    ) -> super::super::runtime::RuntimeResult<(RuntimeSession, OutputStream)> {
        if spec
            .env
            .contains_key(runner_core::protocol::hook::GENERATION_ENV)
        {
            if let Some(endpoint) = self.hook_endpoint.read().unwrap().as_ref() {
                spec.env.insert(
                    runner_core::protocol::hook::ENDPOINT_ENV.into(),
                    endpoint.0.to_string_lossy().into_owned(),
                );
            }
        }
        self.runtime.spawn(spec)
    }

    pub(super) fn resolve_role_executable(&self, role: &Role, pool: &DbPool) -> Result<Role> {
        let Some(definition) = crate::runtimes::for_key(&role.runtime).catalog() else {
            return Ok(role.clone());
        };
        if role.command != definition.command {
            return Ok(role.clone());
        }
        let effective = crate::runtime_status::effective_runtime_command(
            definition.name,
            pool,
            &self.shell_env,
            &self.discovery_state,
        )?;
        let mut resolved = role.clone();
        resolved.command = effective.command;
        Ok(resolved)
    }

    pub(super) fn enter_launch_gate(&self, session_id: &str, runtime_key: &str) {
        crate::runtimes::for_key(runtime_key).launch_gate(session_id, &self.launch_gate);
    }

    pub(super) fn seed_runtime_project_trust(
        &self,
        session_id: &str,
        runtime_key: &str,
        cwd: Option<&Path>,
        copilot_home: Option<&str>,
    ) {
        let result =
            crate::runtimes::for_key(runtime_key).seed_trust(session_id, cwd, copilot_home);
        if let (Err(e), Some(cwd)) = (result, cwd) {
            log::warn!(
                "failed to seed {:?} project trust: session={session_id} cwd={} error={e}",
                Runtime::parse(runtime_key),
                cwd.display()
            );
        }
    }

    /// Build a `SpawnSpec` skeleton with the manager's stable inputs
    /// (shell PATH, role env after merging system vars). The
    /// runtime adapter argv (resume_plan + trailing_runtime_args)
    /// lives at the call site since it depends on a pre-resolved
    /// `agent_session_key`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn base_spawn_spec(
        &self,
        session_id: String,
        role: &Role,
        cwd: Option<String>,
        mission: bool,
        shim_dir: Option<PathBuf>,
        bundled_bin_dir: Option<PathBuf>,
        initial_size: Option<(u16, u16)>,
        extra_env: BTreeMap<String, String>,
    ) -> SpawnSpec {
        let shell_env = self
            .shell_env
            .read()
            .expect("runtime shell environment lock poisoned")
            .clone();
        SpawnSpec {
            agent_runtime: Runtime::parse(&role.runtime),
            pending_turn: None,
            session_id,
            cwd: cwd.map(PathBuf::from),
            command: role.command.clone(),
            args: role.args.clone(),
            env: agent_env(shell_env.vars, &role.env, extra_env, &role.runtime),
            mission,
            shim_dir,
            bundled_bin_dir,
            shell_path: shell_env.path,
            initial_size,
        }
    }

    /// The PTY for a CLI's own update command (#533): the agent
    /// environment `base_spawn_spec` builds, without Runner's layers — no
    /// role env, mission variables, bundled CLI, hooks or #475 quiet flags.
    /// It has no `sessions` row; `spawn_unlisted` runs it.
    pub fn update_spawn_spec(
        &self,
        command: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
        initial_size: (u16, u16),
    ) -> SpawnSpec {
        let shell_env = self
            .shell_env
            .read()
            .expect("runtime shell environment lock poisoned")
            .clone();
        SpawnSpec {
            agent_runtime: None,
            pending_turn: None,
            session_id: ulid::Ulid::new().to_string(),
            cwd,
            command,
            args,
            env: agent_env(shell_env.vars, &HashMap::new(), BTreeMap::new(), ""),
            mission: false,
            shim_dir: None,
            bundled_bin_dir: None,
            shell_path: shell_env.path,
            initial_size: Some(initial_size),
        }
    }

    /// Apply the runtime adapter's resume + trailing args to a
    /// `SpawnSpec`. Mirrors what the portable-pty `spawn` paths
    /// did inline; factored out so spawn / spawn_direct / resume
    /// can share the argv composition.
    ///
    /// `first_turn` is the composed first-user-turn body (member prompt for missions,
    /// persona for direct chats). When the runtime accepts the
    /// positional `[PROMPT]` argv and the body fits in
    /// `FIRST_TURN_ARGV_MAX_BYTES`, the body lands as the trailing
    /// positional. Returns whether the body was delivered via argv
    /// so the caller can warn if a supported runtime somehow missed
    /// the deterministic path.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_runtime_args(
        &self,
        spec: &mut SpawnSpec,
        role: &Role,
        plan: &router::runtime::ResumePlan,
        app_data_dir: &Path,
        system_prompt_path: Option<&Path>,
        first_turn: Option<&str>,
        mission_bus_dir: Option<&Path>,
        pool: Option<&DbPool>,
    ) -> bool {
        let adapter = crate::runtimes::for_key(&role.runtime);
        let env = crate::shell_path::LoginShellEnv {
            path: spec.shell_path.clone(),
            vars: spec.env.clone(),
        };
        let default_model = Runtime::parse(&role.runtime).and_then(|runtime| {
            self.discovery_state
                .read()
                .ok()?
                .models
                .catalog(
                    runtime,
                    &crate::runtime_status::models::source_with_main(
                        runtime,
                        &spec.command,
                        &env,
                        pool,
                    ),
                )?
                .default_model
                .clone()
        });
        spec.agent_runtime = Runtime::parse(&role.runtime);
        spec.pending_turn = adapter
            .status_hooks()
            .is_some_and(|hooks| hooks.tracks_pending_turn())
            .then_some(!plan.resuming && first_turn.is_some_and(|body| !body.trim().is_empty()));
        #[cfg(windows)]
        let first_turn = if crate::session::launch::is_windows_batch(&role.command) {
            None
        } else {
            first_turn
        };
        adapter.prepare_capture(app_data_dir, &spec.session_id);
        if let Some(hooks) = adapter.status_hooks() {
            let env = hooks.env(&role.args, plan, app_data_dir, spec);
            spec.env.extend(env);
        }
        spec.args = adapter.launch_role_args(&spec.args);
        spec.env
            .extend(adapter.launch_context_env(&crate::runtimes::LaunchContext {
                default_model: default_model.as_deref(),
                env: &spec.env,
                cwd: spec.cwd.as_deref(),
                role_args: &role.args,
                app_data_dir,
                session_id: &spec.session_id,
                resuming: plan.resuming,
                mission: spec.mission,
                model: role.model.as_deref(),
                effort: role.effort.as_deref(),
                codex_speed: role.codex_speed,
                system_prompt: None,
                first_turn,
            }));
        let mut composed: Vec<String> = Vec::new();
        if plan.prepend {
            composed.extend(plan.args.iter().cloned());
            composed.append(&mut spec.args);
        } else {
            composed.append(&mut spec.args);
            composed.extend(plan.args.iter().cloned());
        }
        let first_turn_for_argv =
            crate::runtimes::for_key(&role.runtime).first_turn_argv(first_turn);
        let delivered_via_argv = !first_turn_for_argv.is_empty();
        composed.extend(crate::runtimes::for_key(&role.runtime).mission_dir_args(mission_bus_dir));
        for extra in
            crate::runtimes::for_key(&role.runtime).launch_args(&crate::runtimes::LaunchContext {
                default_model: default_model.as_deref(),
                env: &spec.env,
                cwd: spec.cwd.as_deref(),
                role_args: &role.args,
                app_data_dir,
                session_id: &spec.session_id,
                resuming: plan.resuming,
                model: role.model.as_deref(),
                effort: role.effort.as_deref(),
                codex_speed: role.codex_speed,
                system_prompt: system_prompt_path
                    .map(|path| path.to_string_lossy())
                    .as_deref(),
                first_turn,
                mission: spec.mission,
            })
        {
            composed.push(extra);
        }
        spec.args = composed;
        if is_terminal_shell(role) && !spec.mission {
            crate::shell_integration::inject(spec, app_data_dir, |name| std::env::var(name).ok());
        }
        delivered_via_argv
    }

    /// True iff the `sessions` row for `session_id` is still in the
    /// DB. False if the row was deleted out from under an in-flight
    /// spawn — most commonly `role_delete` removing the role's
    /// sessions, but also covers manual
    /// DB cleanup or any other path that drops the row while a
    /// gated spawn was asleep. Returns false on pool errors so the
    /// caller treats "can't tell" the same as "deleted" and bails
    /// out of the spawn — losing a session on a transient DB hiccup
    /// is preferable to leaving an orphan PTY attached to no row.
    pub(super) fn session_row_exists(pool: &DbPool, session_id: &str) -> bool {
        let Ok(conn) = pool.get() else { return false };
        crate::repo::session::exists(&conn, session_id).unwrap_or(false)
    }

    /// The row's persisted last size: the hint from registration until a
    /// frontend push lands, then whatever `resize` recorded.
    pub(super) fn persisted_size(pool: &DbPool, session_id: &str) -> Option<(u16, u16)> {
        let conn = pool.get().ok()?;
        let row = crate::repo::session::get_row(&conn, session_id).ok()??;
        row.last_cols.zip(row.last_rows)
    }

    pub(super) fn runtime_pid(&self, rt_session: &RuntimeSession) -> Option<i32> {
        self.runtime
            .status(rt_session)
            .ok()
            .flatten()
            .and_then(|status| status.pid)
    }
}
