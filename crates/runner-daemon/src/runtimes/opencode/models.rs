use super::*;
use crate::runtime_status::models::{option, ModelCatalog, Query, Reason};
use crate::shell_path::LoginShellEnv;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use crate::session::process::{prepare_headless_fork, ProcessTree};

pub(super) fn query(executable: &str, env: &LoginShellEnv) -> Result<ModelCatalog, Reason> {
    query_with_timeout(executable, env, Duration::from_secs(15))
}

pub(super) fn query_with_timeout(
    executable: &str,
    env: &LoginShellEnv,
    timeout: Duration,
) -> Result<ModelCatalog, Reason> {
    let deadline = Instant::now() + timeout;
    let mut env = env.clone();
    env.vars = config::effective_env(&env.vars);
    if let Ok(mut config) = config::parse(
        env.vars
            .get("OPENCODE_CONFIG_CONTENT")
            .map(String::as_str)
            .unwrap_or("{}"),
    ) {
        if let Some(object) = config.as_object_mut() {
            object.insert("update".into(), json!("disable"));
            env.vars
                .insert("OPENCODE_CONFIG_CONTENT".into(), config.to_string());
        }
    }
    env.vars.insert(
        "OPENCODE_PASSWORD".into(),
        uuid::Uuid::new_v4().simple().to_string(),
    );
    let output = tempfile::NamedTempFile::new().map_err(|_| Reason::QueryFailed)?;
    let mut command = Command::new(executable);
    command
        .args(["serve", "--stdio", "--hostname", "127.0.0.1", "--port", "0"])
        .envs(&env.vars)
        .env("PATH", crate::runtime_status::direct_chat_path(&env))
        .stdin(Stdio::piped())
        .stdout(output.reopen().map_err(|_| Reason::QueryFailed)?)
        .stderr(Stdio::null());
    if let Some(home) = runner_core::app_paths::home_dir() {
        command.current_dir(home);
    }
    prepare_headless_fork(&mut command);
    let mut child = command.spawn().map_err(|_| Reason::QueryFailed)?;
    let tree = match ProcessTree::adopt(child.id()) {
        Ok(tree) => tree,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Reason::QueryFailed);
        }
    };
    let mut server = Server { child, tree };
    let url = loop {
        if Instant::now() >= deadline {
            return Err(Reason::Timeout);
        }
        if server
            .child
            .try_wait()
            .map_err(|_| Reason::QueryFailed)?
            .is_some()
        {
            return Err(Reason::QueryFailed);
        }
        let mut bytes = Vec::new();
        output
            .reopen()
            .map_err(|_| Reason::QueryFailed)?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Reason::QueryFailed)?;
        if bytes.len() > 64 * 1024 {
            return Err(Reason::InvalidOutput);
        }
        if let Some(end) = bytes.iter().position(|byte| *byte == b'\n') {
            let handshake: Value =
                serde_json::from_slice(&bytes[..end]).map_err(|_| Reason::InvalidOutput)?;
            let url = handshake["url"].as_str().ok_or(Reason::InvalidOutput)?;
            let parsed = reqwest::Url::parse(url).map_err(|_| Reason::InvalidOutput)?;
            if parsed.scheme() != "http"
                || parsed.host_str() != Some("127.0.0.1")
                || parsed.port().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.query().is_some()
                || parsed.fragment().is_some()
                || parsed.path() != "/"
            {
                return Err(Reason::InvalidOutput);
            }
            break url.to_owned();
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    // The first snapshot can precede built-in provider/plugin settlement.
    // Keep this server alive while retrying; new standalone queries restart it.
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Reason::EmptyCatalog);
        }
        let output = crate::runtime_status::models::run(Query {
            executable,
            args: &["api", "--server", &url, "GET", "/api/model"],
            stdin: None,
            env: &env,
            timeout: remaining.min(Duration::from_secs(3)),
        })?;
        match parse(&output) {
            Err(Reason::EmptyCatalog) => {
                std::thread::sleep(remaining.min(Duration::from_millis(150)));
            }
            result => return result,
        }
    }
}

struct Server {
    child: Child,
    tree: ProcessTree,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.tree.terminate();
        let _ = self.child.wait();
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<ModelCatalog, Reason> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| Reason::InvalidOutput)?;
    let rows = value["data"].as_array().ok_or(Reason::InvalidOutput)?;
    let mut catalog = ModelCatalog::default();
    for row in rows {
        if row["enabled"] == false {
            continue;
        }
        let (Some(provider), Some(id)) = (row["providerID"].as_str(), row["id"].as_str()) else {
            return Err(Reason::InvalidOutput);
        };
        let valid = |v: &str| {
            !v.is_empty()
                && !v
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control() || c == '#')
        };
        if !valid(provider) || provider.contains('/') || !valid(id) {
            return Err(Reason::InvalidOutput);
        }
        let value = format!("{provider}/{id}");
        if catalog.models.iter().any(|row| row.value == value) {
            continue;
        }
        let mut model = option(value.clone(), value, None);
        let mut efforts = Vec::new();
        if let Some(variants) = row.get("variants") {
            for variant in variants.as_array().ok_or(Reason::InvalidOutput)? {
                let id = variant["id"].as_str().ok_or(Reason::InvalidOutput)?;
                if !valid(id) {
                    return Err(Reason::InvalidOutput);
                }
                if !efforts.iter().any(|effort| effort == id) {
                    efforts.push(id.to_owned());
                }
            }
        }
        model.supported_efforts = Some(efforts);
        catalog.models.push(model);
    }
    if catalog.models.is_empty() {
        return Err(Reason::EmptyCatalog);
    }
    Ok(catalog)
}
