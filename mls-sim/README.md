# MLS Simulator

MLS 云脚本本地多开模拟测试环境。在本地同时运行多个房间实例，模拟完整的 MLS 运行时 API，通过 Web 面板管理房间、玩家和事件。

## 特性

- 完整模拟 MLS 运行时全部 API（日志、定时器、事件、存档、道具）
- 多房间同时运行，互不干扰
- 每个房间支持多个模拟玩家
- Web 控制面板：实时日志、事件发送、状态查看
- 使用 Lua 5.3（与 MLS 生产环境一致）

## 安装

需要 Python 3.10+。

```bash
cd mls-sim
pip install -r requirements.txt
```

依赖项：`lupa`（Lua 5.3 嵌入）、`flask`、`flask-socketio`。

## 快速开始

### 1. 启动服务

```bash
python app.py 5001
```

输出：

```
MLS Simulator running at http://localhost:5001
```

### 2. 打开控制面板

浏览器访问 `http://localhost:5001`。

### 3. 创建房间

点击左侧 **+ New** 按钮，填写：

- **Script Directory**：Lua 脚本所在目录的绝对路径（包含 `main.lua` 的目录）
- **Players**：玩家配置 JSON 数组

示例配置：

```json
[
  {"index": 0, "name": "Alice", "items": {"VIP001": 1}},
  {"index": 1, "name": "Bob"}
]
```

点击 **Create**，房间自动启动，日志面板开始输出脚本执行信息。

### 4. 发送事件

在 **Send Event** 面板输入事件名和数据，选择目标玩家，点击 **Send**。也可以点击底部的预设按钮快速填充。

---

## Web 控制面板

### 房间管理

| 操作 | 说明 |
|------|------|
| + New | 创建房间，指定脚本目录、玩家配置、模式 ID |
| Start | 启动已停止的房间 |
| Stop | 触发 `_roomover` 事件后停止房间 |
| Destroy | 销毁房间并释放资源 |

### 玩家操作

每个玩家卡片上有三个按钮：

| 按钮 | 触发事件 | 效果 |
|------|----------|------|
| Leave | `_playerleave` | 模拟玩家断线，标记为离线 |
| Join | `_playerjoin` | 模拟玩家重连，标记为在线 |
| Exit | `_playerexit` | 模拟玩家退出游戏 |

### 事件发送

- **Event Name**：事件名称（对应脚本中 `RegisterEvent` 注册的事件）
- **Event Data**：事件数据字符串（脚本中 `evalue` 参数）
- **Player Index**：目标玩家槽位，选 -1 表示房间事件
- **Presets**：预置按钮，点击自动填充事件名

### 日志查看器

- **Logs 标签**：脚本通过 `Log.Debug/Info/Error` 输出的日志，实时推送
- **Out Events 标签**：脚本调用 `MsSendMlEvent` 发出的事件
- **过滤**：按日志级别（DBG / INF / ERR）和关键字过滤
- **自动滚动**：勾选 Auto-scroll 跟踪最新日志

### 状态检查器

点击 **Refresh** 查看房间和玩家的完整状态快照，包括：存档数据、道具列表、连接状态、游戏时间。

---

## 玩家配置

创建房间时通过 `players` JSON 数组配置每个玩家。

```json
{
  "index": 0,
  "name": "Alice",
  "map_level": 5,
  "map_exp": 1200,
  "played_count": 10,
  "items": {
    "VIP001": 1,
    "GOLD_CARD": 3
  },
  "script_archive": "{\"last_login_t\":\"2024-10-21\",\"gold\":500}",
  "common_archive": {
    "boss_kill": "232"
  },
  "read_archive": {
    "boss_kill": "232"
  },
  "cfg_archive": {
    "season": "3"
  }
}
```

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `index` | int | 0 | 玩家槽位（对应脚本中的 `player_index`） |
| `name` | string | `Player_{index}` | 玩家昵称 |
| `map_level` | int | 1 | `MsGetPlayerMapLevel` 返回值 |
| `map_exp` | int | 0 | `MsGetPlayerMapExp` 返回值 |
| `played_count` | int | 1 | `MsGetPlayedCount` 返回值 |
| `items` | object | `{}` | 道具背包，`MsGetPlayerItem` 读取 |
| `script_archive` | string | `null` | 脚本存档，`MsGetScriptArchive` 读取 |
| `common_archive` | object | `{}` | 普通存档，`MsGetCommonArchive` 读取 |
| `read_archive` | object | `{}` | 可读存档，`MsGetReadArchive` 读取 |
| `cfg_archive` | object | `{}` | 全局只读存档，`MsGetCfgArchive` 读取 |

---

## 多房间测试示例

同时运行 apidemo 和 towner 两套脚本，验证多房间独立运行：

