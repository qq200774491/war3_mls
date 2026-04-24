use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn archive_dir(base: &str) -> PathBuf {
    let p = PathBuf::from(base);
    std::fs::create_dir_all(&p).ok();
    p
}

fn archive_path(base: &str, script_dir: &str) -> PathBuf {
    let name = Path::new(script_dir)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());
    archive_dir(base).join(format!("{}.json", name))
}

pub fn save_room_archives(
    base: &str,
    script_dir: &str,
    players: &HashMap<i32, crate::player::Player>,
) -> std::io::Result<PathBuf> {
    let path = archive_path(base, script_dir);
    let mut existing: serde_json::Map<String, Value> = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Default::default(),
        }
    } else {
        Default::default()
    };

    for (idx, p) in players {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        existing.insert(
            idx.to_string(),
            serde_json::json!({
                "name": p.name,
                "map_level": p.map_level,
                "map_exp": p.map_exp,
                "played_count": p.played_count,
                "items": p.items,
                "script_archive": p.script_archive,
                "common_archive": p.common_archive,
                "read_archive": p.read_archive,
                "cfg_archive": p.cfg_archive,
                "saved_at": now,
            }),
        );
    }

    let json = serde_json::to_string_pretty(&existing)?;
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn load_player_archives(base: &str, script_name: &str) -> Value {
    let path = archive_dir(base).join(format!("{}.json", script_name));
    if !path.exists() {
        return Value::Object(Default::default());
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or(Value::Object(Default::default())),
        Err(_) => Value::Object(Default::default()),
    }
}

pub fn list_archives(base: &str) -> Vec<Value> {
    let dir = archive_dir(base);
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".json") {
                if let Ok(text) = std::fs::read_to_string(entry.path()) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Map<String, Value>>(&text)
                    {
                        result.push(serde_json::json!({
                            "script": &fname[..fname.len()-5],
                            "players": data.len(),
                            "file": entry.path().to_string_lossy(),
                        }));
                    }
                }
            }
        }
    }
    result
}
