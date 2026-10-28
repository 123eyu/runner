use super::*;

pub(super) async fn run_session(
    client: &impl Caller,
    command: &SessionCommand,
) -> Result<ToolResponse, CliError> {
    match command {
        SessionCommand::List => call(client, "session_list", json!({})).await,
        SessionCommand::Show { session }
        | SessionCommand::Stop { session }
        | SessionCommand::Archive { session } => {
            let session_id = resolve_session(client, session).await?;
            let tool = match command {
                SessionCommand::Show { .. } => "session_get",
                SessionCommand::Stop { .. } => "session_stop",
                SessionCommand::Archive { .. } => "session_archive",
                _ => unreachable!(),
            };
            call(client, tool, json!({"session_id": session_id})).await
        }
        SessionCommand::Resume { session } | SessionCommand::Restart { session } => {
            let id = resolve_session(client, session).await?;
            let tool = if matches!(command, SessionCommand::Resume { .. }) {
                "session_resume"
            } else {
                "session_restart"
            };
            call(client, tool, json!({"session_id": id})).await
        }
    }
}
