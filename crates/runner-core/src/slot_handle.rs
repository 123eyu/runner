use std::collections::HashSet;

pub fn suggest_slot_handle(base: &str, taken: &HashSet<String>) -> String {
    if !taken.contains(base) {
        return base.to_owned();
    }
    (2..100)
        .map(|index| format!("{base}-{index}"))
        .find(|candidate| !taken.contains(candidate))
        .unwrap_or_else(|| base.to_owned())
}

pub fn validate_slot_handle(handle: &str) -> Option<&'static str> {
    if handle.is_empty() {
        return None;
    }
    let bytes = handle.as_bytes();
    let valid = bytes.len() <= 32
        && bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        });
    (!valid).then_some(
        "Lowercase letters, digits, '-' or '_'; must start with a letter or digit; up to 32 chars.",
    )
}

pub fn slot_handle_error(
    handle: &str,
    existing_handles: &HashSet<String>,
    removed_handles: &HashSet<String>,
    scope: &str,
) -> Option<String> {
    validate_slot_handle(handle).map(ToOwned::to_owned).or_else(|| {
        if removed_handles.contains(handle) {
            let taken = existing_handles.union(removed_handles).cloned().collect();
            let suggestion = suggest_slot_handle(handle, &taken);
            Some(format!("@{handle} was removed from this mission, and a handle isn't reused. Try @{suggestion}."))
        } else {
            existing_handles.contains(handle).then(|| format!("'{handle}' is already used in this {scope}."))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_keep_crew_copy_and_refuse_removed_mission_handles() {
        let current = HashSet::from(["coder".into(), "reviewer-2".into()]);
        let removed = HashSet::from(["reviewer".into()]);
        assert_eq!(
            slot_handle_error("coder", &current, &HashSet::new(), "crew").as_deref(),
            Some("'coder' is already used in this crew.")
        );
        assert_eq!(slot_handle_error("reviewer", &current, &removed, "mission").as_deref(), Some("@reviewer was removed from this mission, and a handle isn't reused. Try @reviewer-3."));
        assert!(slot_handle_error("Bad", &current, &removed, "mission").is_some());
        assert_eq!(
            slot_handle_error("fresh", &current, &removed, "mission"),
            None
        );
        assert_eq!(
            suggest_slot_handle("reviewer", &current.union(&removed).cloned().collect()),
            "reviewer-3"
        );
    }
}
