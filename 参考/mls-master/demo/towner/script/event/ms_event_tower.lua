-- 塔相关事件
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
-- 房间加载完毕后
local function buy_tower(id, ename, evalue, player_index)
    -- 如果想移除事件注册，则使用 UnregisterEvent(id)

    LogWar3(player_index, player.player_name .. " 购买塔塔塔塔塔塔")
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end

    -- 限制每一局最多造多少塔
    if g_room.buy_tower_count >= 5 then
        ChatWar3(player_index, player.player_name .. "造塔已满")
        return
    end

    -- 检测玩家的金币
    if player.gold < 200 then
        ChatWar3(player_index, player.player_name .. "金币不足")
        return
    end
    -- 扣除金币
    player.gold = player.gold - 200
    MsSendMlEventRaw(player_index, "asset_update", tostring(player.gold))

    -- 随机一个塔
    local tower_list = {"I000", "I001", "I002", "I003", "I004", "I005", "I006", "I007", "I008", "I009"}
    -- 从数组中随机选择一个
    local randomIndex = math.random(#tower_list)
    local tower_id = tower_list[randomIndex]
    -- 插入玩家的塔列表中
    table.insert(player.tower_list, tower_id)
    local result = MsSendMlEventRaw(player_index, "buy_tower", tostring(tower_id))

    g_room.buy_tower_count = g_room.buy_tower_count + 1
end

-- 房间加载完毕后
local function kill_unit(id, ename, evalue, player_index)
    LogWar3(player_index, player.player_name .. " kill_unit->" .. evalue)

    local attack_round = tonumber(evalue)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end
    -- 杀怪给钱
    player.gold = player.gold + attack_round * 100
    MsSendMlEventRaw(player_index, "asset_update", tostring(player.gold))

    -- 如果限制每个玩家每天杀的boss个数
    -- 用脚本存档来记录

    -- 统计杀死boss的次数
    -- 放到 可读存档，ECA中下一局可以获取到最新的数据
    local hkey = "boss_kill"
    local boss_kill = MsGetReadArchive(player_index, hkey)
    local boss_count = tonumber(boss_kill)

    if boss_count == nil then
        boss_count = 1
    else
        boss_count = boss_count + 1
    end

    -- -- 读取一次数据即可，无需重复读取
    -- local boss_kill_post = MsGetScriptArchive(player_index, hkey)

    -- if boss_kill_post == boss_kill then
    --     -- Do nothing, the values match
    -- else
    --     ChatWar3(player_index,  player.player_name .. " 检测到怪物数量被作弊！！")
    -- end

    MsSetReadArchive(player_index, hkey, tostring(boss_count))

    ChatWar3(player_index, player.player_name .. " 怪物击杀数：->" .. boss_count)
end

--

local function money_666(id, ename, evalue, player_index)
    LogWar3(player_index, player.player_name .. " 666->" .. evalue)
    local player = g_room_players[player_index]
    if player == nil then
        LogDebug("getmsdata player not exist:%d", player_index)
        return
    end

    if player.gold < 5000 then
        player.gold = player.gold + 666
    else
        ChatWar3(player_index, player.player_name .. " 你作弊拉，钱太多了")
    end

    MsSendMlEventRaw(player_index, "asset_update", tostring(player.gold))
end

-- 购买一个塔
RegisterEvent('buy_tower', buy_tower)

RegisterEvent('kill_unit', kill_unit)

RegisterEvent('money_666', money_666)
