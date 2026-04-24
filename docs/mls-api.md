# MLS API 参考

模拟器注入到每个房间 Lua VM 的全部全局 API，与平台 MLS 运行时保持一致。

官方 API 文档见 `参考/mls-master/API.md`。

## 日志

```lua
Log.Debug(fmt, ...)
Log.Info(fmt, ...)
Log.Error(fmt, ...)
```

- 支持 `string.format` 格式化。
- 单条日志最大 2000 字节。
- 频率限制：100 秒内最多 1000 条，超限后熔断至下一周期。

## 定时器

```lua
-- 延后执行（单位：秒）
Timer.After(10, function()
    Log.Debug("after 10s")
end)

-- 循环定时器
local ticker = Timer.NewTicker(1, function()
    Log.Debug("every 1s")
end)
ticker:Cancel()  -- 取消
```

## 事件

```lua
-- 注册事件回调
local id = RegisterEvent("buy_tower", function(id, ename, evalue, player_index)
    local data = json.decode(evalue)
    -- 处理事件
end)

-- 取消注册
UnregisterEvent(id)

-- 发送事件给客户端
MsSendMlEvent(player_index, "asset_update", json.encode({gold = 100}))
-- 返回 0 表示成功
```

事件名限制：最大 32 字节，不能以 `_` 开头（防止与平台内置事件冲突）。

事件数据限制：最大 900 字节。

## 内置系统事件

| 事件名 | 说明 | 数据格式 |
| --- | --- | --- |
| `_roomloaded` | 房间加载完成 | `{"players": [0, 1]}` |
| `_roomover` | 房间结束 | `{"reason": "GameEnd"}` |
| `_playerexit` | 玩家退出 | `{"reason": "Logout"}` |
| `_playerleave` | 玩家断线 | `{"reason": "Disconnect"}` |
| `_playerjoin` | 玩家重连 | `{"reason": "Connect"}` |

## 玩家查询

```lua
MsGetPlayerName(player_index)       -- 玩家昵称 (string)
MsGetPlayerMapLevel(player_index)   -- 地图等级 (int)
MsGetPlayerMapExp(player_index)     -- 地图经验 (int)
MsGetPlayedTime(player_index)       -- 游玩时间/秒 (int)
MsGetTestPlayTime(player_index)     -- 测试大厅时间/秒 (int)
MsGetPlayedCount(player_index)      -- 游玩次数 (int)
```

## 房间查询

```lua
MsGetRoomStartTs()       -- 游戏开始时间戳/秒
MsGetRoomLoadedTs()      -- 加载完成时间戳/秒
MsGetRoomGameTime()      -- 已过去的游戏时间/秒
MsGetRoomPlayerCount()   -- 在线玩家数
MsGetRoomModeId()        -- 模式 ID
```

## 道具

```lua
MsGetPlayerItem(player_index, "VIP001")  -- 道具数量 (int)

-- 消耗道具（异步）
local trans_id = MsConsumeItem(player_index, '{"VIP001": 1}')
-- 结果通过 _citemret 事件回调返回
```

`_citemret` 事件数据：`{"trans_id": 1, "errnu": 0, "iteminfo": {"VIP001": 1}}`

## 存档

```lua
-- 脚本存档（最大 1MB）
local data = MsGetScriptArchive(player_index)   -- string | nil
MsSaveScriptArchive(player_index, json_string)   -- 返回错误码

-- 普通存档
MsGetCommonArchive(player_index, key)  -- string | nil

-- 可读存档
MsGetReadArchive(player_index, key)    -- string | nil
MsSetReadArchive(player_index, key, value)  -- 触发 _rdata 事件

-- 全局只读存档
MsGetCfgArchive(player_index, key)     -- string | nil
```

## 控制

```lua
MsEnd(player_index, "reason")  -- 停止脚本运行
```

## JSON

```lua
local str = json.encode({key = "value"})
local tbl = json.decode('{"key":"value"}')
```

内嵌纯 Lua 实现，无需 require。

## 模块加载

```lua
require("event.ms_event_api")   -- 加载 event/ms_event_api.lua
require("../map/xxx")           -- 支持相对路径
```

从当前模块所在目录和脚本根目录搜索，支持 `init.lua` 目录模块。

## 常用错误码

| 错误码 | 说明 |
| --- | --- |
| 0 | 正常 |
| 3 | 玩家不存在 |
| 4 | 事件 Key 长度不符 |
| 6 | 事件 Value 长度不符 |
| 11 | 脚本存档超过长度限制 |
| 1259 | 道具数量不足 |
