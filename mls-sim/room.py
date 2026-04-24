"""MLS 模拟房间 - 管理 Lua VM、玩家、定时器、事件"""

import json
import os
import re
import threading
import time
import traceback
from collections import deque
from queue import Queue, Empty

from lupa import lua53 as lua

from player import Player

# MLS 错误码
ERR_OK = 0
ERR_UNKNOWN = 1
ERR_ROOM_NOT_EXIST = 2
ERR_PLAYER_NOT_EXIST = 3
ERR_EVENT_KEY_LEN = 4
ERR_EVENT_KEY_CONTENT = 5
ERR_EVENT_VALUE_LEN = 6
ERR_EVENT_VALUE_CONTENT = 7
ERR_ARCHIVE_KEY_LEN = 8
ERR_ARCHIVE_VALUE_LEN = 9
ERR_TEXT_TOO_LONG = 10
ERR_SCRIPT_ARCHIVE_TOO_LONG = 11
ERR_ITEM_NOT_ENOUGH = 1259
ERR_ITEM_NOT_FOUND = 10133

MAX_EVENT_NAME_LEN = 32
MAX_EVENT_DATA_LEN = 900
MAX_SCRIPT_ARCHIVE_LEN = 1024 * 1024
MAX_LOG_LEN = 2000

EVENT_NAME_PATTERN = re.compile(r'^[a-zA-Z0-9:_]+$')


class LogEntry:
    def __init__(self, level: str, source: str, message: str, room_id: str, player_index: int = -1):
        self.timestamp = time.time()
        self.level = level
        self.source = source
        self.message = message
        self.room_id = room_id
        self.player_index = player_index

    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "level": self.level,
            "source": self.source,
            "message": self.message,
            "room_id": self.room_id,
            "player_index": self.player_index,
        }


class OutEvent:
    def __init__(self, player_index: int, ename: str, evalue: str, room_id: str):
        self.timestamp = time.time()
        self.player_index = player_index
        self.ename = ename
        self.evalue = evalue
        self.room_id = room_id

    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "player_index": self.player_index,
            "ename": self.ename,
            "evalue": self.evalue,
            "room_id": self.room_id,
        }


