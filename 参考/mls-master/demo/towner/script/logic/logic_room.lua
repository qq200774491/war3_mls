-- 脚本逻辑处理 -- 对局开始
require("../dao/dao_player")
require("../dao/dao_room")

function change_player_asset()
    for player_index, player in pairs(g_room_players) do
        player.gold = player.gold + 100
        MsSendMlEventRaw(player_index, "asset_update", tostring(player.gold))
    end
end

-- 同步道具
function sync_game_start_data()
    local now_t = os.date("%Y-%m-%d %H:%M:%S")
    for player_index, player in pairs(g_room_players) do
        -- 获取玩家最近一次进入游戏的时间
        local last_login_t = player:get_msdata("last_login_t")
        if last_login_t == nil then
            player:set_msdata("day_play_count", 1)
            ChatWar3(player_index, player.player_name .. " 欢迎第一次来玩游戏")
        elseif string.sub(last_login_t, 1, 10) ~= string.sub(now_t, 1, 10) then
            local play_count = player:get_msdata("day_play_count") + 1
            player:set_msdata("day_play_count", play_count)
            ChatWar3(player_index, player.player_name .. " 这是你第" .. play_count .. "来玩游戏")
        end
        -- 更新玩家最近一次进入游戏时间
        player:set_msdata("last_login_t", now_t)

        -- 同步累计闯关次数
        local boss_kill = MsGetReadArchive(player_index, "boss_kill")
        if boss_kill == nil then
            ChatWar3(player_index, player.player_name .. " 还未击杀怪物，加油...")
        else
            ChatWar3(player_index, player.player_name .. " 累计击杀怪物.." .. boss_kill)
        end
    end
end
function gameinit()
    -- 如果数据量太大，则进行分批同步
    -- msg_body["debug"] = "close"pfile
    for player_index, player in pairs(g_room_players) do
        -- 同步初始玩家金币
        MsSendMlEventRaw(player_index, "asset_update", tostring(player.gold))
    end

    -- 同步游戏初始化数据
    Timer.After(0.1, sync_game_start_data)

    -- 每60秒发送一次资产
    local asset_timer = Timer.NewTicker(60, change_player_asset)
end

function on_room_loaded(evalue)
    LogInfo("on_room_loaded evalue=%s", evalue)
    local room_info = json.decode(evalue)
    local players = room_info["players"]
    for i = 1, #players do
        local player_index = math.floor(players[i])
        LogInfo("on_room_loaded player_index=%s", player_index)
        player = MLSPlayer:new()
        player:init()
        player.player_index = player_index
        player.player_name = MsGetPlayerName(player_index)
        player.atk = 50
        player.gold = 1000
        -- 存在VIP道具，则金币增加50%
        local vip_item = MsGetPlayerItem(player_index, "VIP001")
        if vip_item ~= 0 then
            player.gold = 1500
        end
        g_room_players[player_index] = player

        -- 初始化玩家数据 
        g_room_players[player_index]:init()
        g_room_players[player_index]:dump()
        ChatWar3(player_index, player.player_name .. " 进入游戏")
    end

    -- 执行游戏初始化
    -- 5秒，游戏数据下发
    Timer.After(3, gameinit)
end

function on_room_over(evalue)
    local game_end_t = os.date("%Y-%m-%d %H:%M:%S")
    for player_index, player in pairs(g_room_players) do
        -- 保存玩家最后一次退出时间
        player:set_msdata("last_logout_t", game_end_t)

        -- 游戏结束前，需要调用保存脚本存档
        player:save_msdata()
    end
    -- 对局结束
    MsEnd(-1, "room_over")
end
