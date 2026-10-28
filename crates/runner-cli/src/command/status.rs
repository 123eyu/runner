use super::*;

pub(super) fn status_response(
    client: &SocketClient,
    context: &BusContext,
) -> Result<ToolResponse, CliError> {
    let debug = cfg!(debug_assertions);
    let app_data = runner_core::app_paths::app_data_dir(debug)
        .ok_or_else(|| CliError::usage("Runner app data directory could not be resolved"))?;
    let sidecar = app_data
        .join("bin")
        .join(format!("runner{}", std::env::consts::EXE_SUFFIX));
    let home = runner_core::app_paths::home_dir()
        .ok_or_else(|| CliError::usage("Runner home directory could not be resolved"))?;
    let skills = skill_statuses(&home, debug);
    let command = command_install_status(&home, &sidecar, &app_data, debug);
    let mode = match context {
        BusContext::Mission(mission) => {
            format!(
                "inside mission {} as @{}",
                mission.mission_id, mission.handle
            )
        }
        BusContext::OffBus => "outside a mission".to_owned(),
        BusContext::Partial { .. } => unreachable!(),
    };
    response(json!({
        "cli_version": env!("CARGO_PKG_VERSION"),
        "app_version": client.app_version(),
        "socket": client.endpoint().to_string(),
        "sidecar": sidecar,
        "sidecar_present": sidecar.is_file(),
        "mode": mode,
        "command": command,
        "skills": skills,
    }))
}

pub(super) fn command_install_status(
    home: &Path,
    sidecar: &Path,
    app_data: &Path,
    debug: bool,
) -> Value {
    let process_path = std::env::var("PATH").unwrap_or_default();
    command_install_status_with_path(home, sidecar, app_data, debug, &process_path)
}

pub(super) fn command_install_status_with_path(
    home: &Path,
    sidecar: &Path,
    app_data: &Path,
    debug: bool,
    process_path: &str,
) -> Value {
    #[cfg(windows)]
    let _ = home;
    #[cfg(windows)]
    let status = {
        let sidecar_dir = sidecar.parent().unwrap_or(Path::new(""));
        let user_path = read_windows_user_path().unwrap_or_default();
        let mut search_path = path_without_runner_entries(
            process_path,
            app_data,
            runner_core::command_install::PathStyle::Windows,
        );
        if !search_path.is_empty() {
            search_path.push(';');
        }
        search_path.push_str(&sidecar_dir.to_string_lossy());
        runner_core::command_install::inspect_windows_command(
            sidecar_dir,
            &user_path,
            &search_path,
            debug,
        )
    };
    #[cfg(not(windows))]
    let status = runner_core::command_install::inspect_unix_command(
        sidecar,
        &path_without_runner_entries(
            process_path,
            app_data,
            runner_core::command_install::PathStyle::Unix,
        ),
        &home.join(".local/bin"),
        Path::new("/usr/local/bin"),
        debug,
    );
    serde_json::to_value(status).expect("command status is serializable")
}

pub(super) fn path_without_runner_entries(
    search_path: &str,
    app_data: &Path,
    style: runner_core::command_install::PathStyle,
) -> String {
    let separator = match style {
        runner_core::command_install::PathStyle::Unix => ":",
        runner_core::command_install::PathStyle::Windows => ";",
    };
    runner_core::command_install::path_entries(search_path, style)
        .into_iter()
        .filter(|entry| !runner_core::command_install::path_is_within(entry, app_data, style))
        .map(|entry| entry.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(separator)
}

#[cfg(windows)]
pub(super) fn read_windows_user_path() -> Result<String, CliError> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
    };

    let environment = "Environment"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let name = "Path"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut key = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            environment.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(CliError {
            code: 1,
            message: format!("open HKCU\\Environment failed with code {status}"),
        });
    }
    let mut bytes = 0;
    let status = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut bytes,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        unsafe { RegCloseKey(key) };
        return Ok(String::new());
    }
    if status != ERROR_SUCCESS {
        unsafe { RegCloseKey(key) };
        return Err(CliError {
            code: 1,
            message: format!("read HKCU\\Environment Path size failed with code {status}"),
        });
    }
    let mut data = vec![0u16; bytes as usize / 2];
    let status = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            data.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    unsafe { RegCloseKey(key) };
    if status != ERROR_SUCCESS {
        return Err(CliError {
            code: 1,
            message: format!("read HKCU\\Environment Path failed with code {status}"),
        });
    }
    while data.last() == Some(&0) {
        data.pop();
    }
    Ok(String::from_utf16_lossy(&data))
}

pub(super) fn skill_statuses(home: &Path, debug: bool) -> Vec<Value> {
    let skill_name = runner_core::runner_skill_name(debug);
    runner_core::RUNNER_SKILL_ROOTS
        .iter()
        .map(|relative| {
            let folder = home.join(relative).join(skill_name);
            let state = if !folder.exists() {
                "missing"
            } else if folder.join(runner_core::RUNNER_SKILL_MARKER).is_file() {
                "managed"
            } else {
                "foreign"
            };
            json!({"root": home.join(relative), "folder": folder, "state": state})
        })
        .collect()
}
