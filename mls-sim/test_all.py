"""MLS-Sim 全功能测试 — 模拟客户端通过 Bridge API 通讯"""

import requests
import time
import json
import sys
import subprocess
import atexit

BASE = "http://localhost:5099"
SCRIPT_DIR = r"D:\code2\mls\参考\mls-master\demo\apidemo\script"
TOWNER_DIR = r"D:\code2\mls\参考\mls-master\demo\towner\script"

passed = 0
failed = 0


def check(name, condition, detail=""):
    global passed, failed
    if condition:
        passed += 1
        print(f"  [PASS] {name}")
    else:
        failed += 1
        print(f"  [FAIL] {name} — {detail}")


def section(title):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")


# ============================================================
#  1. 房间管理
# ============================================================
section("1. 房间管理")

# 创建 apidemo 房间（2 人）
r = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": SCRIPT_DIR,
    "mode_id": 1,
    "players": [
        {"index": 0, "name": "Alice", "items": {"VIP001": 1, "GOLD": 100}},
        {"index": 1, "name": "Bob", "items": {"VIP001": 0}},
    ],
    "auto_start": True,
})
check("创建房间 (apidemo)", r.status_code == 201, f"status={r.status_code}")
room1 = r.json()
room1_id = room1.get("id", "")
check("房间 ID 格式", room1_id.startswith("room-"), f"id={room1_id}")
check("房间状态=created/running", room1.get("status") in ("created", "running"), f"status={room1.get('status')}")

time.sleep(2)  # 等待脚本加载完成

# 创建 towner 房间（1 人）
r = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": TOWNER_DIR,
    "mode_id": 2,
    "players": [
        {"index": 0, "name": "Charlie", "items": {"VIP001": 1}},
    ],
})
check("创建第二个房间 (towner)", r.status_code == 201)
room2 = r.json()
room2_id = room2.get("id", "")

time.sleep(2)

# 列出房间
r = requests.get(f"{BASE}/api/rooms")
rooms = r.json()
check("列出房间", len(rooms) >= 2, f"count={len(rooms)}")

# 获取单个房间
r = requests.get(f"{BASE}/api/rooms/{room1_id}")
check("获取房间详情", r.status_code == 200 and r.json().get("status") == "running")

# 获取不存在的房间
r = requests.get(f"{BASE}/api/rooms/room-999")
check("获取不存在房间返回 404", r.status_code == 404)

# ============================================================
#  2. 玩家管理
# ============================================================
section("2. 玩家管理")

# 获取房间状态检查玩家
r = requests.get(f"{BASE}/api/rooms/{room1_id}/state")
state = r.json()
players = state.get("players", {})
check("房间有 2 个玩家", len(players) == 2, f"count={len(players)}")
check("玩家 0 名字=Alice", players.get("0", {}).get("name") == "Alice")
check("玩家 1 名字=Bob", players.get("1", {}).get("name") == "Bob")
check("玩家 0 道具 VIP001=1", players.get("0", {}).get("items", {}).get("VIP001") == 1)
check("玩家 0 道具 GOLD=100", players.get("0", {}).get("items", {}).get("GOLD") == 100)

# 更新玩家属性
r = requests.put(f"{BASE}/api/rooms/{room1_id}/players/0", json={
    "map_level": 10,
    "map_exp": 5000,
})
check("更新玩家属性", r.status_code == 200)
p0 = r.json()
check("更新后 map_level=10", p0.get("map_level") == 10)
check("更新后 map_exp=5000", p0.get("map_exp") == 5000)

# 添加新玩家
r = requests.post(f"{BASE}/api/rooms/{room1_id}/players", json={
    "index": 2, "name": "Charlie_New", "items": {"SWORD": 1}
})
check("添加第 3 个玩家", r.status_code == 201)

# 移除玩家
r = requests.delete(f"{BASE}/api/rooms/{room1_id}/players/2")
check("移除第 3 个玩家", r.status_code == 200)

# ============================================================
#  3. 玩家生命周期事件
# ============================================================
section("3. 玩家生命周期事件")

# 玩家断线
r = requests.post(f"{BASE}/api/rooms/{room1_id}/players/1/leave", json={"reason": "Disconnect"})
check("玩家 1 断线 (leave)", r.status_code == 200)
time.sleep(0.5)

