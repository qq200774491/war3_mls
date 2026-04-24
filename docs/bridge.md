# Bridge 客户端集成

Bridge 用于让 War3 客户端或测试脚本在本地连接 MLS 模拟器，替代平台云脚本通道。

## 配置

Bridge 配置可通过 `POST /api/bridge/config` 生成，格式如下：

```lua
return {
    base_url = "http://127.0.0.1:5000",
    room_id = "room-001",
    player_index = 0,
    poll_interval = 0.05,
    req_sign_enable = false,
}
```

## 通信流程

1. 客户端通过 `/api/bridge/login` 登录房间。
2. 客户端通过 `/api/bridge/event` 发送事件给云脚本。
3. 客户端通过 `/api/bridge/poll/{room_id}/{player_index}` 轮询云脚本发出的事件。
4. 模拟器房间内的 Lua 脚本通过 `MsSendMlEvent` 将事件写入玩家队列。

## 用 Python 模拟客户端

```python
import requests, time

BASE = "http://127.0.0.1:5000"
ROOM = "room-001"
PLAYER = 0

# 登录
requests.post(f"{BASE}/api/bridge/login", json={
    "room_id": ROOM, "player_index": PLAYER, "name": "TestBot"
})

# 发送事件
requests.post(f"{BASE}/api/bridge/event", json={
    "room_id": ROOM, "player_index": PLAYER,
    "ename": "buy_tower", "evalue": '{"id":1}'
})

# 轮询
while True:
    r = requests.get(f"{BASE}/api/bridge/poll/{ROOM}/{PLAYER}")
    events = r.json().get("events", [])
    for ev in events:
        print(f"[{ev['ename']}] {ev['evalue']}")
    time.sleep(0.05)
```

## 用 curl 模拟

```powershell
# 发送事件
curl -X POST http://127.0.0.1:5000/api/bridge/event `
  -H "Content-Type: application/json" `
  -d '{"room_id":"room-001","player_index":0,"ename":"testapi","evalue":""}'

# 轮询
curl http://127.0.0.1:5000/api/bridge/poll/room-001/0
```

## 注意

Bridge 是本地测试适配层，不应直接进入正式平台发布包。
