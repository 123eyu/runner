use super::*;

pub(super) async fn run_msg(
    client: &impl Caller,
    command: &MsgCommand,
    context: &BusContext,
) -> Result<ToolResponse, CliError> {
    match command {
        MsgCommand::Post {
            text,
            to,
            mission,
            from,
        } => {
            let mission_id = resolve_scoped_mission(client, mission.as_deref(), context).await?;
            call(
                client,
                "mission_post",
                mission_post_args(&mission_id, text, to, from.as_deref()),
            )
            .await
        }
        MsgCommand::Read { .. } => unreachable!(),
    }
}

pub(super) fn mission_post_args(
    mission_id: &str,
    text: &str,
    to: &[String],
    from: Option<&str>,
) -> Value {
    let to = runner_core::Recipients::new(to.to_vec());
    let mut args = json!({"mission_id": mission_id, "text": text, "to": to});
    insert_string(&mut args, "from", from);
    args
}
