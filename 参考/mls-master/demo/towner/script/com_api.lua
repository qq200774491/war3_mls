-- 分隔字符串
function split(str, split_char, split_count)
    if (str == nil) then
        return nil
    end
    local sub_str_tab = {}
    local count = 0
    while true do
        count = count + 1
        if (split_count ~= nil and count >= split_count) then
            table.insert(sub_str_tab, str)
            break
        end
        local pos = string.find(str, split_char)
        if not pos then
            table.insert(sub_str_tab, str)
            break
        end
        local sub_str = string.sub(str, 1, pos - 1)
        table.insert(sub_str_tab, sub_str)
        str = string.sub(str, pos + 1, string.len(str))
    end
    return sub_str_tab
end

