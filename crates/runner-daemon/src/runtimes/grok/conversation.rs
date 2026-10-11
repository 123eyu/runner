use super::*;

pub(super) fn exists(key: &str, ctx: &ProbeContext<'_>) -> Option<bool> {
    if !is_uuid(key) {
        return Some(false);
    }
    let cwd = ctx
        .cwd
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;
    let cwd = cwd.canonicalize().unwrap_or(cwd);
    let env = ctx
        .role_env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let root = config_dir(&env, Some(&cwd), None)?.join("sessions");
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(false),
        Err(_) => return None,
    };
    Some(entries.flatten().any(|entry| {
        let summary = entry.path().join(key).join("summary.json");
        let Some(document) = std::fs::read(summary)
            .ok()
            .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
        else {
            return false;
        };
        let info = &document["info"];
        info["id"].as_str() == Some(key)
            && info["cwd"].as_str().is_some_and(|saved| {
                let saved = PathBuf::from(saved);
                saved.canonicalize().unwrap_or(saved) == cwd
            })
    }))
}
