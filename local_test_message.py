#!/usr/bin/env python3
"""
本地向 mls-sim 发送一条客户端消息，用来测试类似：

client "测试消息" (
    function(p, label, data)
        print(string.format("玩家[%s]发送了测试消息", p:name()))
    end
)

使用前先启动模拟器，例如：
    cd mls-sim-rs
    cargo run -- -s "D:/Code/MlScriptFolder/script_compile"

再运行：
    python local_test_message.py

注意：编译后的 client "测试消息" 不是直接监听底层事件名“测试消息”，
而是 client 模块统一监听底层事件 `war3_data`，再从 JSON payload
里的 `label` 分发到“测试消息”。

另外客户端进房后会先发 `player_info` 注册玩家；没有注册时，云脚本里
player(pid) 查不到，会报“未知玩家 war3_data PID:0”。
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from typing import Any


MESSAGE_ID = int(time.time() * 1000)

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")


def request_json(method: str, url: str, payload: dict[str, Any] | None = None) -> Any:
    data = None
    headers = {"Accept": "application/json"}
    if payload is not None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        headers["Content-Type"] = "application/json; charset=utf-8"

    request = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"{method} {url} -> HTTP {error.code}: {body}") from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"{method} {url} 失败：{error.reason}") from error

    return json.loads(body) if body else None


def pick_room(base_url: str, room_id: str | None) -> str:
    if room_id:
        return room_id

    rooms = request_json("GET", f"{base_url}/api/bridge/rooms")
    if not rooms:
        raise RuntimeError("当前没有房间，请先用 -s 指定脚本目录启动模拟器，或在 Web 面板创建房间")

    running_rooms = [room for room in rooms if room.get("status") == "running"]
    room = running_rooms[0] if running_rooms else rooms[0]
    return str(room["id"])


def main() -> int:
    parser = argparse.ArgumentParser(description="向本地 mls-sim 发送一条 client 测试消息")
    parser.add_argument("--base-url", default="http://127.0.0.1:5000", help="mls-sim 地址")
    parser.add_argument("--room-id", help="房间 ID；不填则自动选择第一个房间")
    parser.add_argument("--player-index", type=int, default=0, help="玩家槽位")
    parser.add_argument("--player-name", default="本地测试玩家", help="登录到房间时使用的玩家名")
    parser.add_argument("--platform-id", default="10001", help="player_info 里的平台 UID，不能是 0")
    parser.add_argument("--login-wait", type=float, default=0.1, help="login 后等待房间处理玩家加入的秒数")
    parser.add_argument("--skip-player-info", action="store_true", help="跳过 player_info 注册")
    parser.add_argument("--label", default="测试消息", help="client 后面的消息名")
    parser.add_argument("--data", default="", help="回调 data；默认按字符串发送")
    parser.add_argument("--json-data", action="store_true", help="把 --data 当 JSON 解析后发送")
    parser.add_argument("--poll", action="store_true", help="发送后顺便轮询脚本发给该玩家的事件")
    parser.add_argument("--poll-only", action="store_true", help="只轮询出站事件，不发送 client 消息")
    parser.add_argument("--poll-times", type=int, default=1, help="轮询次数")
    parser.add_argument("--poll-interval", type=float, default=0.1, help="轮询间隔秒数")
    args = parser.parse_args()

    base_url = args.base_url.rstrip("/")

    try:
        health = request_json("GET", f"{base_url}/api/health")
        print(f"模拟器在线：{health.get('name')} v{health.get('version')}，房间数={health.get('room_count')}")

        room_id = pick_room(base_url, args.room_id)
        print(f"使用房间：{room_id}")

        login_payload = {
            "room_id": room_id,
            "player_index": args.player_index,
            "name": args.player_name,
        }
        login_result = request_json("POST", f"{base_url}/api/bridge/login", login_payload)
        if not login_result.get("ok"):
            raise RuntimeError(f"登录失败：{login_result}")
        print(f"玩家登录成功：index={args.player_index}, name={args.player_name}")
        time.sleep(args.login_wait)

        if not args.skip_player_info:
            player_info_payload = {
                "room_id": room_id,
                "player_index": args.player_index,
                "ename": "player_info",
                "evalue": json.dumps({"player_uuids": [args.platform_id]}, ensure_ascii=False),
            }
            player_info_result = request_json("POST", f"{base_url}/api/bridge/event", player_info_payload)
            if not player_info_result.get("ok"):
                raise RuntimeError(f"注册玩家失败：{player_info_result}")
            print(f"已发送 player_info：player_uuids=[{args.platform_id!r}]")
            time.sleep(0.1)

        if not args.poll_only:
            if args.json_data:
                callback_data: Any = json.loads(args.data)
            else:
                callback_data = args.data

            war3_payload = {
                "id": MESSAGE_ID,
                "label": args.label,
                "pid": args.player_index + 1,
                "data": {"__v": callback_data, "__i": MESSAGE_ID},
            }
            event_payload = {
                "room_id": room_id,
                "player_index": args.player_index,
                "ename": "war3_data",
                "evalue": json.dumps(war3_payload, ensure_ascii=False),
            }
            event_result = request_json("POST", f"{base_url}/api/bridge/event", event_payload)
            if not event_result.get("ok"):
                raise RuntimeError(f"发送事件失败：{event_result}")
            print(f"已发送 client 消息：label={args.label!r}，data={callback_data!r}")

        if args.poll or args.poll_only:
            for poll_index in range(max(1, args.poll_times)):
                if poll_index > 0 or not args.poll_only:
                    time.sleep(args.poll_interval)
                polled = request_json("GET", f"{base_url}/api/bridge/poll/{room_id}/{args.player_index}")
                events = polled.get("events", [])
                print(f"轮询结果 #{poll_index + 1}：{len(events)} 条")
                print(json.dumps(polled, ensure_ascii=False, indent=2))

        print("请在 mls-sim 日志面板查看 Lua print 输出。")
        return 0
    except Exception as error:
        print(f"错误：{error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