# 确认玩家离线
r = requests.get(f"{BASE}/api/rooms/{room1_id}/state")
p1 = r.json().get("players", {}).get("1", {})
check("玩家 1 离线", p1.get("is_connected") == False, f"is_connected={p1.get('is_connected')}")

# 玩家重连
r = requests.post(f"{BASE}/api/rooms/{room1_id}/players/1/join", json={"reason": "Reconnect"})
check("玩家 1 重连 (join)", r.status_code == 200)
time.sleep(0.5)

r = requests.get(f"{BASE}/api/rooms/{room1_id}/state")
p1 = r.json().get("players", {}).get("1", {})
check("玩家 1 恢复在线", p1.get("is_connected") == True)

# 玩家退出
r = requests.post(f"{BASE}/api/rooms/{room1_id}/players/1/exit", json={"reason": "Logout"})
check("玩家 1 退出 (exit)", r.status_code == 200)

# ============================================================
#  4. 事件发送（Web UI 通道）
# ============================================================
section("4. 事件发送 (Web UI)")

# 发送自定义事件
r = requests.post(f"{BASE}/api/rooms/{room1_id}/events", json={
    "ename": "testapi",
    "evalue": "",
    "player_index": 0,
})
check("发送事件 testapi", r.status_code == 200)

# 发送带数据的事件
r = requests.post(f"{BASE}/api/rooms/{room1_id}/events", json={
    "ename": "pong",
    "evalue": '{"msg":"hello"}',
    "player_index": 0,
})
check("发送事件 pong 带数据", r.status_code == 200)

# 发送房间事件 (player_index=-1)
r = requests.post(f"{BASE}/api/rooms/{room1_id}/events", json={
    "ename": "msdata",
    "evalue": "",
    "player_index": -1,
})
check("发送房间事件 (player=-1)", r.status_code == 200)

# 无事件名
r = requests.post(f"{BASE}/api/rooms/{room1_id}/events", json={
    "ename": "",
    "evalue": "",
    "player_index": 0,
})
check("空事件名返回 400", r.status_code == 400)

# ============================================================
#  5. Bridge API（模拟客户端通讯）
# ============================================================
section("5. Bridge API (模拟客户端)")

# 查询可用房间
r = requests.get(f"{BASE}/api/bridge/rooms")
bridge_rooms = r.json()
check("Bridge 房间列表", len(bridge_rooms) >= 2)
check("Bridge 房间包含 mode_id", all("mode_id" in rm for rm in bridge_rooms))

# 登录到房间
r = requests.post(f"{BASE}/api/bridge/login", json={
    "room_id": room1_id,
    "player_index": 0,
    "name": "Alice",
})
check("Bridge 登录", r.status_code == 200 and r.json().get("ok"))

# 清空之前的事件队列
requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
time.sleep(0.3)

# 发送事件
r = requests.post(f"{BASE}/api/bridge/event", json={
    "room_id": room1_id,
    "player_index": 0,
    "ename": "testapi",
    "evalue": "",
})
check("Bridge 发送事件", r.status_code == 200 and r.json().get("ok"))
time.sleep(1)

# 轮询结果
r = requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
events = r.json().get("events", [])
check("Bridge 轮询收到事件", len(events) > 0, f"收到 {len(events)} 个事件")
if events:
    event_names = [e["ename"] for e in events]
    check("Bridge 事件包含 mslog", "mslog" in event_names, f"events={event_names}")

# 再次轮询应为空或极少（Lua 定时器可能异步产生少量事件）
r = requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
events2 = r.json().get("events", [])
check("Bridge 二次轮询为空或极少", len(events2) <= 3, f"残留 {len(events2)} 个事件")

# 登录不存在的房间
r = requests.post(f"{BASE}/api/bridge/login", json={
    "room_id": "room-999",
    "player_index": 0,
})
check("Bridge 登录不存在房间返回 404", r.status_code == 404)

# 发送事件到不存在的房间
r = requests.post(f"{BASE}/api/bridge/event", json={
    "room_id": "room-999",
    "player_index": 0,
    "ename": "test",
    "evalue": "",
})
check("Bridge 发送到不存在房间返回 404", r.status_code == 404)

# ============================================================
#  6. 多房间独立性
# ============================================================
section("6. 多房间独立性")

