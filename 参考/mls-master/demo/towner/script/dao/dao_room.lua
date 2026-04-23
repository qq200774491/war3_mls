-- 定义一个房间
MLSRoom = {}
MLSRoom.meta = {
    __index = MLSRoom
}

function MLSRoom:new()
    -- 关联对象和表
    local o = setmetatable({}, self.meta)

    -- 初始化房间信息
    o:constructor()
    return o
end

function MLSRoom:constructor()
    self.buy_tower_count = 0
    self.room_players = {}
end

function MLSRoom:dump()

end

-- 访问房间中的变量数据
g_room = MLSRoom:new()

-- 访问房间中对局玩家的数据
-- 下标为0...n 的玩家数据
g_room_players = g_room.room_players

