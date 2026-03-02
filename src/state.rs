use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    NeedsHelp,
    Done,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Active => write!(f, "active"),
            Status::Inactive => write!(f, "inactive"),
            Status::NeedsHelp => write!(f, "needs help"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionStatus {
    pub status: Status,
    pub last_updated: String,
    pub project_path: Option<String>,
    pub notification_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookState {
    pub sessions: HashMap<String, SessionStatus>,
}

impl Default for HookState {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

pub fn state_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory must exist")
        .join(".claude")
        .join("mccm")
        .join("state.json")
}

pub fn read_hook_state() -> HookState {
    let path = state_file_path();
    if !path.exists() {
        return HookState::default();
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}
