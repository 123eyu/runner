use super::*;
use crate::runtime_status::models::{option, ModelCatalog, Query, Reason};
use crate::shell_path::LoginShellEnv;
use serde_json::Value;

pub(super) fn query(executable: &str, env: &LoginShellEnv) -> Result<ModelCatalog, Reason> {
    let output = crate::runtime_status::models::run(Query {
        executable,
        args: &["models"],
        stdin: None,
        env,
        timeout: Duration::from_secs(15),
    })?;
    parse(
        &output,
        config_dir(
            &env.vars,
            runner_core::app_paths::home_dir().as_deref(),
            None,
        )
        .as_deref(),
    )
}
fn cache(home: Option<&Path>) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(home?.join("models_cache.json")).ok()?).ok()
}
fn efforts(cache: Option<&Value>, model: &str) -> Vec<String> {
    cache
        .and_then(|cache| cache["models"][model]["info"]["reasoning_efforts"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|effort| effort["id"].as_str())
        .filter(|id| !id.is_empty() && !id.chars().any(|c| c.is_whitespace() || c.is_control()))
        .map(str::to_owned)
        .collect()
}
fn parse(bytes: &[u8], home: Option<&Path>) -> Result<ModelCatalog, Reason> {
    let output = std::str::from_utf8(bytes).map_err(|_| Reason::InvalidOutput)?;
    let cache = cache(home);
    let mut catalog = ModelCatalog::default();
    let mut available = false;
    for line in output.lines().map(str::trim) {
        if let Some(model) = line.strip_prefix("Default model: ") {
            catalog.default_model = trim_some(Some(model)).map(str::to_owned);
        }
        if line == "Available models:" {
            available = true;
            continue;
        }
        if !available {
            continue;
        }
        let Some(row) = line.strip_prefix("* ").or_else(|| line.strip_prefix("- ")) else {
            continue;
        };
        let (id, default) = row
            .strip_suffix(" (default)")
            .map(|id| (id, true))
            .unwrap_or((row, false));
        if id.is_empty()
            || id.starts_with('-')
            || id.chars().any(|c| c.is_whitespace() || c.is_control())
            || catalog.models.iter().any(|model| model.value == id)
        {
            continue;
        }
        if default {
            catalog.default_model = Some(id.into());
        }
        let mut model = option(id.into(), id.into(), None);
        model.supported_efforts = Some(efforts(cache.as_ref(), id));
        catalog.models.push(model);
    }
    if catalog.models.is_empty() {
        return Err(if available {
            Reason::EmptyCatalog
        } else {
            Reason::InvalidOutput
        });
    }
    Ok(catalog)
}
pub(super) fn args(
    model: Option<&str>,
    effort: Option<&str>,
    environment_model: Option<&str>,
    default_model: Option<&str>,
    home: Option<&Path>,
) -> Vec<String> {
    let model = trim_some(model);
    let mut args = model
        .map(|model| strings(&["-m", model]))
        .unwrap_or_default();
    let native = home
        .map(|home| defaults(&home.join("config.toml")))
        .unwrap_or_default();
    let effective = model
        .or(environment_model)
        .or(native.model.as_deref())
        .or(default_model);
    let effort = trim_some(effort);
    if let (Some(effective), Some(effort)) = (effective, effort) {
        if efforts(cache(home).as_ref(), effective)
            .iter()
            .any(|supported| supported == effort)
        {
            args.extend(strings(&["--reasoning-effort", effort]));
        }
    }
    args
}
pub(super) fn environment_model(
    env: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    #[cfg(test)]
    use crate::golden::config_var_os as var_os;
    #[cfg(not(test))]
    use std::env::var_os;
    env.get("GROK_DEFAULT_MODEL")
        .and_then(|value| trim_some(Some(value)))
        .map(str::to_owned)
        .or_else(|| {
            let value = var_os("GROK_DEFAULT_MODEL")?.into_string().ok()?;
            trim_some(Some(&value)).map(str::to_owned)
        })
}
pub(super) fn mcp_entry(
    definition: &crate::ops::mcp::McpServerDefinition,
) -> crate::error::Result<crate::ops::mcp::NativeEntry> {
    let crate::ops::mcp::NativeEntry::Toml(mut table) = crate::ops::mcp::toml_entry(definition)?
    else {
        unreachable!()
    };
    table["type"] = toml_edit::value(match definition {
        crate::ops::mcp::McpServerDefinition::Stdio { .. } => "stdio",
        crate::ops::mcp::McpServerDefinition::Http { .. } => "http",
    });
    if let Some(headers) = table.remove("http_headers") {
        table["headers"] = headers;
        table["type"] = toml_edit::value("http");
    }
    Ok(crate::ops::mcp::NativeEntry::Toml(table))
}

pub(super) fn stored_option<'a>(args: &'a [String], flags: &[&str]) -> Option<&'a str> {
    args.iter()
        .enumerate()
        .filter_map(|(index, arg)| {
            flags.iter().find_map(|flag| {
                if arg == flag {
                    args.get(index + 1).map(String::as_str)
                } else {
                    arg.strip_prefix(&format!("{flag}="))
                        .or_else(|| (*flag == "-m").then(|| arg.strip_prefix("-m")).flatten())
                }
            })
        })
        .next_back()
        .and_then(|value| trim_some(Some(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_catalog_uses_only_cached_model_efforts() {
        let home = tempfile::tempdir().unwrap();
        super::super::tests::model_cache(home.path());
        let output=b"You are logged in with grok.com.\n\nDefault model: account-model\n\nAvailable models:\n  * account-model (default)\n  - other-model\n  - unknown\n  - account-model\n";
        let catalog = parse(output, Some(home.path())).unwrap();
        assert_eq!(catalog.default_model.as_deref(), Some("account-model"));
        assert_eq!(catalog.models.len(), 3);
        assert_eq!(
            catalog.models[0].supported_efforts,
            Some(strings(&["low", "high"]))
        );
        assert_eq!(
            catalog.models[1].supported_efforts,
            Some(strings(&["xhigh"]))
        );
        assert_eq!(catalog.models[2].supported_efforts, Some(Vec::new()));
        for contents in [None, Some("{"), Some("{}")] {
            if let Some(contents) = contents {
                std::fs::write(home.path().join("models_cache.json"), contents).unwrap();
            } else {
                std::fs::remove_file(home.path().join("models_cache.json")).unwrap();
            }
            assert!(parse(output, Some(home.path()))
                .unwrap()
                .models
                .iter()
                .all(|model| model.supported_efforts == Some(Vec::new())));
            assert_eq!(
                args(
                    Some("account-model"),
                    Some("high"),
                    None,
                    None,
                    Some(home.path())
                ),
                strings(&["-m", "account-model"])
            );
        }
        assert_eq!(
            parse(b"Please authenticate", None),
            Err(Reason::InvalidOutput)
        );
        assert_eq!(
            parse(b"Available models:\n- white space\n", None),
            Err(Reason::EmptyCatalog)
        );
        assert_eq!(parse(&[0xff], None), Err(Reason::InvalidOutput));
    }
}
