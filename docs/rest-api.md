# REST API

默认地址：

```text
http://127.0.0.1:5000
```

## 健康检查

### `GET /api/health`

返回模拟器状态、版本、房间数量和房间列表。

## 房间

### `POST /api/rooms`

创建房间并可自动启动。

```json
{
  "script_dir": "D:/code2/mls/参考/mls-master/demo/apidemo/script",
  "mode_id": 0,
  "players": [
    {"index": 0, "name": "Player_0"},
    {"index": 1, "name": "Player_1"}
  ],
  "auto_start": true
}
```

### 常用端点

- `GET /api/rooms`
- `GET /api/rooms/{room_id}`
- `DELETE /api/rooms/{room_id}`
- `POST /api/rooms/{room_id}/start`
- `POST /api/rooms/{room_id}/stop`
- `POST /api/rooms/{room_id}/events`
- `GET /api/rooms/{room_id}/state`

## Bridge

- `POST /api/bridge/login`
- `POST /api/bridge/event`
- `GET /api/bridge/poll/{room_id}/{player_index}`
- `GET /api/bridge/rooms`
- `POST /api/bridge/config`

## Profiles

- `GET /api/profiles`
- `POST /api/profiles`

Profile 用于保存测试脚本目录、玩家配置和 War3 配置，方便插件后续复用。
