-- 地图中用到的一些测试命令，方便进行开发，在正式上线后，需要进行屏蔽掉该功能
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
-- 查询/修改脚本存档指令
local function getmsdata(id, ename, evalue, player_index)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end
    local args_e = split(evalue, " ", 3)
    local hkey = args_e[2]
    local set_value = args_e[3]
    local hvalue = player:get_msdata(hkey)
    if hvalue == nil then
        hvalue = "nil"
    end
    if set_value ~= nil then
        player:set_msdata(hkey, set_value)
        hvalue = hvalue .. '->' .. set_value
        ChatWar3(player_index, "更新脚本存档:" .. hkey .. ":" .. hvalue)
    else
        ChatWar3(player_index, "查询脚本存档:" .. hkey .. ":" .. hvalue)
    end
end

-- 查询普通存档指令
local function getcdata(id, ename, evalue, player_index)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end
    local args_e = split(evalue, " ", 3)
    local hkey = args_e[2]
    local hvalue = MsGetCommonArchive(player_index, hkey)
    if hvalue == nil then
        hvalue = "nil"
    end
    ChatWar3(player_index, "查询普通存档:" .. hkey .. ":" .. hvalue)
end

-- 查询/修改可读存档指令
local function getrdata(id, ename, evalue, player_index)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end
    local args_e = split(evalue, " ", 3)
    local hkey = args_e[2]
    local set_value = args_e[3]
    local hvalue = MsGetReadArchive(player_index, hkey)
    if hvalue == nil then
        hvalue = "nil"
    end
    if set_value ~= nil then
        MsSetReadArchive(player_index, hkey, set_value)
        hvalue = hvalue .. '->' .. set_value
        ChatWar3(player_index, "更新只读存档:" .. hkey .. ":" .. hvalue)
    else
        ChatWar3(player_index, "查询只读存档:" .. hkey .. ":" .. hvalue)
    end
end

-- 查看道具
local function getitem(id, ename, evalue, player_index)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end
    local args_e = split(evalue, " ", 2)
    local hkey = args_e[2]
    local hvalue = MsGetPlayerItem(player_index, hkey)
    if hvalue == nil then
        hvalue = "0"
    end
    ChatWar3(player_index, "查询道具:" .. hkey .. ":" .. hvalue)
end

-- 方便开发，可以增加一个debug指令，获取脚本存档数据
RegisterEvent('msdata', getmsdata)

-- 方便开发，可以增加一个debug指令，获取普通存档数据
RegisterEvent('cdata', getcdata)

-- 方便开发，可以增加一个debug指令，获取可读存档数据
RegisterEvent('rdata', getrdata)

-- 方便开发，可以增加一个debug指令，查看道具
RegisterEvent('item', getitem)