```python
import requests
import time

BASE = "http://localhost:5001"

# 创建 apidemo 房间（2 个玩家）
r1 = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": "D:/code2/mls/参考/mls-master/demo/apidemo/script",
    "mode_id": 1,
    "players": [
        {"index": 0, "name": "Alice", "items": {"VIP001": 1}},
        {"index": 1, "name": "Bob"}
    ]
}).json()
print(f"Room1: {r1['id']} - {r1['status']}")

# 创建 towner 房间（1 个玩家）
r2 = requests.post(f"{BASE}/api/rooms", json={
    "script_dir": "D:/code2/mls/参考/mls-master/demo/towner/script",
    "mode_id": 2,
    "players": [
        {"index": 0, "name": "Charlie", "items": {"VIP001": 1}}
    ]
}).json()
print(f"Room2: {r2['id']} - {r2['status']}")

time.sleep(3)

# 向 towner 发送 buy_tower 事件
requests.post(f"{BASE}/api/rooms/{r2['id']}/events", json={
    "ename": "buy_tower",
    "evalue": "",
    "player_index": 0
})

# 模拟 apidemo 房间的玩家断线和重连
requests.post(f"{BASE}/api/rooms/{r1['id']}/players/1/leave")
time.sleep(2)
requests.post(f"{BASE}/api/rooms/{r1['id']}/players/1/join")

# 查看两个房间的状态
for rid in [r1['id'], r2['id']]:
    state = requests.get(f"{BASE}/api/rooms/{rid}").json()
    print(f"{rid}: {state['status']}, {state['player_count']}P, {state['game_time']}s")

# 停止房间（触发存档保存）
requests.post(f"{BASE}/api/rooms/{r1['id']}/stop")
```

---

## REST API 参考

### 房间管理

#### `POST /api/rooms`

创建并启动一个房间。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `script_dir` | string | 是 | — | 脚本目录绝对路径（须包含 `main.lua`） |
| `mode_id` | int | 否 | 0 | 对局模式 ID |
| `players` | array | 否 | 单个 Player_0 | 玩家配置数组 |
| `auto_start` | bool | 否 | true | 创建后自动启动 |

**响应**：房间状态对象（201）。

#### `GET /api/rooms`

列出所有房间。返回房间状态对象数组。

#### `GET /api/rooms/{room_id}`

获取指定房间的状态。

#### `DELETE /api/rooms/{room_id}`

销毁房间。

#### `POST /api/rooms/{room_id}/start`

启动房间（加载脚本、触发 `_roomloaded`）。

#### `POST /api/rooms/{room_id}/stop`

停止房间（触发 `_roomover`）。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `reason` | string | `"GameEnd"` | 结束原因 |

### 玩家管理

#### `POST /api/rooms/{room_id}/players`

添加玩家到房间。

#### `PUT /api/rooms/{room_id}/players/{idx}`

更新玩家属性（name、items、archives 等）。

#### `DELETE /api/rooms/{room_id}/players/{idx}`

移除玩家。

#### `POST /api/rooms/{room_id}/players/{idx}/leave`

模拟玩家断线，触发 `_playerleave` 事件。

#### `POST /api/rooms/{room_id}/players/{idx}/join`

模拟玩家重连，触发 `_playerjoin` 事件。

#### `POST /api/rooms/{room_id}/players/{idx}/exit`

模拟玩家退出，触发 `_playerexit` 事件。

### 事件

#### `POST /api/rooms/{room_id}/events`

向房间发送自定义事件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `ename` | string | 是 | 事件名称 |
| `evalue` | string | 否 | 事件数据 |
| `player_index` | int | 否 | 目标玩家槽位（默认 0） |

#### `GET /api/rooms/{room_id}/state`

获取房间和所有玩家的完整状态快照。

---

## 模拟的 MLS API

模拟器注入的全部 Lua 全局函数，行为与 MLS 生产环境一致。

### 日志

| 函数 | 说明 |
|------|------|
| `Log.Debug(fmt, ...)` | 调试日志（生产模式可屏蔽） |
| `Log.Info(fmt, ...)` | 信息日志 |
| `Log.Error(fmt, ...)` | 错误日志 |

频率限制：100 秒最多 1000 条，超过后熔断到下一个周期。单条上限 2000 字节。

### 定时器

| 函数 | 说明 |
|------|------|
| `Timer.After(seconds, callback)` | 延迟执行一次 |
| `Timer.NewTicker(seconds, callback)` | 循环定时器，返回带 `:Cancel()` 方法的对象 |

### 事件系统

| 函数 | 说明 |
|------|------|
| `RegisterEvent(name, callback)` | 注册事件，返回 ID。回调签名：`(id, ename, evalue, player_index)` |
| `UnregisterEvent(id)` | 取消注册 |
| `MsSendMlEvent(player_index, ename, evalue)` | 发送事件到客户端（显示在 Out Events 面板） |
| `MsEnd(player_index, reason)` | 停止脚本执行 |

事件名约束：最长 32 字节，仅允许 `[a-zA-Z0-9:_]`。事件数据上限 900 字节。

