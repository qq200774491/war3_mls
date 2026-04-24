---@meta
--- MLS 云脚本 API 类型定义
--- 用于 LuaLS 智能提示，放入 mls-sim/types/ 目录
--- VS Code 中配置 Lua.workspace.library 指向此目录即可获得补全

---------------------------------------------------------------------------
-- 日志 API
---------------------------------------------------------------------------

---@class LogModule
---@field Debug fun(fmt: string, ...: any) 调试日志（生产环境屏蔽）
---@field Info fun(fmt: string, ...: any) 信息日志
---@field Error fun(fmt: string, ...: any) 错误日志
Log = {}

---------------------------------------------------------------------------
-- 定时器 API
---------------------------------------------------------------------------

---@class TimerTicker
local TimerTicker = {}

---取消循环定时器
function TimerTicker:Cancel() end

---@class TimerModule
Timer = {}

---延迟执行一次
---@param seconds number 延迟秒数
---@param callback fun() 回调函数
function Timer.After(seconds, callback) end

---创建循环定时器
---@param seconds number 间隔秒数
---@param callback fun() 回调函数
---@return TimerTicker ticker 可调用 ticker:Cancel() 取消
function Timer.NewTicker(seconds, callback) end

---------------------------------------------------------------------------
-- 事件 API
---------------------------------------------------------------------------

---注册脚本事件回调
---@param ename string 事件名（不能用 _ 开头，最长 32 字节，[a-zA-Z0-9:_]+）
---@param callback fun(id: integer, ename: string, evalue: string, player_index: integer)
---@return integer id 注册编号，可用于 UnregisterEvent
function RegisterEvent(ename, callback) end

---取消事件注册
---@param id integer 注册编号（RegisterEvent 返回值）
function UnregisterEvent(id) end

---------------------------------------------------------------------------
-- 控制 API
---------------------------------------------------------------------------

---发送事件到客户端（War3）
---@param player_index integer 玩家槽位
---@param ename string 事件名（最长 32 字节，[a-zA-Z0-9:_]+）
---@param evalue string 事件数据（最长 900 字节）
---@return integer errcode 0=成功
function MsSendMlEvent(player_index, ename, evalue) end

---停止脚本执行
---@param player_index integer 玩家槽位
---@param reason string 原因
---@return integer errcode 0=成功
function MsEnd(player_index, reason) end

---------------------------------------------------------------------------
-- 玩家查询 API
---------------------------------------------------------------------------

---获取玩家昵称
---@param player_index integer 玩家槽位
---@return string name 玩家昵称
function MsGetPlayerName(player_index) end

---获取玩家当前地图等级
---@param player_index integer 玩家槽位
---@return integer level 地图等级
function MsGetPlayerMapLevel(player_index) end

---获取玩家当前地图经验
---@param player_index integer 玩家槽位
---@return integer exp 地图经验
function MsGetPlayerMapExp(player_index) end

---获取玩家已游玩时间（秒）
---@param player_index integer 玩家槽位
---@return integer seconds 游玩时间
function MsGetPlayedTime(player_index) end

---获取玩家测试大厅游玩时间（秒）
---@param player_index integer 玩家槽位
---@return integer seconds 测试游玩时间
function MsGetTestPlayTime(player_index) end

---获取玩家当前地图游玩次数
---@param player_index integer 玩家槽位
---@return integer count 游玩次数
function MsGetPlayedCount(player_index) end

---------------------------------------------------------------------------
-- 房间查询 API
---------------------------------------------------------------------------

---获取游戏开始时间戳（秒）
---@return integer timestamp
function MsGetRoomStartTs() end

---获取游戏加载完成时间戳（秒）
---@return integer timestamp
function MsGetRoomLoadedTs() end

---获取游戏已经过去的时间（秒，从加载完成开始计时）
---@return integer seconds
function MsGetRoomGameTime() end

---获取在线玩家人数（不含 AI 和断线玩家）
---@return integer count
function MsGetRoomPlayerCount() end

---获取对局模式 ID
---@return integer mode_id
function MsGetRoomModeId() end

---------------------------------------------------------------------------
-- 道具 API
---------------------------------------------------------------------------

---获取玩家道具数量
---@param player_index integer 玩家槽位
---@param key string 道具 key
---@return integer quantity 道具数量，不存在返回 0
function MsGetPlayerItem(player_index, key) end

---消耗玩家道具（异步，结果通过 _citemret 事件返回）
---@param player_index integer 玩家槽位
---@param iteminfo string 道具消耗信息 JSON：'{"key1": 1, "key2": 2}'
---@return integer trans_id 业务 ID，关联回调结果
function MsConsumeItem(player_index, iteminfo) end

---------------------------------------------------------------------------
-- 存档 API
---------------------------------------------------------------------------

---获取脚本存档数据
---@param player_index integer 玩家槽位
---@return string|nil data 存档数据，不存在返回 nil
function MsGetScriptArchive(player_index) end

---保存脚本存档数据（上限 1MB）
---@param player_index integer 玩家槽位
---@param data string 序列化后的存档数据
---@return integer errcode 0=成功
function MsSaveScriptArchive(player_index, data) end

---获取普通存档数据
---@param player_index integer 玩家槽位
---@param key string 存档 key
---@return string|nil value 存档数据，不存在返回 nil
function MsGetCommonArchive(player_index, key) end

---获取可读存档数据
---@param player_index integer 玩家槽位
---@param key string 存档 key
---@return string|nil value 存档数据，不存在返回 nil
function MsGetReadArchive(player_index, key) end

---保存可读存档数据（自动触发 _rdata 事件）
---@param player_index integer 玩家槽位
---@param key string 存档 key
---@param value string 存档数据
---@return integer errcode 0=成功
function MsSetReadArchive(player_index, key, value) end

---获取全局只读存档数据
---@param player_index integer 玩家槽位
---@param key string 存档 key
---@return string|nil value 存档数据，不存在返回 nil
function MsGetCfgArchive(player_index, key) end

---------------------------------------------------------------------------
-- JSON 库
---------------------------------------------------------------------------

---@class JsonModule
json = {}

---将 Lua 值编码为 JSON 字符串
---@param value any Lua table/string/number/boolean/nil
---@return string json_string
function json.encode(value) end

---将 JSON 字符串解码为 Lua 值
---@param s string JSON 字符串
---@return any value
function json.decode(s) end

---------------------------------------------------------------------------
-- 内置事件说明（通过 RegisterEvent 接收）
---------------------------------------------------------------------------
-- _roomloaded   : 房间加载完成    数据: {"players": [0, 1, ...]}
-- _roomover     : 房间结束        数据: {"reason": "GameEnd"}
-- _playerexit   : 玩家退出        数据: {"reason": "Logout"}
-- _playerleave  : 玩家断线        数据: {"reason": "Disconnect"}
-- _playerjoin   : 玩家重连        数据: {"reason": "Connect"}
-- _rdata        : 可读存档更新    数据: "key\tvalue"
-- _citemret     : 道具消耗结果    数据: {"trans_id": N, "errnu": 0, "iteminfo": {...}}
-- _mlroomfail   : 脚本启动失败    数据: "失败原因"
