use super::*;

pub(super) const DEFAULT_FEED_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(not(test))]
pub(super) const FEED_REQUEST_TIMEOUT: Duration = DEFAULT_FEED_REQUEST_TIMEOUT;
#[cfg(test)]
pub(super) const FEED_REQUEST_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub(super) struct FeedFilter {
    pub(super) types: Option<HashSet<String>>,
    pub(super) from: Option<String>,
    pub(super) all: bool,
}

impl FeedFilter {
    pub(super) fn new(
        types: Option<&str>,
        from: Option<&str>,
        all: bool,
    ) -> Result<Self, CliError> {
        Ok(Self {
            types: parse_feed_types(types)?,
            from: from.map(str::to_owned),
            all,
        })
    }

    pub(super) fn is_explicit(&self) -> bool {
        self.types.is_some() || self.from.is_some() || self.all
    }

    pub(super) fn matches(&self, entry: &Value) -> bool {
        let event = entry.get("event").unwrap_or(entry);
        if self
            .from
            .as_deref()
            .is_some_and(|from| field(event, "from") != Some(from))
        {
            return false;
        }
        let event_type = field(event, "type").or_else(|| field(event, "kind"));
        if let Some(types) = &self.types {
            return event_type.is_some_and(|kind| types.contains(kind));
        }
        self.all
            || event
                .pointer("/payload/status/lifecycle")
                .and_then(Value::as_str)
                == Some("error")
            || event
                .pointer("/payload/status/observation/outcome")
                .and_then(Value::as_str)
                == Some("failed")
            || !event_type.is_some_and(|kind| {
                matches!(kind, "session_status" | "runner_status" | "inbox_read")
            })
    }
}

pub(super) fn parse_feed_types(types: Option<&str>) -> Result<Option<HashSet<String>>, CliError> {
    let Some(types) = types else {
        return Ok(None);
    };
    let values = types
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<HashSet<_>>();
    if values.is_empty() {
        return Err(CliError::usage(
            "runner mission feed: --types requires at least one event kind",
        ));
    }
    Ok(Some(values))
}

