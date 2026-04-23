-- 塔相关事件
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
-- 处理客户端的pong事件

require('../logic/logic_testapi')

local function testapi(id, ename, evalue, player_index)
    -- 如果想移除事件注册，则使用 UnregisterEvent(id)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end

    player.testapi_timer  = Timer.NewTicker(2, function ()
        enter_test_api(player, evalue)
    end)
end

-- API:注册事件[RegisterEvent]
-- 注册来自客户端发送的pong事件
RegisterEvent('testapi', testapi)