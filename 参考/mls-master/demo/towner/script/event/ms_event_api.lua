-- KK平台提供的默认事件
-- 事件用_开头
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
require('../logic/logic_room')

-- 房间加载完毕后
local function ms_room_loaded(id, ename, evalue, player_index)
    LogInfo("ms_room_loaded ename=%s evalue=%s player_index=%d", ename, evalue, player_index)
    on_room_loaded(evalue)
    LogInfo("ms_room_loaded")
end

-- 房间结束后
local function ms_room_over(id, ename, evalue, player_index)
    LogInfo("ms_room_over ename=%s evalue=%s player_index=%d", ename, evalue, player_index)
    on_room_over(evalue)
end

-- 玩家退出游戏
local function ms_player_exit(id, ename, evalue, player_index)
    LogInfo("ms_player_exit ename=%s evalue=%s player_index=%d", ename, evalue, player_index)
end

-- 玩家离开游戏，可能断线重连回来
local function ms_player_leave(id, ename, evalue, player_index)
    LogInfo("ms_player_leave ename=%s evalue=%s player_index=%d", ename, evalue, player_index)
end

-- 注册事件
RegisterEvent('_roomloaded', ms_room_loaded)
RegisterEvent('_roomover', ms_room_over)
RegisterEvent('_playerexit', ms_player_exit)
RegisterEvent('_playerleave', ms_player_leave)

