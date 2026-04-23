-- 脚本逻辑处理 -- 对局开始
require("../dao/dao_player")
require("../dao/dao_room")

-- 即将进入API测试
test_api_list = {}

function enter_test_api(player, evalue)
    player.testapi_index = player.testapi_index + 1
    LogInfo("player:%s enter test api index=%d", player.player_name, player.testapi_index)
    if  player.testapi_index > #test_api_list then
        player.testapi_index = 1
    end
    -- 调用测试API
    test_api_list[player.testapi_index](player, evalue)
end

-- 测试API:进入测试
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    player.testapi_round = player.testapi_round + 1
    -- 如果需要测试脚本是否更新，请在这个地方修改版本
    local script_version = "0.0.1"
    ChatWar3(player_index, "MLS API TEST[%s] 进入第%d轮测试====脚本版本:%s========", "MsSendMlEvent", player.testapi_round, script_version)
    last_logout_t = player:get_msdata("last_logout_t")
    play_game_count = player:get_msdata("play_game_count")
    ChatWar3(player_index, "MLS API TEST[%s] 进入游戏%s次====最后一次游戏时间:%s========", "MsSendMlEvent", safeString(play_game_count), safeString(last_logout_t))
end)

-- 测试API: 测试同帧发送谁
-- 测试API:MsSendMlEvent
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local randomNumber = math.random(1, 100)
    for i = 1, randomNumber do
        player.send_stat_msg = player.send_stat_msg + 1
        -- MsSendMlEvent(player_index, "api_send_msg_stat", string.format("服务器长度:%d AAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBBBBBB%d", player.send_stat_msg, player.send_stat_msg))
        MsSendMlEvent(player_index, "api_send_msg_stat", string.format("服务器长度:%d AAAAAAAAAAAAAAA注意观察AAAAABBBBBBBBBBB %d", player.send_stat_msg, player.send_stat_msg))
    end
    ChatWar3(player_index, "MLS API TEST:[%s] 发送消息:%d个", "MsSendMlEvent",player.send_stat_msg)
end)

-- 测试API:MsGetReadArchive|MsSetReadArchive
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local readachieve_count = MsGetReadArchive(player_index, "apit_readachieve_count")
    if readachieve_count == nil or readachieve_count == "" then
        readachieve_count = 1
        MsSetReadArchive(player_index, "apit_readachieve_count", tostring(readachieve_count))
    else
        readachieve_count = tonumber(readachieve_count) + 1
        MsSetReadArchive(player_index, "apit_readachieve_count", tostring(readachieve_count))
    end
    local readachieve_count = MsGetReadArchive(player_index, "apit_readachieve_count")
    local readachieve_count = tonumber(readachieve_count)
    ChatWar3(player_index, "MLS API TEST:[%s] 进入测试:%d次", "MsGetReadArchive|MsSetReadArchive", readachieve_count)
end)

-- 测试API:get_msdata|set_msdata
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local scriptachieve_count = player:get_msdata("apit_scriptachieve_count")
    if scriptachieve_count == nil or scriptachieve_count == "" then
        scriptachieve_count = 1
        player:set_msdata("apit_scriptachieve_count",scriptachieve_count)
    else
        scriptachieve_count = scriptachieve_count + 1
        player:set_msdata("apit_scriptachieve_count",scriptachieve_count)
    end
    local scriptachieve_count = player:get_msdata("apit_scriptachieve_count")
    ChatWar3(player_index, "MLS API TEST:[%s] 进入测试:%d次", "get_msdata|set_msdata", scriptachieve_count)
end)

-- 测试API:MsGetCommonArchive
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local archieve_value = MsGetCommonArchive(player_index, "boss_kill")
    ChatWar3(player_index, "MLS API TEST:[%s] 普通存档: 存档Key: %s 存档值:%s", "MsGetCommonArchive", "boss_kill", tostring(archieve_value))
end)

-- 测试API:MsGetPlayerItem
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local item_count = MsGetPlayerItem(player_index, "VIP001")
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家道具: 道具Key: %s 拥有个数:%d", "MsGetPlayerItem", "VIP001", item_count)
end)

-- 测试API:MsGetPlayerName
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local player_name = MsGetPlayerName(player_index)
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家昵称: %s", "MsGetPlayerName", player_name)
end)

-- 测试API:MsGetPlayerMapLevel
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local map_level = MsGetPlayerMapLevel(player_index)
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家地图等级: %d", "MsGetPlayerMapLevel", map_level)
end)

-- 测试API:MsGetPlayerMapExp
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local map_exp = MsGetPlayerMapExp(player_index)
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家地图经验: %d", "MsGetPlayerMapExp", map_exp)
end)