pub(super) fn filter_feed_response(response: &ToolResponse, filter: &FeedFilter) -> ToolResponse {
    let mut value = response.value.clone();
    if let Some(events) = value.get_mut("events").and_then(Value::as_array_mut) {
        events.retain(|entry| filter.matches(entry));
    }
    ToolResponse {
        raw_json: serde_json::to_string(&value).unwrap_or_else(|_| response.raw_json.clone()),
        value,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn follow_feed(
    client: &impl Caller,
    mission_id: &str,
    since: Option<u64>,
    limit: Option<usize>,
    oldest_first: bool,
    filter: &FeedFilter,
    json: bool,
    quiet: bool,
    writer: &mut impl std::io::Write,
    diagnostics: &mut impl std::io::Write,
) -> Result<(), CliError> {
    let mut cursor = since.unwrap_or(0);
    let follow = async {
        let mut subscription = client.subscribe();
        let mut initial = true;
        let mut seen = HashSet::new();
        let mut session_states = BTreeMap::new();
        loop {
            // Snapshot before reading so a terminal transition cannot skip its final events.
            let snapshot = follow_call(client, "mission_status", json!({"id": mission_id})).await?;
            let end_offset = snapshot.value["last_event_offset"].as_u64().unwrap_or(0);
            loop {
                let response = follow_call(
                    client,
                    "mission_feed",
                    json!({
                        "mission_id": mission_id,
                        "since_offset": if initial { since } else { Some(cursor) },
                        "limit": limit,
                        "order": if initial && !oldest_first { "newest_first" } else { "oldest_first" },
                    }),
                )
                .await?;
                let previous = cursor;
                write_follow_events(&response, filter, json, quiet, &mut seen, writer)?;
                if let Some(next) = response.value["next_offset"].as_u64() {
                    cursor = cursor.max(next);
                }
                initial = false;
                if cursor >= end_offset {
                    break;
                }
                if cursor == previous {
                    return Err(CliError {
                        code: 1,
                        message: "feed cursor did not advance to the mission snapshot".into(),
                    });
                }
            }
            if let Some(sessions) = snapshot.value["sessions"].as_array() {
                for session in sessions {
                    let id = field(session, "id").unwrap_or_default();
                    let status = field(session, "status").unwrap_or_default();
                    let previous = session_states.insert(id.to_owned(), status.to_owned());
                    if matches!(status, "stopped" | "crashed")
                        && previous.as_deref() != Some(status)
                    {
                        let label = field(session, "handle")
                            .map(|handle| format!("@{handle}"))
                            .unwrap_or_else(|| id.to_owned());
                        write_watch_notice(
                            diagnostics,
                            mission_id,
                            &format!("session {label} {status}"),
                        )?;
                    }
                }
            }
            if let Some((reason, failed)) = feed_end_reason(&snapshot.value) {
                if failed {
                    return Err(CliError {
                        code: 1,
                        message: reason.into(),
                    });
                }
                write_watch_notice(diagnostics, mission_id, reason)?;
                return Ok(());
            }
            loop {
                let update = match subscription.recv().await {
                    Ok(update) => update,
                    Err(EventError::Lagged(_)) => break,
                    Err(EventError::Closed) => return Err(ClientError::NotRunning.into()),
                };
                if update.name == "daemon/disconnected" {
                    return Err(ClientError::NotRunning.into());
                }
                let target = update.payload.get("mission_id").and_then(Value::as_str);
                match update.name.as_str() {
                    "event/appended" if target == Some(mission_id) => {
                        let next =
                            update.payload["next_offset"]
                                .as_u64()
                                .ok_or_else(|| CliError {
                                    code: 1,
                                    message: runner_cli::client::VERSION_SKEW_MESSAGE.into(),
                                })?;
                        if next <= cursor {
                            continue;
                        }
                        let response = response(json!({
                            "events": [{"next_offset": next, "event": update.payload["event"]}],
                            "next_offset": next,
                        }))?;
                        write_follow_events(&response, filter, json, quiet, &mut seen, writer)?;
                        cursor = next;
                    }
                    "mission/changed" if target.is_none() || target == Some(mission_id) => break,
                    "session/exit" | "session/updated" | "session/spawned"
                        if target == Some(mission_id) =>
                    {
                        break
                    }
                    _ => {}
                }
            }
        }
    };
    let result = tokio::select! {
        signal = tokio::signal::ctrl_c() => handle_ctrl_c(signal),
        result = follow => result,
    };
    result.map_err(|error| CliError {
        code: error.code,
        message: format!(
            "runner mission feed {mission_id}: watch ended: {}. Use the same Runner executable for `mission show {mission_id} --json`; if still active, resume with `mission feed {mission_id} --since {cursor} --oldest-first --follow --json`.",
            error.message
        ),
    })
}

pub(super) async fn follow_call(
    client: &impl Caller,
    tool: &str,
    args: Value,
) -> Result<ToolResponse, CliError> {
    tokio::time::timeout(FEED_REQUEST_TIMEOUT, call(client, tool, args))
        .await
        .map_err(|_| CliError {
            code: 1,
            message: "watch request timed out; Runner may still be running".into(),
        })?
}

pub(super) fn feed_end_reason(snapshot: &Value) -> Option<(&'static str, bool)> {
    let mission = &snapshot["mission"];
    if field(mission, "status") == Some("aborted") {
        return Some(("mission aborted", true));
    }
    let sessions = snapshot["sessions"].as_array();
    let all_exited = sessions.is_some_and(|sessions| {
        !sessions.is_empty()
            && sessions
                .iter()
                .all(|session| matches!(field(session, "status"), Some("stopped" | "crashed")))
    });
    let archived = mission
        .get("archived_at")
        .is_some_and(|value| !value.is_null());
    let completed = field(mission, "status") == Some("completed");
    if (archived || completed || all_exited)
        && sessions.is_some_and(|sessions| {
            sessions
                .iter()
                .any(|session| field(session, "status") == Some("crashed"))
        })
    {
        Some(("mission watch ended with crashed sessions", true))
    } else if archived {
        Some(("mission archived; watch ended", false))
    } else if completed {
        Some(("mission completed; watch ended", false))
    } else if all_exited {
        Some((
            "all mission sessions exited; watch ended (arm a new watch if resumed)",
            false,
        ))
    } else {
        None
    }
}

pub(super) fn write_watch_notice(
    writer: &mut impl std::io::Write,
    mission_id: &str,
    message: &str,
) -> Result<(), CliError> {
    writeln!(writer, "runner mission feed {mission_id}: {message}")
        .and_then(|()| writer.flush())
        .map_err(|error| CliError {
            code: 1,
            message: format!("watch diagnostic write failed: {error}"),
        })
}

pub(super) fn handle_ctrl_c(result: std::io::Result<()>) -> Result<(), CliError> {
    result.map_err(|error| CliError {
        code: 1,
        message: format!("runner mission feed: Ctrl-C handler failed: {error}"),
    })
}

pub(super) fn write_follow_events(
    response: &ToolResponse,
    filter: &FeedFilter,
    json: bool,
    quiet: bool,
    seen: &mut HashSet<String>,
    writer: &mut impl std::io::Write,
) -> Result<(), CliError> {
    let mut filtered = filter_feed_response(response, filter);
    if let Some(events) = filtered
        .value
        .get_mut("events")
        .and_then(Value::as_array_mut)
    {
        events.retain(|entry| {
            let event = entry.get("event").unwrap_or(entry);
            let key = field(event, "id")
                .map(str::to_owned)
                .or_else(|| entry.get("next_offset").map(Value::to_string))
                .unwrap_or_else(|| event.to_string());
            seen.insert(key)
        });
    }
    output::write_feed_events(&filtered.value, json, quiet, writer).map_err(|error| CliError {
        code: 1,
        message: format!("runner mission feed: write failed: {error}"),
    })
}