class Room:
    _id_counter = 0
    _id_lock = threading.Lock()

    def __init__(self, script_dir: str, mode_id: int = 0):
        with Room._id_lock:
            Room._id_counter += 1
            self.id = f"room-{Room._id_counter:03d}"

        self.script_dir = os.path.abspath(script_dir)
        self.mode_id = mode_id
        self.status = "created"  # created / running / stopped / error
        self.error_message = ""

        self.players: dict[int, Player] = {}
        self.start_ts = int(time.time())
        self.loaded_ts = 0

        # Lua VM
        self._lua: lua.LuaRuntime | None = None
        self._event_handlers: dict[str, list[dict]] = {}
        self._next_event_id = 1
        self._trans_id_counter = 0

        # 定时器
        self._timers: list[threading.Timer] = []
        self._timer_lock = threading.Lock()

        # 事件队列（外部 -> room 线程）
        self._event_queue: Queue = Queue()
        # 输出回调
        self.on_log: list = []  # list of callbacks(LogEntry)
        self.on_event: list = []  # list of callbacks(OutEvent)

        # per-player 出站事件队列（供 Bridge API 轮询）
        self._out_queues: dict[int, deque] = {}

        # room 线程
        self._thread: threading.Thread | None = None
        self._running = False

        # 日志频率限制
        self._log_count = 0
        self._log_window_start = time.time()
        self._log_fused = False

    # ---- 玩家管理 ----

    def add_player(self, player: Player):
        self.players[player.index] = player

    def remove_player(self, index: int):
        self.players.pop(index, None)

    def get_player(self, index: int) -> Player | None:
        return self.players.get(index)

    # ---- 生命周期 ----

    def start(self):
        if self.status == "running":
            return
        self._running = True
        self._thread = threading.Thread(target=self._run_loop, name=f"room-{self.id}", daemon=True)
        self._thread.start()

    def stop(self, reason: str = "GameEnd"):
        if self.status != "running":
            return
        self._event_queue.put(("_system_stop", reason))

    def destroy(self):
        self._running = False
        self._cleanup_timers()
        if self._thread and self._thread.is_alive():
            self._event_queue.put(("_system_destroy", ""))
            self._thread.join(timeout=3)
        self.status = "stopped"

    # ---- Room 线程主循环 ----

    def _run_loop(self):
        try:
            self._init_lua()
            self.status = "running"
            self._load_and_run_script()
            self.loaded_ts = int(time.time())
            self._fire_room_loaded()
        except Exception as e:
            self.status = "error"
            self.error_message = str(e)
            self._emit_log("ERR", "System", f"Room start failed: {e}\n{traceback.format_exc()}")
            self._emit_out_event(-1, "_mlroomfail", str(e))
            return

        while self._running:
            try:
                event = self._event_queue.get(timeout=0.05)
                if event is None:
                    continue
                cmd = event[0]
                if cmd == "_system_stop":
                    self._fire_room_over(event[1])
                    break
                elif cmd == "_system_destroy":
                    break
                elif cmd == "_dispatch_event":
                    _, ename, evalue, player_index = event
                    self._dispatch_event(ename, evalue, player_index)
                elif cmd == "_timer_callback":
                    _, lua_func = event
                    try:
                        lua_func()
                    except Exception as e:
                        self._emit_log("ERR", "Timer", f"Timer callback error: {e}")
            except Empty:
                continue
            except Exception as e:
                self._emit_log("ERR", "System", f"Event loop error: {e}")

        self._cleanup_timers()
        self._lua = None
        if self.status == "running":
            self.status = "stopped"

    # ---- Lua VM 初始化 ----

    def _init_lua(self):
        self._lua = lua.LuaRuntime(unpack_returned_tuples=True)
        self._inject_json()
        self._inject_log()
        self._inject_timer()
        self._inject_event_api()
        self._inject_player_api()
        self._inject_room_api()
        self._inject_item_api()
        self._inject_archive_api()
        self._inject_control_api()

    def _load_and_run_script(self):
        main_lua = os.path.join(self.script_dir, "main.lua")
        if not os.path.exists(main_lua):
            raise FileNotFoundError(f"main.lua not found in {self.script_dir}")

        # 用 Python 替代 Lua 的文件加载，完全绕过中文路径问题
        room = self
        loaded_modules: dict[str, object] = {}

        # 当前脚本目录栈，用于支持 require 中的相对路径
        dir_stack = [self.script_dir]

        def py_require(modname):
            modname_str = str(modname)

            # 已加载过的模块直接返回
            if modname_str in loaded_modules:
                return loaded_modules[modname_str]

            # 支持 "../xxx" 和 "xxx.yyy" 两种写法
            if "/" in modname_str or "\\" in modname_str:
                rel_path = modname_str
            else:
                rel_path = modname_str.replace(".", "/")

            # 从当前目录和脚本根目录搜索
            search_dirs = [dir_stack[-1], room.script_dir]
            fpath = None
            for sdir in search_dirs:
                for candidate in [
                    os.path.normpath(os.path.join(sdir, rel_path + ".lua")),
                    os.path.normpath(os.path.join(sdir, rel_path, "init.lua")),
                ]:
                    if os.path.exists(candidate):
                        fpath = candidate
                        break
                if fpath:
                    break

            if not fpath:
                raise RuntimeError(f"module '{modname_str}' not found")

            with open(fpath, "r", encoding="utf-8") as f:
                code = f.read()

            # 切换当前目录到模块文件所在目录
            mod_dir = os.path.dirname(fpath)
            dir_stack.append(mod_dir)
            try:
                # 标记为正在加载（防止循环依赖）
                loaded_modules[modname_str] = True
                safe_load = room._lua.eval("function(code, name) local f, e = load(code, name); return f, e end")
                chunk, err = safe_load(code, "@" + modname_str)
                if chunk is None:
                    raise RuntimeError(f"Failed to load {modname_str}: {err}")
                result = chunk()
                loaded_modules[modname_str] = result if result is not None else True
                return loaded_modules[modname_str]
            finally:
                dir_stack.pop()

        self._lua.globals()["require"] = py_require

        # 执行 main.lua
        with open(main_lua, "r", encoding="utf-8") as f:
            code = f.read()
        safe_load = self._lua.eval("function(code, name) local f, e = load(code, name); return f, e end")
        chunk, err = safe_load(code, "@main.lua")
        if chunk is None:
            raise RuntimeError(f"Failed to load main.lua: {err}")
        chunk()

    # ---- JSON 注入 ----

    def _inject_json(self):
        # 使用纯 Lua JSON 实现，直接内嵌
        json_lua_code = _get_json_lua()
        self._lua.execute(json_lua_code)

    # ---- Log API 注入 ----

    def _inject_log(self):
        room = self

        def make_log_func(level):
            def log_func(fmt, *args):
                try:
                    if args:
                        msg = room._lua_string_format(fmt, *args)
                    else:
                        msg = str(fmt) if fmt is not None else ""
                except Exception:
                    msg = str(fmt)
                if len(msg) > MAX_LOG_LEN:
                    msg = msg[:MAX_LOG_LEN]
                room._emit_log(level, "Lua", msg)
            return log_func

        log_table = self._lua.eval("{}")
        log_table["Debug"] = make_log_func("DBG")
        log_table["Info"] = make_log_func("INF")
        log_table["Error"] = make_log_func("ERR")
        self._lua.globals()["Log"] = log_table

    def _lua_string_format(self, fmt, *args):
        """在 Lua 中调用 string.format"""
        sf = self._lua.eval("string.format")
        return sf(fmt, *args)

    def _emit_log(self, level: str, source: str, message: str):
        # 频率限制
        now = time.time()
        if now - self._log_window_start > 100:
            self._log_window_start = now
            self._log_count = 0
            self._log_fused = False

        if self._log_fused:
            return

        self._log_count += 1
        if self._log_count > 1000:
            self._log_fused = True
            message = "[LOG FUSE] Log rate exceeded 1000/100s, logs fused until next window"

        entry = LogEntry(level, source, message, self.id)
        for cb in self.on_log:
            try:
                cb(entry)
            except Exception:
                pass

    # ---- Timer API 注入 ----

    def _inject_timer(self):
        room = self

        def timer_after(seconds, callback):
            def fire():
                if room._running:
                    room._event_queue.put(("_timer_callback", callback))

            t = threading.Timer(float(seconds), fire)
            t.daemon = True
            with room._timer_lock:
                room._timers.append(t)
            t.start()

        def timer_new_ticker(seconds, callback):
            cancelled = threading.Event()

            def tick_loop():
                while not cancelled.is_set() and room._running:
                    cancelled.wait(float(seconds))
                    if not cancelled.is_set() and room._running:
                        room._event_queue.put(("_timer_callback", callback))

            t = threading.Thread(target=tick_loop, daemon=True)
            t.start()

            # 返回一个带 Cancel 方法的 Lua table
            ticker = room._lua.eval("{}")
            ticker["Cancel"] = lambda: cancelled.set()
            return ticker

        timer_table = self._lua.eval("{}")
        timer_table["After"] = timer_after
        timer_table["NewTicker"] = timer_new_ticker
        self._lua.globals()["Timer"] = timer_table

    def _cleanup_timers(self):
        with self._timer_lock:
            for t in self._timers:
                t.cancel()
            self._timers.clear()

    # ---- Event API 注入 ----

    def _inject_event_api(self):
        room = self

        def register_event(ename, callback):
            eid = room._next_event_id
            room._next_event_id += 1
            if ename not in room._event_handlers:
                room._event_handlers[ename] = []
            room._event_handlers[ename].append({"id": eid, "callback": callback})
            return eid

        def unregister_event(eid):
            for handlers in room._event_handlers.values():
                for i, h in enumerate(handlers):
                    if h["id"] == eid:
                        handlers.pop(i)
                        return

        self._lua.globals()["RegisterEvent"] = register_event
        self._lua.globals()["UnregisterEvent"] = unregister_event

    def _dispatch_event(self, ename: str, evalue: str, player_index: int):
        handlers = self._event_handlers.get(ename, [])
        for h in list(handlers):
            try:
                h["callback"](h["id"], ename, evalue, player_index)
            except Exception as e:
                self._emit_log("ERR", "Event", f"Handler error for '{ename}': {e}\n{traceback.format_exc()}")

    def send_event(self, ename: str, evalue: str, player_index: int):
        """外部调用（Web UI / Bridge API）：向房间发送事件"""
        if len(ename.encode('utf-8')) > MAX_EVENT_NAME_LEN:
            self._emit_log("ERR", "System", f"Event name too long: {ename}")
            return
        if not ename.startswith('_') and not EVENT_NAME_PATTERN.match(ename):
            self._emit_log("ERR", "System", f"Invalid event name: {ename}")
            return
        if len(evalue.encode('utf-8')) > MAX_EVENT_DATA_LEN:
            self._emit_log("ERR", "System", f"Event data too long: {len(evalue.encode('utf-8'))} bytes")
            return
        self._event_queue.put(("_dispatch_event", ename, evalue, player_index))

    # ---- Player API 注入 ----

    def _inject_player_api(self):
        room = self

        def ms_get_player_name(idx):
            p = room.get_player(int(idx))
            return p.name if p else ""

        def ms_get_player_map_level(idx):
            p = room.get_player(int(idx))
            return p.map_level if p else 0

        def ms_get_player_map_exp(idx):
            p = room.get_player(int(idx))
            return p.map_exp if p else 0

        def ms_get_played_time(idx):
            p = room.get_player(int(idx))
            return p.get_played_time() if p else 0

        def ms_get_test_play_time(idx):
            p = room.get_player(int(idx))
            return p.test_play_time if p else 0

        def ms_get_played_count(idx):
            p = room.get_player(int(idx))
            return p.played_count if p else 0

        g = self._lua.globals()
        g["MsGetPlayerName"] = ms_get_player_name
        g["MsGetPlayerMapLevel"] = ms_get_player_map_level
        g["MsGetPlayerMapExp"] = ms_get_player_map_exp
        g["MsGetPlayedTime"] = ms_get_played_time
        g["MsGetTestPlayTime"] = ms_get_test_play_time
        g["MsGetPlayedCount"] = ms_get_played_count

    # ---- Room API 注入 ----

    def _inject_room_api(self):
        room = self

        def ms_get_room_start_ts():
            return room.start_ts

        def ms_get_room_loaded_ts():
            return room.loaded_ts

        def ms_get_room_game_time():
            if room.loaded_ts == 0:
                return 0
            return int(time.time()) - room.loaded_ts

        def ms_get_room_player_count():
            return sum(1 for p in room.players.values() if p.is_connected)

        def ms_get_room_mode_id():
            return room.mode_id

        g = self._lua.globals()
        g["MsGetRoomStartTs"] = ms_get_room_start_ts
        g["MsGetRoomLoadedTs"] = ms_get_room_loaded_ts
        g["MsGetRoomGameTime"] = ms_get_room_game_time
        g["MsGetRoomPlayerCount"] = ms_get_room_player_count
        g["MsGetRoomModeId"] = ms_get_room_mode_id

    # ---- Item API 注入 ----

    def _inject_item_api(self):
        room = self

        def ms_get_player_item(idx, key):
            p = room.get_player(int(idx))
            if not p:
                return 0
            return p.items.get(str(key), 0)

        def ms_consume_item(idx, iteminfo_json):
            p = room.get_player(int(idx))
            if not p:
                return 0

            room._trans_id_counter += 1
            trans_id = room._trans_id_counter

            try:
                items_to_consume = json.loads(str(iteminfo_json))
            except json.JSONDecodeError:
                # 异步返回错误
                room._async_consume_result(int(idx), trans_id, ERR_UNKNOWN, str(iteminfo_json))
                return trans_id

            # 检查数量是否足够
            for key, count in items_to_consume.items():
                if p.items.get(key, 0) < int(count):
                    room._async_consume_result(int(idx), trans_id, ERR_ITEM_NOT_ENOUGH, str(iteminfo_json))
                    return trans_id

            # 扣减
            for key, count in items_to_consume.items():
                p.items[key] = p.items.get(key, 0) - int(count)

            room._async_consume_result(int(idx), trans_id, ERR_OK, str(iteminfo_json))
            return trans_id

        self._lua.globals()["MsGetPlayerItem"] = ms_get_player_item
        self._lua.globals()["MsConsumeItem"] = ms_consume_item

    def _async_consume_result(self, player_index: int, trans_id: int, errnu: int, iteminfo_json: str):
        result = json.dumps({"trans_id": trans_id, "errnu": errnu, "iteminfo": json.loads(iteminfo_json)})

        def fire():
            self._event_queue.put(("_dispatch_event", "_citemret", result, player_index))

        t = threading.Timer(0.01, fire)
        t.daemon = True
        t.start()

    # ---- Archive API 注入 ----

    def _inject_archive_api(self):
        room = self

        def ms_get_script_archive(idx):
            p = room.get_player(int(idx))
            if not p or p.script_archive is None:
                return None
            return p.script_archive

        def ms_save_script_archive(idx, data, *args):
            p = room.get_player(int(idx))
            if not p:
                return ERR_PLAYER_NOT_EXIST
            s = str(data)
            if len(s.encode('utf-8')) > MAX_SCRIPT_ARCHIVE_LEN:
                return ERR_SCRIPT_ARCHIVE_TOO_LONG
            p.script_archive = s
            return ERR_OK

        def ms_get_common_archive(idx, key):
            p = room.get_player(int(idx))
            if not p:
                return None
            v = p.common_archive.get(str(key))
            return v if v else None

        def ms_get_read_archive(idx, key):
            p = room.get_player(int(idx))
            if not p:
                return None
            v = p.read_archive.get(str(key))
            return v if v else None

        def ms_set_read_archive(idx, key, value):
            p = room.get_player(int(idx))
            if not p:
                return ERR_PLAYER_NOT_EXIST
            k, v = str(key), str(value)
            p.read_archive[k] = v
            # 触发 _rdata 事件
            rdata = f"{k}\t{v}"
            room._emit_out_event(int(idx), "_rdata", rdata)
            return ERR_OK

        def ms_get_cfg_archive(idx, key):
            p = room.get_player(int(idx))
            if not p:
                return None
            v = p.cfg_archive.get(str(key))
            return v if v else None

        g = self._lua.globals()
        g["MsGetScriptArchive"] = ms_get_script_archive
        g["MsSaveScriptArchive"] = ms_save_script_archive
        g["MsGetCommonArchive"] = ms_get_common_archive
        g["MsGetReadArchive"] = ms_get_read_archive
        g["MsSetReadArchive"] = ms_set_read_archive
        g["MsGetCfgArchive"] = ms_get_cfg_archive

    # ---- Control API 注入 ----

    def _inject_control_api(self):
        room = self

        def ms_send_ml_event(idx, ename, evalue):
            ename_s = str(ename)
            evalue_s = str(evalue) if evalue is not None else ""
            if len(ename_s.encode('utf-8')) > MAX_EVENT_NAME_LEN:
                return ERR_EVENT_KEY_LEN
            if len(evalue_s.encode('utf-8')) > MAX_EVENT_DATA_LEN:
                return ERR_EVENT_VALUE_LEN
            room._emit_out_event(int(idx), ename_s, evalue_s)
            return ERR_OK

        def ms_end(idx, reason):
            room._emit_log("INF", "System", f"MsEnd called: player={idx} reason={reason}")
            room._running = False
            return ERR_OK

        g = self._lua.globals()
        g["MsSendMlEvent"] = ms_send_ml_event
        g["MsEnd"] = ms_end

    def _emit_out_event(self, player_index: int, ename: str, evalue: str):
        ev = OutEvent(player_index, ename, evalue, self.id)
        if player_index >= 0:
            if player_index not in self._out_queues:
                self._out_queues[player_index] = deque(maxlen=500)
            self._out_queues[player_index].append(ev)
        else:
            for idx in self.players:
                if idx not in self._out_queues:
                    self._out_queues[idx] = deque(maxlen=500)
                self._out_queues[idx].append(ev)
        for cb in self.on_event:
            try:
                cb(ev)
            except Exception:
                pass

    def poll_events(self, player_index: int) -> list[dict]:
        q = self._out_queues.get(player_index)
        if not q:
            return []
        events = [e.to_dict() for e in q]
        q.clear()
        return events

    # ---- 内置事件触发 ----

    def _fire_room_loaded(self):
        player_indices = list(self.players.keys())
        data = json.dumps({"players": player_indices})
        self._dispatch_event("_roomloaded", data, -1)

    def _fire_room_over(self, reason: str = "GameEnd"):
        data = json.dumps({"reason": reason})
        self._dispatch_event("_roomover", data, -1)

    def fire_player_exit(self, player_index: int, reason: str = "Logout"):
        data = json.dumps({"reason": reason})
        self._event_queue.put(("_dispatch_event", "_playerexit", data, player_index))

    def fire_player_leave(self, player_index: int, reason: str = "Disconnect"):
        p = self.get_player(player_index)
        if p:
            p.is_connected = False
        data = json.dumps({"reason": reason})
        self._event_queue.put(("_dispatch_event", "_playerleave", data, player_index))

    def fire_player_join(self, player_index: int, reason: str = "Connect"):
        p = self.get_player(player_index)
        if p:
            p.is_connected = True
        data = json.dumps({"reason": reason})
        self._event_queue.put(("_dispatch_event", "_playerjoin", data, player_index))

    # ---- 状态查询 ----

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "script_dir": self.script_dir,
            "mode_id": self.mode_id,
            "status": self.status,
            "error_message": self.error_message,
            "start_ts": self.start_ts,
            "loaded_ts": self.loaded_ts,
            "game_time": int(time.time()) - self.loaded_ts if self.loaded_ts else 0,
            "player_count": sum(1 for p in self.players.values() if p.is_connected),
            "players": {idx: p.to_dict() for idx, p in self.players.items()},
        }