-- 测试API:MsGetPlayedTime
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local played_time = MsGetPlayedTime(player_index)
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家地图游玩时间: %d", "MsGetPlayedTime", played_time)
end)

-- 测试API:MsGetPlayedCount
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local played_count = MsGetPlayedCount(player_index)
    ChatWar3(player_index, "MLS API TEST:[%s] 玩家地图游玩次数: %d", "MsGetPlayedCount", played_count)
end)

-- 测试API:MsGetRoomStartTs
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local start_ts = MsGetRoomStartTs()
    ChatWar3(player_index, "MLS API TEST:[%s] 游戏对局开始时间: %d", "MsGetRoomStartTs", start_ts)
end)

-- 测试API:MsGetRoomLoadedTs
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local start_ts = MsGetRoomLoadedTs()
    ChatWar3(player_index, "MLS API TEST:[%s] 游戏脚本加载完成时间: %d", "MsGetRoomLoadedTs", start_ts)
end)

-- 测试API:MsGetRoomGameTime
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local game_t = MsGetRoomGameTime()
    ChatWar3(player_index, "MLS API TEST:[%s] 游戏逝去时间: %d", "MsGetRoomGameTime", game_t)
end)

-- 测试API:MsGetRoomPlayerCount
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local player_count = MsGetRoomPlayerCount()
    ChatWar3(player_index, "MLS API TEST:[%s] 房间人数: %d", "MsGetRoomPlayerCount",player_count)
end)

-- 测试API:MsGetRoomModeId
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local mode_id = MsGetRoomModeId()
    ChatWar3(player_index, "MLS API TEST:[%s] 房间模式ID: %d", "MsGetRoomModeId",mode_id)
end)

-- 定义字符集
local charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
-- 函数生成随机字符串
local function randomString(length)
    local result = {}
    local charsetLength = #charset
    for i = 1, length do
        local randomIndex = math.random(1, charsetLength)
        result[i] = charset:sub(randomIndex, randomIndex)
    end
    return table.concat(result)
end

-- 测试API: 性能测试
-- 测试API:MsGetScriptArchive|MsSaveScriptArchive
table.insert(test_api_list, function (player, evalue)
    local player_index = player.player_index
    local start_t = os.clock()
    local cost_load_t = os.clock() - start_t
    local start_t = os.clock()
    local key_count = 0
    for _ in pairs(player.ms_data) do
        key_count = key_count + 1
    end
    local rand_key_count = 1000
    if key_count < 2000 then
        for i = 1, rand_key_count do
            local key = randomString(10)
            local value = randomString(100)
            player:set_msdata(key, value)
        end
    end
    local cost_make_t = os.clock() - start_t
    local start_t = os.clock()
    player:save_msdata()
    local cost_save_t = os.clock() - start_t
    local testapi_json = json.encode(player.ms_data)
    local length = string.len(testapi_json)/1000
    ChatWar3(player_index, "MLS API TEST:[%s] 保存:%d个 %0.3fK 消耗 解析:%0.3f 设置:%0.3f 保存:%0.3f", "MsGetScriptArchive|MsSaveScriptArchive ",rand_key_count, length, cost_load_t,cost_make_t,cost_save_t)
end)

-- -- 测试API:jons.decode|encode
-- table.insert(test_api_list, function (player, evalue)
--     local player_index = player.player_index
--     local start_t = os.clock()
--     local json_data = {}
--     if player.testapi_json ~= nil then
--         json_data = json.decode(player.testapi_json)
--     end
--     local cost_load_t = os.clock() - start_t
--     local start_t = os.clock()
--     local rand_key_count = 2000
--     local key_count = 0
--     for _ in pairs(json_data) do
--         key_count = key_count + 1
--     end
--     if key_count < 3000 then
--         for i = 1, rand_key_count do
--             local key = randomString(16)
--             local value = randomString(100)
--             json_data[key] = value
--         end
--         key_count = key_count + rand_key_count
--     end
--     local cost_make_t = os.clock() - start_t
--     local start_t = os.clock()
--     player.testapi_json = json.encode(json_data)
--     local length = string.len(player.testapi_json)/1000
--     local cost_save_t = os.clock() - start_t
--     ChatWar3(player_index, "MLS API TEST:[%s] 保存:%d个 %0.3fK 消耗 解析:%0.3f 设置:%0.3f 保存:%0.3f", "jons.decode|encode",key_count, length, cost_load_t,cost_make_t,cost_save_t)
-- end)


-- -- 测试API:MsGetRoomLoadedTs
-- table.insert(test_api_list, function (player, evalue)
--     MsSendMlEventRaw(-1, "_mlroomfail", "TestFail")
-- end)

