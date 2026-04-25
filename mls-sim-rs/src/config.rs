use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::player::Player;

#[derive(Parser, Debug)]
#[command(name = "mls-sim", version, about = "MLS 云脚本本地模拟环境")]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, short, default_value_t = 5000)]
    pub port: u16,

    #[arg(long, short)]
    pub script_dir: Option<String>,

    #[arg(long, default_value = "config.json")]
    pub config: String,

    #[arg(long, help = "Hide the console window (Windows only)")]
    pub console_notwrte: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRoomConfig {
    pub script_dir: String,
    #[serde(default)]
    pub mode_id: i32,
    #[serde(default = "default_players")]
    pub players: Vec<PlayerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub items: std::collections::HashMap<String, i32>,
    #[serde(default)]
    pub map_level: Option<i32>,
    #[serde(default)]
    pub map_exp: Option<i32>,
    #[serde(default)]
    pub played_count: Option<i32>,
    #[serde(default)]
    pub script_archive: Option<String>,
    #[serde(default)]
    pub common_archive: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub read_archive: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub cfg_archive: Option<std::collections::HashMap<String, String>>,
}

fn default_players() -> Vec<PlayerConfig> {
    vec![PlayerConfig {
        index: 0,
        name: "Player_0".into(),
        items: Default::default(),
        map_level: None,
        map_exp: None,
        played_count: None,
        script_archive: None,
        common_archive: None,
        read_archive: None,
        cfg_archive: None,
    }]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_open_browser: bool,
    #[serde(default = "default_archive_dir")]
    pub archive_dir: String,
    pub auto_room: Option<AutoRoomConfig>,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    5000
}
fn default_true() -> bool {
    true
}
fn default_archive_dir() -> String {
    "./archives".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            auto_open_browser: true,
            archive_dir: default_archive_dir(),
            auto_room: None,
        }
    }
}

impl AppConfig {
    pub fn load(cli: &Cli) -> Self {
        let config_path = PathBuf::from(&cli.config);
        let mut config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(text) => serde_json::from_str::<AppConfig>(&text).unwrap_or_default(),
                Err(_) => AppConfig::default(),
            }
        } else {
            AppConfig::default()
        };

        if cli.host != "127.0.0.1" {
            config.host = cli.host.clone();
        }
        if cli.port != 5000 {
            config.port = cli.port;
        }
        if let Some(ref sd) = cli.script_dir {
            config.auto_room = Some(AutoRoomConfig {
                script_dir: sd.clone(),
                mode_id: 0,
                players: default_players(),
            });
        }

        config
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

pub fn build_players_from_config(configs: &[PlayerConfig]) -> HashMap<i32, Player> {
    let configs = if configs.is_empty() {
        vec![PlayerConfig {
            index: 0,
            name: "Player_0".into(),
            items: Default::default(),
            map_level: None,
            map_exp: None,
            played_count: Some(0),
            script_archive: None,
            common_archive: None,
            read_archive: None,
            cfg_archive: None,
        }]
    } else {
        configs.to_vec()
    };

    let mut players = HashMap::new();
    for pc in &configs {
        let mut p = Player::new(pc.index, pc.name.clone());
        if !pc.items.is_empty() {
            p.items = pc.items.clone();
        }
        if let Some(v) = pc.map_level {
            p.map_level = v;
        }
        if let Some(v) = pc.map_exp {
            p.map_exp = v;
        }
        if let Some(v) = pc.played_count {
            p.played_count = v;
        }
        if let Some(ref v) = pc.script_archive {
            p.script_archive = Some(v.clone());
        }
        if let Some(ref v) = pc.common_archive {
            p.common_archive = v.clone();
        }
        if let Some(ref v) = pc.read_archive {
            p.read_archive = v.clone();
        }
        if let Some(ref v) = pc.cfg_archive {
            p.cfg_archive = v.clone();
        }
        players.insert(pc.index, p);
    }
    players
}
