use super::*;

pub(super) fn daemon_command(command: &DaemonCommand) -> Result<ToolResponse, CliError> {
    use runner_core::{
        daemon_process,
        protocol::{
            socket::{ConnectError, SocketTransport},
            wire::Hello,
        },
    };
    let paths = daemon_process::NativePaths::resolve()
        .map_err(|error| CliError::from(ClientError::Protocol(error.to_string())))?;
    let endpoint =
        runner_core::app_paths::daemon_endpoint(&paths.app_data_dir, cfg!(debug_assertions));
    let client = SocketTransport::connect(
        &endpoint,
        Hello {
            exe_sha256: String::new(),
            client: "cli".into(),
        },
    )
    .map_err(|error| {
        CliError::from(match error {
            ConnectError::Blocked => ClientError::Blocked,
            ConnectError::NotRunning => ClientError::NotRunning,
            other => ClientError::Protocol(other.to_string()),
        })
    })?;
    let value = match command {
        DaemonCommand::Status => {
            json!({ "running": true, "pid": client.welcome.pid, "started_at": client.welcome.started_at, "exe_sha256": client.welcome.exe_sha256 })
        }
        DaemonCommand::Stop => {
            let started = std::time::Instant::now();
            client
                .shutdown(true)
                .map_err(|error| CliError::from(ClientError::Protocol(error.to_string())))?;
            wait_for_daemon_stop(&paths.app_data_dir, started)
                .map_err(|error| CliError::from(ClientError::Protocol(error.to_string())))?;
            json!({ "stopped": true })
        }
    };
    Ok(ToolResponse {
        raw_json: value.to_string(),
        value,
    })
}

pub(super) fn wait_for_daemon_stop(
    data: &Path,
    started: std::time::Instant,
) -> std::io::Result<()> {
    runner_core::daemon_process::wait_unlocked(
        data,
        runner_core::daemon_process::DAEMON_STOP_TIMEOUT.saturating_sub(started.elapsed()),
    )
}
