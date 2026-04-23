LogInfo = Log.Info
LogError = Log.Error
LogDebug = Log.Debug

-- 发送一个mslog 时间到 war3客户端
-- 便于测试，输入一些信息到war3客户端中
-- 地图正式上限后，需要注释掉
function LogWar3(player_index, format, ...)
    local logMessage = string.format(format, ...)
    MsSendMlEvent(player_index, "mslog", logMessage)
end

-- 发送一个mslog 时间到 war3客户端
-- 展示消息给玩家
function ChatWar3(player_index, format, ...)
    local logMessage = string.format(format, ...)
    MsSendMlEvent(player_index, "mslog", logMessage)
end

function safeString(value)
    return value ~= nil and tostring(value) or "nil"
end

-- 发送一个事件到客户端, evalue 进行json序列化
-- text 长度不能超过1000个字符，否则将会被底层丢弃消息
function MsSendMlEventJson(player_index, eventName, evalue)
    local text = json.encode(evalue)
    LogDebug("同步脚本数据: player:%s event=%s evalue=%s", player_index, eventName, text)
    MsSendMlEvent(player_index, eventName, json.encode(evalue))
end

-- 发送一个事件到客户端, evalue 为原始的字符串
-- evalue 长度不能超过1000个字符，否则将会被底层丢弃消息
function MsSendMlEventRaw(player_index, eventName, evalue)
    LogDebug("同步脚本数据: player:%s event=%s evalue=%s", player_index, eventName, evalue)
    MsSendMlEvent(player_index, eventName, evalue)
end
