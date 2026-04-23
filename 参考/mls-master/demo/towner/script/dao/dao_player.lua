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
    -- 初始化玩家信息
    self.player_index = 0
    self.player_name = ""
    self.gold = 0
    self.atk = 20
    self.max_hp = 100
    self.boss_damage = 0
    self.max_atk = 0
    self.is_cheat = false
    self.ms_data = {} -- 脚本存档数据
    self.tower_list = {} -- 玩家购买的塔ID
end

function MLSPlayer:dump()
    LogInfo("mlsplayer player_index=%d name=%s", self.player_index, self.player_name)
end

function MLSPlayer:init()
    local hvalue = MsGetScriptArchive(self.player_index)
    LogDebug("player:%s load  msdata=%s", self.player_name, safeString(hvalue))
    if hvalue ~= nil and hvalue ~= '' then
        self.ms_data = json.decode(hvalue)
    end
end

function MLSPlayer:get_msdata(mkey)
    return self.ms_data[mkey]
end

function MLSPlayer:set_msdata(mkey, mvalue)
    self.ms_data[mkey] = mvalue
end

function MLSPlayer:save_msdata(mkey, mvalue)
    local text = json.encode(self.ms_data)
    local result = MsSaveScriptArchive(self.player_index, text, string.len(text))
    LogDebug("player:%s save msdata=%s", self.player_name, text)
end
