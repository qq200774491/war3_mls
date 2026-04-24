"""MLS 存档持久化 - 将玩家存档保存到本地 JSON 文件"""

import json
import os
import time

ARCHIVE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "archives")


def _ensure_dir():
    os.makedirs(ARCHIVE_DIR, exist_ok=True)


def _archive_path(script_dir: str) -> str:
    safe_name = os.path.basename(os.path.normpath(script_dir))
    return os.path.join(ARCHIVE_DIR, f"{safe_name}.json")


def save_room_archives(room):
    """房间停止时持久化所有玩家存档"""
    _ensure_dir()
    path = _archive_path(room.script_dir)

    existing = {}
    if os.path.exists(path):
        try:
            with open(path, "r", encoding="utf-8") as f:
                existing = json.load(f)
        except (json.JSONDecodeError, OSError):
            existing = {}

    for idx, p in room.players.items():
        key = str(idx)
        existing[key] = {
            "name": p.name,
            "map_level": p.map_level,
            "map_exp": p.map_exp,
            "played_count": p.played_count,
            "items": dict(p.items),
            "script_archive": p.script_archive,
            "common_archive": dict(p.common_archive),
            "read_archive": dict(p.read_archive),
            "cfg_archive": dict(p.cfg_archive),
            "saved_at": time.strftime("%Y-%m-%d %H:%M:%S"),
        }

    with open(path, "w", encoding="utf-8") as f:
        json.dump(existing, f, ensure_ascii=False, indent=2)

    return path


def load_player_archives(script_dir: str) -> dict:
    """加载指定脚本目录的存档数据，返回 {player_index_str: archive_dict}"""
    path = _archive_path(script_dir)
    if not os.path.exists(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return {}


def list_archives() -> list[dict]:
    """列出所有已保存的存档"""
    _ensure_dir()
    result = []
    for fname in os.listdir(ARCHIVE_DIR):
        if fname.endswith(".json"):
            path = os.path.join(ARCHIVE_DIR, fname)
            try:
                with open(path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                result.append({
                    "script": fname[:-5],
                    "players": len(data),
                    "file": path,
                })
            except (json.JSONDecodeError, OSError):
                continue
    return result
