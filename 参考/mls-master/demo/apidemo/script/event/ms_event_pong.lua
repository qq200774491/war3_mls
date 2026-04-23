-- 塔相关事件
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
-- 处理客户端的pong事件
local function pong(id, ename, evalue, player_index)
    -- 如果想移除事件注册，则使用 UnregisterEvent(id)
    local now_t = os.clock()
    local send_t = tonumber(evalue)
    local ping_t =  math.floor( (now_t - send_t)*1000)
    -- 发送ping
    ChatWar3(player_index, "MLS的ping:%d(ms)", ping_t)
end

-- API:注册事件[RegisterEvent]
-- 注册来自客户端发送的pong事件
RegisterEvent('pong', pong)