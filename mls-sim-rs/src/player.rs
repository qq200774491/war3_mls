use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub index: i32,
    pub name: String,
    #[serde(default = "default_one")]
    pub map_level: i32,
    #[serde(default)]
    pub map_exp: i32,
    #[serde(default)]
    pub played_count: i32,
    #[serde(default)]
    pub test_play_time: i32,
    #[serde(skip_deserializing)]
    pub joined_at: f64,
    #[serde(skip_deserializing)]
    pub is_connected: bool,
    #[serde(default)]
    pub items: HashMap<String, i32>,
    #[serde(default)]
    pub script_archive: Option<String>,
    #[serde(default)]
    pub common_archive: HashMap<String, String>,
    #[serde(default)]
    pub read_archive: HashMap<String, String>,
    #[serde(default)]
    pub cfg_archive: HashMap<String, String>,
}

fn default_one() -> i32 {
    1
}

impl Player {
    pub fn new(index: i32, name: String) -> Self {
        let name = if name.is_empty() {
            format!("Player_{}", index)
        } else {
            name
        };
        Self {
            index,
            name,
            map_level: 1,
            map_exp: 0,
            played_count: 0,
            test_play_time: 0,
            joined_at: now_secs(),
            is_connected: true,
            items: HashMap::new(),
            script_archive: None,
            common_archive: HashMap::new(),
            read_archive: HashMap::new(),
            cfg_archive: HashMap::new(),
        }
    }

    pub fn played_time(&self) -> i32 {
        (now_secs() - self.joined_at) as i32
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "index": self.index,
            "name": self.name,
            "map_level": self.map_level,
            "map_exp": self.map_exp,
            "played_time": self.played_time(),
            "played_count": self.played_count,
            "test_play_time": self.test_play_time,
            "is_connected": self.is_connected,
            "items": self.items,
            "script_archive": self.script_archive,
            "common_archive": self.common_archive,
            "read_archive": self.read_archive,
            "cfg_archive": self.cfg_archive,
        })
    }
}
