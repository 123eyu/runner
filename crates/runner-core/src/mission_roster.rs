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
            lead: slot.lead,
            position: slot.position,
            role_id: slot.role_id.clone(),
            runtime_override: slot.runtime_override.clone(),
            model_override: slot.model_override.clone(),
            effort_override: slot.effort_override.clone(),
            codex_speed_override: slot.codex_speed_override,
        }
    }
}