### 玩家查询

| 函数 | 返回值 |
|------|--------|
| `MsGetPlayerName(idx)` | 玩家昵称（string） |
| `MsGetPlayerMapLevel(idx)` | 地图等级（int） |
| `MsGetPlayerMapExp(idx)` | 地图经验（int） |
| `MsGetPlayedTime(idx)` | 游玩时间，秒（int，动态计算） |
| `MsGetTestPlayTime(idx)` | 测试大厅游玩时间，秒（int） |
| `MsGetPlayedCount(idx)` | 游玩次数（int） |

### 房间查询

| 函数 | 返回值 |
|------|--------|
| `MsGetRoomStartTs()` | 游戏开始时间戳（int） |
| `MsGetRoomLoadedTs()` | 加载完成时间戳（int） |
| `MsGetRoomGameTime()` | 已过去时间，秒（int，动态计算） |
| `MsGetRoomPlayerCount()` | 在线玩家数（int，不含断线玩家） |
| `MsGetRoomModeId()` | 模式 ID（int） |

### 道具

| 函数 | 说明 |
|------|------|
| `MsGetPlayerItem(idx, key)` | 查询道具数量，不存在返回 0 |
| `MsConsumeItem(idx, iteminfo_json)` | 消耗道具，返回 trans_id。异步触发 `_citemret` 事件返回结果 |

### 存档

| 函数 | 说明 |
|------|------|
| `MsGetScriptArchive(idx)` | 获取脚本存档（string），不存在返回 nil |
| `MsSaveScriptArchive(idx, data)` | 保存脚本存档，上限 1MB |
| `MsGetCommonArchive(idx, key)` | 获取普通存档 |
| `MsGetReadArchive(idx, key)` | 获取可读存档 |
| `MsSetReadArchive(idx, key, value)` | 设置可读存档，自动触发 `_rdata` 事件 |
| `MsGetCfgArchive(idx, key)` | 获取全局只读存档 |

### 内置事件

服务端自动触发，脚本通过 `RegisterEvent` 接收。

| 事件 | 触发时机 | 数据格式 |
|------|----------|----------|
| `_roomloaded` | 脚本加载完成 | `{"players": [0, 1, ...]}` |
| `_roomover` | 房间结束 | `{"reason": "GameEnd"}` |
| `_playerexit` | 玩家退出 | `{"reason": "Logout"}` |
| `_playerleave` | 玩家断线 | `{"reason": "Disconnect"}` |
| `_playerjoin` | 玩家重连 | `{"reason": "Connect"}` |
| `_rdata` | 可读存档更新 | `"key\tvalue"` |
| `_citemret` | 道具消耗结果 | `{"trans_id": N, "errnu": 0, "iteminfo": {...}}` |

### JSON

| 函数 | 说明 |
|------|------|
| `json.encode(value)` | Lua table 转 JSON 字符串 |
| `json.decode(string)` | JSON 字符串转 Lua table |

---

## 错误码

| 码 | 说明 |
|----|------|
| 0 | 成功 |
| 1 | 未知错误 |
| 2 | 房间不存在 |
| 3 | 玩家不存在 |
| 4 | 事件 Key 长度不合规 |
| 5 | 事件 Key 内容不合规 |
| 6 | 事件 Value 长度不合规 |
| 7 | 事件 Value 内容不合规 |
| 8 | 存档 Key 长度不合规 |
| 9 | 存档 Value 长度不合规 |
| 10 | 文本内容超限 |
| 11 | 脚本存档超长度限制 |
| 1259 | 道具数量不足 |
| 10133 | 包裹内没有指定物品 |

---

## 与 MLS 生产环境的差异

| 项目 | 生产环境 | 模拟器 |
|------|----------|--------|
| Lua 版本 | 5.3.6 | 5.3（lupa 内置） |
| 脚本执行超时 | 5 秒中断 | 无超时限制 |
| 内存限制 | 10MB | 无限制 |
| 事件传输 | 网络传输到 War3 客户端 | 显示在 Web 面板 |
| 存档持久化 | 服务端数据库 | 内存（进程退出后丢失） |
| `os.execute` / `io.open` | 不可用 | 可用（未沙箱化） |

---

## 常见问题

### 脚本加载报 "module not found"

检查 `script_dir` 是否指向包含 `main.lua` 的目录。模拟器支持 `require("../xxx")` 相对路径和 `require("xxx.yyy")` 点号路径。

### 中文路径显示乱码

不影响功能。模拟器通过 Python 读取文件（正确处理 Unicode），Lua 内部的路径显示可能出现乱码，但不影响脚本加载和执行。

### 端口被占用

指定其他端口：`python app.py 5002`。

### 存档数据丢失

模拟器将存档存储在内存中，进程退出后丢失。如需跨会话保留存档，在创建房间时通过 `script_archive` 字段预设存档数据。
