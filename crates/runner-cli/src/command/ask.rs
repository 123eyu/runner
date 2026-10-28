use super::*;

pub(super) async fn run_ask(
    client: &impl Caller,
    args: &AskArgs,
    context: &BusContext,
) -> Result<ToolResponse, CliError> {
    let mission_id = resolve_scoped_mission(client, args.mission.as_deref(), context).await?;
    let (signal_type, payload) = ask_signal(args)?;
    call(
        client,
        "mission_signal",
        mission_signal_args(&mission_id, signal_type, payload, args.from.as_deref()),
    )
    .await
}

pub(super) fn ask_signal(args: &AskArgs) -> Result<(&'static str, Value), CliError> {
    let prompt = args
        .question
        .as_ref()
        .or(args.human.as_ref())
        .ok_or_else(|| CliError::usage("runner ask requires <question> or --human <prompt>"))?;
    let choices = args
        .choices
        .as_deref()
        .map(|choices| {
            choices
                .split(',')
                .map(str::trim)
                .filter(|choice| !choice.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(("ask_human", json!({"prompt": prompt, "choices": choices})))
}