# 先清空两个房间的事件队列
requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
requests.get(f"{BASE}/api/bridge/poll/{room2_id}/0")
time.sleep(0.5)
# 再清一次（Lua 异步定时器可能在间隙产生事件）
requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
requests.get(f"{BASE}/api/bridge/poll/{room2_id}/0")
time.sleep(0.3)

# 向 towner 发送 buy_tower 事件
r = requests.post(f"{BASE}/api/bridge/event", json={
    "room_id": room2_id,
    "player_index": 0,
    "ename": "buy_tower",
    "evalue": "",
})
check("向 towner 发送 buy_tower", r.status_code == 200)
time.sleep(1)

# 轮询 towner 的结果
r = requests.get(f"{BASE}/api/bridge/poll/{room2_id}/0")
towner_events = r.json().get("events", [])
check("towner 收到回应事件", len(towner_events) > 0, f"收到 {len(towner_events)} 个事件")

# 确认 apidemo 没有收到 towner 的事件（允许少量自身定时器事件）
r = requests.get(f"{BASE}/api/bridge/poll/{room1_id}/0")
cross_events = r.json().get("events", [])
towner_enames = [e["ename"] for e in cross_events if "tower" in e.get("ename", "").lower()]
check("apidemo 未收到 towner 事件 (隔离)", len(towner_enames) == 0,
      f"收到 towner 事件: {towner_enames}")

# ============================================================
#  7. 存档操作
# ============================================================
section("7. 存档操作")

# 更新玩家存档
r = requests.put(f"{BASE}/api/rooms/{room1_id}/players/0", json={
    "script_archive": '{"gold":999,"level":50}',
    "common_archive": {"kills": "100"},
    "read_archive": {"rank": "S"},
    "cfg_archive": {"season": "4"},
})
check("设置玩家存档", r.status_code == 200)
p = r.json()
check("script_archive 已设置", p.get("script_archive") == '{"gold":999,"level":50}')
check("common_archive 已设置", p.get("common_archive", {}).get("kills") == "100")
check("read_archive 已设置", p.get("read_archive", {}).get("rank") == "S")
check("cfg_archive 已设置", p.get("cfg_archive", {}).get("season") == "4")

# ============================================================
#  8. 房间停止与存档持久化
# ============================================================
section("8. 房间停止与存档持久化")

# 停止 apidemo 房间
r = requests.post(f"{BASE}/api/rooms/{room1_id}/stop", json={"reason": "TestEnd"})
check("停止 apidemo 房间", r.status_code == 200)
time.sleep(1)

# 检查房间状态
r = requests.get(f"{BASE}/api/rooms/{room1_id}")
check("房间状态=stopped", r.json().get("status") == "stopped", f"status={r.json().get('status')}")

# 查看存档
r = requests.get(f"{BASE}/api/archives")
archives = r.json()
check("存档已保存", len(archives) > 0, f"count={len(archives)}")

# 停止 towner 房间
r = requests.post(f"{BASE}/api/rooms/{room2_id}/stop")
check("停止 towner 房间", r.status_code == 200)
time.sleep(1)

# 销毁房间
r = requests.delete(f"{BASE}/api/rooms/{room1_id}")
check("销毁 apidemo 房间", r.status_code == 200)
r = requests.delete(f"{BASE}/api/rooms/{room2_id}")
check("销毁 towner 房间", r.status_code == 200)

# 确认测试房间已清空
r = requests.get(f"{BASE}/api/rooms")
remaining = [rm for rm in r.json() if rm["id"] in (room1_id, room2_id)]
check("测试房间已清空", len(remaining) == 0, f"剩余 {len(remaining)} 个")

# ============================================================
#  9. 错误房间（脚本加载失败）
# ============================================================
section("9. 错误处理")

# 无效脚本目录
r = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": "C:/nonexistent/path",
})
check("无效脚本目录返回 400", r.status_code == 400)

# 空脚本目录
r = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": "",
})
check("空脚本目录返回 400", r.status_code == 400)


# ============================================================
#  Summary
# ============================================================
print(f"\n{'='*60}")
print(f"  测试完成: {passed} 通过, {failed} 失败, 共 {passed+failed}")
print(f"{'='*60}")

sys.exit(1 if failed > 0 else 0)
