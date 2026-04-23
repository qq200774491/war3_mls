"""MLS 本地模拟测试环境 - Flask Web 服务"""

import json
import os
import sys
import time
from collections import deque

from flask import Flask, request, jsonify, render_template
from flask_socketio import SocketIO, emit

from room import RoomManager, Room, LogEntry, OutEvent
from player import Player

app = Flask(__name__, template_folder="templates", static_folder="static")
app.config["SECRET_KEY"] = "mls-sim-dev"
socketio = SocketIO(app, cors_allowed_origins="*", async_mode="threading")

manager = RoomManager()

# 最近日志缓存（每个房间最多保留 500 条）
log_buffers: dict[str, deque] = {}
event_buffers: dict[str, deque] = {}
MAX_BUFFER = 500


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


# ---- 页面 ----

@app.route("/")
def index():
    return render_template("index.html")


# ---- 启动 ----

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
    print(f"\n  MLS Simulator running at http://localhost:{port}\n")
    socketio.run(app, host="0.0.0.0", port=port, debug=False, allow_unsafe_werkzeug=True)


if __name__ == "__main__":
    main()
