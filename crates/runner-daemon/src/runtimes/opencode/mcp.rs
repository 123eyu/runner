use super::*;
use crate::error::{Error, Result};
use crate::ops::mcp::{McpServerDefinition, NativeEntry};

pub(super) fn definition(value: &Value) -> Option<McpServerDefinition> {
    let strings = |key: &str| -> Option<BTreeMap<String, String>> {
        match value.get(key) {
            None => Some(BTreeMap::new()),
            Some(value) => value
                .as_object()?
                .iter()
                .map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
                .collect(),
        }
    };
    match value["type"].as_str()? {
        "local" => {
            let argv = value["command"]
                .as_array()?
                .iter()
                .map(|arg| arg.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()?;
            let (command, args) = argv.split_first()?;
            Some(McpServerDefinition::Stdio {
                command: command.clone(),
                args: args.to_vec(),
                env: strings("environment")?,
            })
        }
        "remote" => Some(McpServerDefinition::Http {
            url: value["url"].as_str()?.to_owned(),
            headers: strings("headers")?,
        }),
        _ => None,
    }
}

pub(super) fn entry(definition: &McpServerDefinition) -> Result<NativeEntry> {
    let value = match definition {
        McpServerDefinition::Stdio { command, args, env } => {
            let mut argv = vec![command.clone()];
            argv.extend(args.clone());
            json!({"type":"local", "command":argv, "environment":env})
        }
        McpServerDefinition::Http { url, headers } => {
            json!({"type":"remote", "url":url, "headers":headers})
        }
    };
    Ok(NativeEntry::Json(value, self::definition))
}
pub(crate) fn read(path: &Path) -> Result<BTreeMap<String, NativeEntry>> {
    if matches!(
        path.file_name().and_then(|v| v.to_str()),
        Some("opencode.json" | "opencode.jsonc")
    ) && path.with_file_name("opencode.json").exists()
        && path.with_file_name("opencode.jsonc").exists()
    {
        return Err(Error::msg("OpenCode merges opencode.json and opencode.jsonc. Runner manages one global file at a time; edit dual-file MCP configuration with OpenCode."));
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
        Err(e) => return Err(e.into()),
    };
    if raw.trim().is_empty() {
        return Ok(Default::default());
    }
    let value = config::parse(&raw)?;
    if !value.is_object() {
        return Err(Error::msg("OpenCode config must be an object"));
    }
    let Some(mcp) = value.get("mcp") else {
        return Ok(Default::default());
    };
    if !mcp.is_object() {
        return Err(Error::msg("OpenCode mcp must be an object"));
    }
    let Some(servers) = mcp.get("servers") else {
        return Ok(Default::default());
    };
    Ok(servers
        .as_object()
        .ok_or_else(|| Error::msg("OpenCode mcp.servers must be an object"))?
        .iter()
        .map(|(k, v)| (k.clone(), NativeEntry::Json(v.clone(), definition)))
        .collect())
}
pub(crate) fn write(
    path: &Path,
    name: &str,
    entry: Option<NativeEntry>,
    merge: bool,
) -> Result<()> {
    let mut raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "{}".into(),
        Err(e) => return Err(e.into()),
    };
    if raw.trim().is_empty() {
        raw = "{}".into();
    }
    let existing = read(path)?;
    if entry.is_none() && !existing.contains_key(name) {
        return Ok(());
    }
    let mut value = match entry {
        Some(NativeEntry::Json(value, _)) => Some(value),
        None => None,
        _ => return Err(Error::msg("OpenCode MCP entry must be JSON")),
    };
    if merge {
        if let (Some(NativeEntry::Json(old, _)), Some(value)) = (existing.get(name), value.as_mut())
        {
            let mut base = old
                .as_object()
                .cloned()
                .ok_or_else(|| Error::msg("OpenCode MCP entry must be an object"))?;
            for key in ["type", "command", "environment", "url", "headers"] {
                base.remove(key);
            }
            let disabled = base.get("disabled").cloned();
            base.extend(
                value
                    .as_object()
                    .ok_or_else(|| Error::msg("OpenCode MCP entry must be an object"))?
                    .clone(),
            );
            if let Some(disabled) = disabled {
                base.insert("disabled".into(), disabled);
            }
            *value = Value::Object(base);
        }
    }
    let locate = |raw: &str, key: &str| -> Result<Option<std::ops::Range<usize>>> {
        // config::replace validates JSONC; use its offset-preserving mask.
        config::member_range(raw, key)
    };
    if let Some(mcp_range) = locate(&raw, "mcp")? {
        let mcp_raw = &raw[mcp_range.clone()];
        let updated = if let Some(servers_range) = locate(mcp_raw, "servers")? {
            let servers = config::replace(&mcp_raw[servers_range.clone()], name, value.as_ref())?;
            let mut updated = mcp_raw.to_owned();
            updated.replace_range(servers_range, &servers);
            updated
        } else {
            config::replace(mcp_raw, "servers", Some(&json!({ name: value })))?
        };
        raw.replace_range(mcp_range, &updated);
    } else {
        raw = config::replace(&raw, "mcp", Some(&json!({"servers": {name: value}})))?;
    }
    let target = if path.is_symlink() {
        path.canonicalize()?
    } else {
        path.to_path_buf()
    };
    write_config_atomically(&target, raw.as_bytes())
}
