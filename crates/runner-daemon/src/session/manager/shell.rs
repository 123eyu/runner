use super::*;

/// A terminal pane or drawer shell: the runtime-only `shell` role that
/// `runtime_direct_role` builds, as opposed to a role-backed shell chat or
/// mission slot. Only these get the OSC 7 shell integration (#575).
pub(super) fn is_terminal_shell(role: &Role) -> bool {
    role.id == format!("runtime:{}", Runtime::Shell.key())
}

pub(super) fn resolve_shell_resume_cwd(
    recorded_cwd: Option<&str>,
    project_cwd: Option<&str>,
    home: Option<&Path>,
) -> Result<(String, Option<Vec<u8>>)> {
    let recorded_path = recorded_cwd
        .map(str::trim)
        .filter(|cwd| !cwd.is_empty())
        .map(Path::new);
    let resolved = recorded_path
        .and_then(|cwd| cwd.ancestors().find(|candidate| candidate.is_dir()))
        .or_else(|| {
            project_cwd
                .map(str::trim)
                .filter(|cwd| !cwd.is_empty())
                .map(Path::new)
                .filter(|cwd| cwd.is_dir())
        })
        .or_else(|| home.filter(|home| home.is_dir()))
        .ok_or_else(|| Error::msg("shell session has no existing working directory fallback"))?;
    let resolved = resolved.to_string_lossy().into_owned();
    let notice = recorded_path
        .filter(|recorded| *recorded != Path::new(&resolved))
        .map(|recorded| {
            let recorded = display_terminal_path(recorded, home);
            let resolved = display_terminal_path(Path::new(&resolved), home);
            format!(
                "\x1b[33mrunner: {recorded} no longer exists\r\n        opened {resolved} instead\x1b[0m\r\n"
            )
            .into_bytes()
        });
    Ok((resolved, notice))
}

fn display_terminal_path(path: &Path, home: Option<&Path>) -> String {
    let Some(home) = home else {
        return path.to_string_lossy().into_owned();
    };
    if path == home {
        return "~".into();
    }
    match path.strip_prefix(home) {
        Ok(relative) => format!("~/{}", relative.to_string_lossy()),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}
