use super::*;

pub(super) async fn run_project(
    client: &impl Caller,
    command: &ProjectCommand,
) -> Result<ToolResponse, CliError> {
    match command {
        ProjectCommand::List => call(client, "project_list", json!({})).await,
        ProjectCommand::Show { project } => {
            let project = resolve_named(client, "project", "project_list", project).await?;
            call(client, "project_get", json!({"id": project.id})).await
        }
        ProjectCommand::Create { name, path } => {
            let path = absolute_path(path.as_deref().unwrap_or(Path::new(".")))?;
            call(client, "project_create", json!({"name": name, "cwd": path})).await
        }
        ProjectCommand::Rename { project, name } => {
            let project = resolve_named(client, "project", "project_list", project).await?;
            call(
                client,
                "project_rename",
                json!({"id": project.id, "name": name}),
            )
            .await
        }
        ProjectCommand::Delete { project, force } => {
            let project = resolve_named(client, "project", "project_list", project).await?;
            let mut response = call(
                client,
                "project_delete",
                json!({"id": project.id, "force": force}),
            )
            .await?;
            response
                .value
                .as_object_mut()
                .unwrap()
                .insert("id".into(), json!(project.id));
            Ok(response)
        }
    }
}
