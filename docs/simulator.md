# 云脚本模拟器

`mls-sim-rs` 是 Rust 实现的本地 MLS 云脚本运行时，使用 axum (HTTP) + mlua (Lua 5.4) + rust-embed (前端嵌入)。

## 核心能力

- 多房间同时运行，每个房间拥有独立 Lua VM 和独立 OS 线程。
- 加载指定脚本目录下的 `main.lua`。
- 注入完整 MLS 全局 API：日志、定时器、事件、房间查询、玩家查询、道具、存档。
- 自定义 `require()` 支持相对路径和 `.` 分隔符。
- 内嵌纯 Lua JSON 库（`json.encode` / `json.decode`）。
- Web 仪表盘管理房间、玩家、事件和日志（编译时嵌入二进制）。
- REST API 供外部工具调用。
- Bridge API 供 War3 客户端轮询和发送事件。
- WebSocket 实时推送日志和出站事件。
- 存档持久化到本地 JSON 文件。

## 架构

```
┌─────────────────────────────────────┐
│  Tokio 异步运行时                    │
│  (HTTP 服务器、WebSocket、定时器)     │
│                                     │
│  axum 处理器通过 mpsc channel        │
│  向房间线程发送命令                   │
└──────────┬──────────────────────────┘
           │ mpsc::UnboundedSender
    ┌──────▼──────┐
    │  Room 线程   │  std::thread (长生命周期)
    │  Lua VM     │  独占 lua_State
    │  事件循环    │  50ms 超时轮询命令队列
    └──┬──────────┘
       │ broadcast::Sender
       ├─→ WebSocket 日志订阅
       └─→ WebSocket 事件订阅
```

每个房间独占一个 OS 线程，Lua VM 的全部调用都在该线程上执行，保证线程安全。

## 启动参数

```powershell
mls-sim.exe [--host <host>] [--port <port>] [--script-dir <path>] [--config <path>]
```

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--host` | `127.0.0.1` | 监听地址 |
| `--port` | `5000` | 监听端口 |
| `--script-dir` | - | 云脚本目录，启动后自动创建房间 |
| `--config` | `config.json` | 配置文件路径 |

## 配置文件

`config.json` 格式：

```json
{
  "host": "127.0.0.1",
  "port": 5000,
  "auto_open_browser": true,
  "archive_dir": "./archives",
  "auto_room": {
    "script_dir": "D:/path/to/script",
    "mode_id": 0,
    "players": [
      {
        "index": 0,
        "name": "Player_0",
        "items": {"VIP001": 1},
        "map_level": 1,
        "script_archive": null,
        "read_archive": {"boss_kill": "0"}
      }
    ]
  }
}
```

可通过 Web 仪表盘的 Settings API (`GET/PUT /api/settings`) 动态修改。

## WebSocket

使用原生 WebSocket 协议（非 Socket.IO），连接地址：

```
ws://127.0.0.1:5000/ws
```

消息格式：

```json
// 客户端 → 服务器
{"type": "join_room", "room_id": "room-001"}
{"type": "leave_room", "room_id": "room-001"}

// 服务器 → 客户端
{"type": "log", "data": {"timestamp": ..., "level": "INF", "source": "Lua", "message": "...", ...}}
{"type": "out_event", "data": {"player_index": 0, "ename": "...", "evalue": "...", ...}}
```

连接后发送 `join_room` 消息，服务器会先推送该房间的缓存日志和事件，然后实时推送新的日志和事件。

## 存档

房间停止时自动保存玩家存档到 `archives/<script_dir_name>.json`。下次创建房间时可通过配置文件预设存档数据。

详细接口见 [REST API](rest-api.md)。
