"""MLS 本地模拟测试环境 - Flask Web 服务"""

import json
import os
import sys
import time
from copy import deepcopy
from collections import deque

from flask import Flask, request, jsonify, render_template
from flask_socketio import SocketIO, emit

from room import RoomManager, Room, LogEntry, OutEvent
from player import Player
from storage import save_room_archives, load_player_archives, list_archives

app = Flask(__name__, template_folder="templates", static_folder="static")
app.config["SECRET_KEY"] = "mls-sim-dev"
socketio = SocketIO(app, cors_allowed_origins="*", async_mode="threading")

manager = RoomManager()
APP_VERSION = "0.2.0"
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROFILE_PATH = os.path.join(BASE_DIR, "profiles.json")

# 最近日志缓存（每个房间最多保留 500 条）
log_buffers: dict[str, deque] = {}
event_buffers: dict[str, deque] = {}
MAX_BUFFER = 500


def _public_host() -> str:
    return os.environ.get("MLS_SIM_HOST", "127.0.0.1")


def _load_profiles() -> dict:
    if not os.path.exists(PROFILE_PATH):
        return {}
    try:
        with open(PROFILE_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except (json.JSONDecodeError, OSError):
        return {}


def _save_profiles(profiles: dict):
    with open(PROFILE_PATH, "w", encoding="utf-8") as f:
        json.dump(profiles, f, ensure_ascii=False, indent=2)


def _bridge_config_text(base_url: str, room_id: str, player_index: int, poll_interval: float) -> str:
    return f'''-- MLS Bridge 本地测试配置
-- 由 mls-sim / VSCode 插件生成
return {{
    base_url = "{base_url}",
    room_id = "{room_id}",
    player_index = {player_index},
    poll_interval = {poll_interval},
    req_sign_enable = false,
}}
'''


def _profile_id(name: str) -> str:
    safe = "".join(ch if ch.isalnum() or ch in "-_" else "-" for ch in name.strip())
    return safe or f"profile-{int(time.time())}"


def _attach_callbacks(room: Room):
    """给房间绑定日志和事件回调"""
    room_id = room.id
    log_buffers[room_id] = deque(maxlen=MAX_BUFFER)
    event_buffers[room_id] = deque(maxlen=MAX_BUFFER)

    def on_log(entry: LogEntry):
        d = entry.to_dict()
        log_buffers[room_id].append(d)
        socketio.emit("log", d, namespace="/", to=room_id)

    def on_event(ev: OutEvent):
        d = ev.to_dict()
        event_buffers[room_id].append(d)
        socketio.emit("out_event", d, namespace="/", to=room_id)

    room.on_log.append(on_log)
    room.on_event.append(on_event)


# ---- WebSocket ----

@socketio.on("join_room")
def handle_join_room(data):
    from flask_socketio import join_room
    room_id = data.get("room_id", "")
    join_room(room_id)
    # 发送缓存日志
    for entry in log_buffers.get(room_id, []):
        emit("log", entry)
    for ev in event_buffers.get(room_id, []):
        emit("out_event", ev)

@socketio.on("leave_room")
def handle_leave_room(data):
    from flask_socketio import leave_room
    room_id = data.get("room_id", "")
    leave_room(room_id)


# ---- REST API: 房间管理 ----

@app.route("/api/health", methods=["GET"])
def health():
    return jsonify({
        "ok": True,
        "name": "mls-sim",
        "version": APP_VERSION,
        "host": _public_host(),
        "room_count": len(manager.rooms),
        "rooms": manager.list_rooms(),
        "cwd": os.getcwd(),
        "base_dir": BASE_DIR,
    })


@app.route("/api/rooms", methods=["POST"])
def create_room():
    data = request.get_json(silent=True) or {}
    script_dir = data.get("script_dir", "")
    if not script_dir or not os.path.isdir(script_dir):
        return jsonify({"error": f"Invalid script_dir: {script_dir}"}), 400

    mode_id = int(data.get("mode_id", 0))
    room = manager.create_room(script_dir, mode_id)

    # 添加玩家
    players_cfg = data.get("players", [])
    if not players_cfg:
        players_cfg = [{"index": 0, "name": "Player_0"}]

    for pc in players_cfg:
        p = Player(int(pc.get("index", 0)), pc.get("name", ""))
        if "items" in pc:
            p.items = {str(k): int(v) for k, v in pc["items"].items()}
        if "script_archive" in pc and pc["script_archive"]:
            p.script_archive = str(pc["script_archive"])
        if "common_archive" in pc:
            p.common_archive = {str(k): str(v) for k, v in pc["common_archive"].items()}
        if "read_archive" in pc:
            p.read_archive = {str(k): str(v) for k, v in pc["read_archive"].items()}
        if "cfg_archive" in pc:
            p.cfg_archive = {str(k): str(v) for k, v in pc["cfg_archive"].items()}
        if "map_level" in pc:
            p.map_level = int(pc["map_level"])
        if "map_exp" in pc:
            p.map_exp = int(pc["map_exp"])
        if "played_count" in pc:
            p.played_count = int(pc["played_count"])
        room.add_player(p)

    _attach_callbacks(room)

    # 自动启动
    auto_start = data.get("auto_start", True)
    if auto_start:
        room.start()

    return jsonify(room.to_dict()), 201


# ---- REST API: 开发 Profile / Bridge 配置 ----

@app.route("/api/profiles", methods=["GET"])
def list_profiles():
    profiles = _load_profiles()
    return jsonify(list(profiles.values()))


@app.route("/api/profiles", methods=["POST"])
def create_profile():
    data = request.get_json(silent=True) or {}
    script_dir = data.get("script_dir", "")
    if not script_dir or not os.path.isdir(script_dir):
        return jsonify({"error": f"Invalid script_dir: {script_dir}"}), 400

    name = data.get("name") or os.path.basename(os.path.abspath(script_dir))
    profile = {
        "id": _profile_id(name),
        "name": name,
        "script_dir": os.path.abspath(script_dir),
        "mode_id": int(data.get("mode_id", 0)),
        "players": deepcopy(data.get("players") or [{"index": 0, "name": "Player_0"}]),
        "war3": deepcopy(data.get("war3") or {}),
        "created_at": int(time.time()),
        "updated_at": int(time.time()),
    }

    profiles = _load_profiles()
    profiles[profile["id"]] = profile
    _save_profiles(profiles)
    return jsonify(profile), 201


@app.route("/api/bridge/config", methods=["POST"])
def bridge_config():
    data = request.get_json(silent=True) or {}
    room_id = data.get("room_id", "")
    if room_id and not manager.get_room(room_id):
        return jsonify({"error": f"Room not found: {room_id}"}), 404

    port = int(data.get("port", request.host.split(":")[-1] if ":" in request.host else 5000))
    base_url = data.get("base_url") or f"http://{_public_host()}:{port}"
    player_index = int(data.get("player_index", 0))
    poll_interval = float(data.get("poll_interval", 0.05))
    text = _bridge_config_text(base_url, room_id, player_index, poll_interval)
    return jsonify({
        "base_url": base_url,
        "room_id": room_id,
        "player_index": player_index,
        "poll_interval": poll_interval,
        "content": text,
    })


@app.route("/api/rooms", methods=["GET"])
def list_rooms():
    return jsonify(manager.list_rooms())


@app.route("/api/rooms/<room_id>", methods=["GET"])
def get_room(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    return jsonify(room.to_dict())


@app.route("/api/rooms/<room_id>", methods=["DELETE"])
def delete_room(room_id):
    if manager.destroy_room(room_id):
        log_buffers.pop(room_id, None)
        event_buffers.pop(room_id, None)
        return jsonify({"ok": True})
    return jsonify({"error": "Room not found"}), 404


@app.route("/api/rooms/<room_id>/start", methods=["POST"])
def start_room(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    room.start()
    return jsonify({"ok": True})


@app.route("/api/rooms/<room_id>/stop", methods=["POST"])
def stop_room(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    reason = (request.get_json(silent=True) or {}).get("reason", "GameEnd")
    room.stop(reason)
    try:
        path = save_room_archives(room)
        print(f"  Archives saved: {path}")
    except Exception as e:
        print(f"  Archive save failed: {e}")
    return jsonify({"ok": True})


# ---- REST API: 玩家管理 ----

@app.route("/api/rooms/<room_id>/players", methods=["POST"])
def add_player(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    data = request.get_json(silent=True) or {}
    p = Player(int(data.get("index", 0)), data.get("name", ""))
    if "items" in data:
        p.items = {str(k): int(v) for k, v in data["items"].items()}
    room.add_player(p)
    return jsonify(p.to_dict()), 201


@app.route("/api/rooms/<room_id>/players/<int:idx>", methods=["PUT"])
def update_player(room_id, idx):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    p = room.get_player(idx)
    if not p:
        return jsonify({"error": "Player not found"}), 404
    data = request.get_json(silent=True) or {}
    if "name" in data:
        p.name = data["name"]
    if "items" in data:
        p.items = {str(k): int(v) for k, v in data["items"].items()}
    if "map_level" in data:
        p.map_level = int(data["map_level"])
    if "map_exp" in data:
        p.map_exp = int(data["map_exp"])
    if "script_archive" in data:
        p.script_archive = data["script_archive"] if data["script_archive"] else None
    if "common_archive" in data:
        p.common_archive = {str(k): str(v) for k, v in data["common_archive"].items()}
    if "read_archive" in data:
        p.read_archive = {str(k): str(v) for k, v in data["read_archive"].items()}
    if "cfg_archive" in data:
        p.cfg_archive = {str(k): str(v) for k, v in data["cfg_archive"].items()}
    return jsonify(p.to_dict())


@app.route("/api/rooms/<room_id>/players/<int:idx>", methods=["DELETE"])
def remove_player(room_id, idx):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    room.remove_player(idx)
    return jsonify({"ok": True})


@app.route("/api/rooms/<room_id>/players/<int:idx>/leave", methods=["POST"])
def player_leave(room_id, idx):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    reason = (request.get_json(silent=True) or {}).get("reason", "Disconnect")
    room.fire_player_leave(idx, reason)
    return jsonify({"ok": True})


@app.route("/api/rooms/<room_id>/players/<int:idx>/join", methods=["POST"])
def player_join(room_id, idx):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    reason = (request.get_json(silent=True) or {}).get("reason", "Connect")
    room.fire_player_join(idx, reason)
    return jsonify({"ok": True})


@app.route("/api/rooms/<room_id>/players/<int:idx>/exit", methods=["POST"])
def player_exit(room_id, idx):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    reason = (request.get_json(silent=True) or {}).get("reason", "Logout")
    room.fire_player_exit(idx, reason)
    return jsonify({"ok": True})


# ---- REST API: 事件发送 ----

@app.route("/api/rooms/<room_id>/events", methods=["POST"])
def send_event(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    data = request.get_json(silent=True) or {}
    ename = data.get("ename", "")
    evalue = data.get("evalue", "")
    player_index = int(data.get("player_index", 0))
    if not ename:
        return jsonify({"error": "ename is required"}), 400
    room.send_event(ename, evalue, player_index)
    return jsonify({"ok": True})


# ---- REST API: 状态 ----

@app.route("/api/rooms/<room_id>/state", methods=["GET"])
def get_state(room_id):
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": "Room not found"}), 404
    return jsonify(room.to_dict())


# ---- REST API: 存档持久化 ----

@app.route("/api/archives", methods=["GET"])
def get_archives():
    return jsonify(list_archives())


@app.route("/api/archives/<script_name>", methods=["GET"])
def get_archive(script_name):
    archives = load_player_archives(script_name)
    return jsonify(archives)


# ---- REST API: Bridge（客户端通讯桥）----

@app.route("/api/bridge/login", methods=["POST"])
def bridge_login():
    """客户端登录到指定房间"""
    data = request.get_json(silent=True) or {}
    room_id = data.get("room_id", "")
    player_index = int(data.get("player_index", 0))
    name = data.get("name", "")

    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": f"Room not found: {room_id}"}), 404

    p = room.get_player(player_index)
    if not p:
        p = Player(player_index, name or f"Player_{player_index}")
        room.add_player(p)

    return jsonify({"ok": True, "player_index": player_index, "room_id": room_id})


@app.route("/api/bridge/event", methods=["POST"])
def bridge_event():
    """客户端发送事件给云脚本"""
    data = request.get_json(silent=True) or {}
    room_id = data.get("room_id", "")
    player_index = int(data.get("player_index", 0))
    ename = data.get("ename", "")
    evalue = data.get("evalue", "")

    if not room_id or not ename:
        return jsonify({"error": "room_id and ename are required"}), 400

    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": f"Room not found: {room_id}"}), 404

    room.send_event(ename, evalue, player_index)
    return jsonify({"ok": True})


@app.route("/api/bridge/poll/<room_id>/<int:player_index>", methods=["GET"])
def bridge_poll(room_id, player_index):
    """客户端轮询云脚本返回的事件"""
    room = manager.get_room(room_id)
    if not room:
        return jsonify({"error": f"Room not found: {room_id}"}), 404

    events = room.poll_events(player_index)
    return jsonify({"events": events})


@app.route("/api/bridge/rooms", methods=["GET"])
def bridge_list_rooms():
    """客户端查询可用房间列表"""
    rooms = manager.list_rooms()
    result = [{"id": r["id"], "status": r["status"], "player_count": r["player_count"], "mode_id": r["mode_id"]} for r in rooms]
    return jsonify(result)


# ---- 页面 ----

@app.route("/")
def index():
    return render_template("index.html")


# ---- 启动 ----

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
    host = sys.argv[2] if len(sys.argv) > 2 else _public_host()
    os.environ["MLS_SIM_HOST"] = host
    print(f"\n  MLS Simulator running at http://{host}:{port}\n")
    socketio.run(app, host=host, port=port, debug=False, allow_unsafe_werkzeug=True)


if __name__ == "__main__":
    main()
