-- 定义一个玩家
MLSPlayer = {}
MLSPlayer.meta = {
    __index = MLSPlayer
}
function MLSPlayer:new()
    -- 关联对象和表
    local o = setmetatable({}, self.meta)

    -- 初始化房间信息
    o:constructor()
    return o
end

function MLSPlayer:constructor()
    -- 初始化玩家数据

    -- 脚本存档数据
    self.ms_data = {} 

    -- 玩家信息
    self.player_index = 0
    self.player_name = ""

    -- 玩家对局信息
    self.gold = 0

    -- 测试相关数据
    self.testapi_timer = nil
    self.testapi_index = 0  -- 测试API的索引
    self.testapi_round = 0
    self.testapi_json = nil
    self.send_stat_msg = 0  -- 发送消息统计
end

function MLSPlayer:dump()
    LogInfo("mlsplayer player_index=%d name=%s", self.player_index, self.player_name)
end

-- 测试API:MsGetScriptArchive
-- 玩家数据初始化
function MLSPlayer:init()
    
    -- 读取玩家脚本存档数据
    local start_t = os.clock()

    local hvalue = MsGetScriptArchive(self.player_index)
    LogDebug("player:%s load  msdata=%s", self.player_name, safeString(hvalue))
    if hvalue ~= nil and hvalue ~= '' then
        self.ms_data = json.decode(hvalue)
    end

    local cost_t = os.clock() - start_t
    LogInfo("player:%s:msload  cost_t=%0.3f", self.player_name, cost_t)
end

function MLSPlayer:get_msdata(mkey)
    return self.ms_data[mkey]
end

function MLSPlayer:set_msdata(mkey, mvalue)
    self.ms_data[mkey] = mvalue
end

-- 测试API:MsSaveScriptArchive
-- 保存脚本存档
function MLSPlayer:save_msdata()
    -- 读取玩家脚本存档数据
    local start_t = os.clock()

    local text = json.encode(self.ms_data)
    local result = MsSaveScriptArchive(self.player_index, text, string.len(text))
    LogDebug("player:%s save msdata=%s result=%s", self.player_name, text, result)

    local cost_t = os.clock() - start_t
    LogInfo("player:%s:mssave  cost_t=%0.3f", self.player_name, cost_t)
end

-- 玩家离开游戏
function MLSPlayer:exit_game(reason)
    local game_end_t = os.date("%Y-%m-%d %H:%M:%S")
    -- 保存玩家最后一次退出时间
    self:set_msdata("last_logout_t", game_end_t)
    local play_game_count = self:get_msdata("play_game_count")
    if play_game_count == nil then
        self:set_msdata("play_game_count", 1)
    else
        self:set_msdata("play_game_count", play_game_count + 1)
    end
    -- 游戏结束前，需要调用保存脚本存档
    self:save_msdata()
    LogInfo("player:%d:%s exit game:%s ",  self.player_index,self.player_name, reason)
end