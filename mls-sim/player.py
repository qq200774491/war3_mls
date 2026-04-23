"""MLS 模拟玩家模型"""

import time


class Player:
    def __init__(self, index: int, name: str = ""):
        self.index = index
        self.name = name or f"Player_{index}"
        self.map_level = 1
        self.map_exp = 0
        self.played_count = 1
        self.test_play_time = 0
        self.joined_at = time.time()
        self.is_connected = True

        # 道具 {item_key: quantity}
        self.items: dict[str, int] = {}
        # 脚本存档（单个 JSON 字符串，最大 1MB）
        self.script_archive: str | None = None
        # 普通存档 {key: value}
        self.common_archive: dict[str, str] = {}
        # 可读存档 {key: value}
        self.read_archive: dict[str, str] = {}
        # 全局只读存档 {key: value}
        self.cfg_archive: dict[str, str] = {}

    def get_played_time(self) -> int:
        return int(time.time() - self.joined_at)

    def to_dict(self) -> dict:
        return {
            "index": self.index,
            "name": self.name,
            "map_level": self.map_level,
            "map_exp": self.map_exp,
            "played_time": self.get_played_time(),
            "played_count": self.played_count,
            "test_play_time": self.test_play_time,
            "is_connected": self.is_connected,
            "items": dict(self.items),
            "script_archive": self.script_archive,
            "common_archive": dict(self.common_archive),
            "read_archive": dict(self.read_archive),
            "cfg_archive": dict(self.cfg_archive),
        }
