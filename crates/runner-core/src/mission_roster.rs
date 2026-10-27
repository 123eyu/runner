use serde::{Deserialize, Serialize};

use crate::protocol::model::{CodexSpeed, Slot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionRoster {
    #[serde(deserialize_with = "deserialize_version")]
    pub version: u8,
    pub crew_name: String,
    pub crew_addendum: Option<String>,
    pub slots: Vec<RosterSlot>,
}

fn deserialize_version<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    let version = u8::deserialize(deserializer)?;
    if version != 2 {
        return Err(serde::de::Error::custom(
            "expected mission roster version 2",
        ));
    }
    Ok(version)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterSlot {
    pub id: String,
    pub handle: String,
    #[serde(skip_deserializing, serialize_with = "serialize_no_lead")]
    pub lead: bool,
    pub position: i64,
    pub role_id: String,
    pub runtime_override: Option<String>,
    pub model_override: Option<String>,
    pub effort_override: Option<String>,
    pub codex_speed_override: Option<CodexSpeed>,
}

impl From<&Slot> for RosterSlot {
    fn from(slot: &Slot) -> Self {
        Self {
            id: slot.id.clone(),
            handle: slot.slot_handle.clone(),
            lead: false,
            position: slot.position,
            role_id: slot.role_id.clone(),
            runtime_override: slot.runtime_override.clone(),
            model_override: slot.model_override.clone(),
            effort_override: slot.effort_override.clone(),
            codex_speed_override: slot.codex_speed_override,
        }
    }
}

fn serialize_no_lead<S: serde::Serializer>(_: &bool, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bool(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_two_ignores_legacy_flags_and_always_writes_false() {
        let mut roster: MissionRoster = serde_json::from_value(serde_json::json!({
            "version": 2, "crew_name": "Old", "crew_addendum": null,
            "slots": [{"id":"s", "role_id":"r", "position":0, "handle":"coder", "lead":true}]
        }))
        .unwrap();
        assert!(!roster.slots[0].lead);
        roster.slots[0].lead = true;
        assert_eq!(
            serde_json::to_value(roster).unwrap()["slots"][0]["lead"],
            false
        );
    }
}