class RoomManager:
    def __init__(self):
        self.rooms: dict[str, Room] = {}
        self._lock = threading.Lock()

    def create_room(self, script_dir: str, mode_id: int = 0) -> Room:
        room = Room(script_dir, mode_id)
        with self._lock:
            self.rooms[room.id] = room
        return room

    def get_room(self, room_id: str) -> Room | None:
        return self.rooms.get(room_id)

    def list_rooms(self) -> list[dict]:
        return [r.to_dict() for r in self.rooms.values()]

    def destroy_room(self, room_id: str) -> bool:
        room = self.rooms.get(room_id)
        if not room:
            return False
        room.destroy()
        with self._lock:
            self.rooms.pop(room_id, None)
        return True


# ---- 内嵌 JSON Lua 库 ----

def _get_json_lua() -> str:
    """纯 Lua 的 JSON encode/decode 实现"""
    return r'''
-- Minimal JSON library for MLS simulator
json = {}

local function escape_str(s)
    local in_char  = {'\\', '"', '\n', '\r', '\t', '/', '\b', '\f'}
    local out_char = {'\\', '"', 'n',  'r',  't',  '/', 'b',  'f'}
    for i, c in ipairs(in_char) do
        s = s:gsub(c, '\\' .. out_char[i])
    end
    return s
end

local function is_array(t)
    local i = 0
    for _ in pairs(t) do
        i = i + 1
        if t[i] == nil then return false end
    end
    return true
end

local function encode_value(val)
    local vtype = type(val)
    if val == nil then
        return "null"
    elseif vtype == "boolean" then
        return val and "true" or "false"
    elseif vtype == "number" then
        if val ~= val then return "null" end
        if val >= math.huge then return "1e9999" end
        if val <= -math.huge then return "-1e9999" end
        if val == math.floor(val) and math.abs(val) < 1e15 then
            return string.format("%d", val)
        end
        return string.format("%.14g", val)
    elseif vtype == "string" then
        return '"' .. escape_str(val) .. '"'
    elseif vtype == "table" then
        if is_array(val) then
            local parts = {}
            for i = 1, #val do
                parts[i] = encode_value(val[i])
            end
            return "[" .. table.concat(parts, ",") .. "]"
        else
            local parts = {}
            for k, v in pairs(val) do
                local key = type(k) == "number" and string.format("%d", k) or tostring(k)
                parts[#parts + 1] = '"' .. escape_str(key) .. '":' .. encode_value(v)
            end
            return "{" .. table.concat(parts, ",") .. "}"
        end
    else
        return "null"
    end
end

function json.encode(val)
    return encode_value(val)
end

-- JSON decode
local function skip_ws(s, pos)
    while pos <= #s do
        local c = s:byte(pos)
        if c == 32 or c == 9 or c == 10 or c == 13 then
            pos = pos + 1
        else
            break
        end
    end
    return pos
end

local decode_value

local function decode_string(s, pos)
    pos = pos + 1 -- skip opening "
    local parts = {}
    while pos <= #s do
        local c = s:sub(pos, pos)
        if c == '"' then
            return table.concat(parts), pos + 1
        elseif c == '\\' then
            pos = pos + 1
            local esc = s:sub(pos, pos)
            if esc == 'n' then parts[#parts+1] = '\n'
            elseif esc == 'r' then parts[#parts+1] = '\r'
            elseif esc == 't' then parts[#parts+1] = '\t'
            elseif esc == '"' then parts[#parts+1] = '"'
            elseif esc == '\\' then parts[#parts+1] = '\\'
            elseif esc == '/' then parts[#parts+1] = '/'
            elseif esc == 'b' then parts[#parts+1] = '\b'
            elseif esc == 'f' then parts[#parts+1] = '\f'
            elseif esc == 'u' then
                local hex = s:sub(pos+1, pos+4)
                local code = tonumber(hex, 16)
                if code then
                    if code < 128 then
                        parts[#parts+1] = string.char(code)
                    elseif code < 2048 then
                        parts[#parts+1] = string.char(192 + math.floor(code/64), 128 + code%64)
                    else
                        parts[#parts+1] = string.char(224 + math.floor(code/4096), 128 + math.floor(code%4096/64), 128 + code%64)
                    end
                end
                pos = pos + 4
            else
                parts[#parts+1] = esc
            end
            pos = pos + 1
        else
            parts[#parts+1] = c
            pos = pos + 1
        end
    end
    error("unterminated string")
end

local function decode_number(s, pos)
    local start = pos
    if s:sub(pos,pos) == '-' then pos = pos + 1 end
    while pos <= #s and s:byte(pos) >= 48 and s:byte(pos) <= 57 do pos = pos + 1 end
    if pos <= #s and s:sub(pos,pos) == '.' then
        pos = pos + 1
        while pos <= #s and s:byte(pos) >= 48 and s:byte(pos) <= 57 do pos = pos + 1 end
    end
    if pos <= #s and (s:sub(pos,pos) == 'e' or s:sub(pos,pos) == 'E') then
        pos = pos + 1
        if pos <= #s and (s:sub(pos,pos) == '+' or s:sub(pos,pos) == '-') then pos = pos + 1 end
        while pos <= #s and s:byte(pos) >= 48 and s:byte(pos) <= 57 do pos = pos + 1 end
    end
    local num = tonumber(s:sub(start, pos-1))
    return num, pos
end

local function decode_object(s, pos)
    pos = pos + 1 -- skip {
    local obj = {}
    pos = skip_ws(s, pos)
    if s:sub(pos,pos) == '}' then return obj, pos + 1 end
    while true do
        pos = skip_ws(s, pos)
        if s:sub(pos,pos) ~= '"' then error("expected string key at pos " .. pos) end
        local key
        key, pos = decode_string(s, pos)
        pos = skip_ws(s, pos)
        if s:sub(pos,pos) ~= ':' then error("expected ':' at pos " .. pos) end
        pos = skip_ws(s, pos + 1)
        local val
        val, pos = decode_value(s, pos)
        obj[key] = val
        pos = skip_ws(s, pos)
        local c = s:sub(pos,pos)
        if c == '}' then return obj, pos + 1 end
        if c ~= ',' then error("expected ',' or '}' at pos " .. pos) end
        pos = pos + 1
    end
end

local function decode_array(s, pos)
    pos = pos + 1 -- skip [
    local arr = {}
    pos = skip_ws(s, pos)
    if s:sub(pos,pos) == ']' then return arr, pos + 1 end
    while true do
        pos = skip_ws(s, pos)
        local val
        val, pos = decode_value(s, pos)
        arr[#arr+1] = val
        pos = skip_ws(s, pos)
        local c = s:sub(pos,pos)
        if c == ']' then return arr, pos + 1 end
        if c ~= ',' then error("expected ',' or ']' at pos " .. pos) end
        pos = pos + 1
    end
end

decode_value = function(s, pos)
    pos = skip_ws(s, pos)
    local c = s:sub(pos, pos)
    if c == '"' then return decode_string(s, pos)
    elseif c == '{' then return decode_object(s, pos)
    elseif c == '[' then return decode_array(s, pos)
    elseif c == 't' then
        if s:sub(pos, pos+3) == 'true' then return true, pos+4 end
    elseif c == 'f' then
        if s:sub(pos, pos+4) == 'false' then return false, pos+5 end
    elseif c == 'n' then
        if s:sub(pos, pos+3) == 'null' then return nil, pos+4 end
    elseif c == '-' or (c >= '0' and c <= '9') then
        return decode_number(s, pos)
    end
    error("unexpected character '" .. c .. "' at pos " .. pos)
end

function json.decode(s)
    if s == nil or s == "" then return nil end
    local val, _ = decode_value(s, 1)
    return val
end
'''
