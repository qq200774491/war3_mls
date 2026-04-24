pub const JSON_LUA: &str = r#"
json = {}
local function esc(s)
  s = tostring(s)
  s = s:gsub('\\', '\\\\'):gsub('"', '\\"'):gsub('\n', '\\n'):gsub('\r', '\\r'):gsub('\t', '\\t')
  return '"' .. s .. '"'
end
local function is_array(t)
  local i = 0
  for _ in pairs(t) do
    i = i + 1
    if t[i] == nil then return false end
  end
  return true
end
local function enc(v)
  local tv = type(v)
  if tv == 'nil' then return 'null' end
  if tv == 'boolean' then return v and 'true' or 'false' end
  if tv == 'number' then
    if v ~= v then return 'null' end
    if v >= math.huge then return '1e9999' end
    if v <= -math.huge then return '-1e9999' end
    if v == math.floor(v) and math.abs(v) < 1e15 then
      return string.format('%d', v)
    end
    return string.format('%.14g', v)
  end
  if tv == 'string' then return esc(v) end
  if tv == 'table' then
    if is_array(v) then
      local out = {}
      for i = 1, #v do out[#out+1] = enc(v[i]) end
      return '[' .. table.concat(out, ',') .. ']'
    end
    local out = {}
    for k, val in pairs(v) do
      local key = type(k) == 'number' and string.format('%d', k) or tostring(k)
      out[#out+1] = esc(key) .. ':' .. enc(val)
    end
    return '{' .. table.concat(out, ',') .. '}'
  end
  return 'null'
end
function json.encode(v) return enc(v) end

local function skip(s, p)
  while p <= #s and s:sub(p,p):match('%s') do p = p + 1 end
  return p
end
local parse
local function parse_string(s, p)
  p = p + 1
  local out = {}
  while p <= #s do
    local c = s:sub(p,p)
    if c == '"' then return table.concat(out), p + 1 end
    if c == '\\' then
      p = p + 1; local e = s:sub(p,p)
      if e == 'n' then out[#out+1] = '\n'
      elseif e == 'r' then out[#out+1] = '\r'
      elseif e == 't' then out[#out+1] = '\t'
      elseif e == 'u' then
        local hex = s:sub(p+1, p+4)
        local code = tonumber(hex, 16)
        if code then
          if code < 128 then out[#out+1] = string.char(code)
          elseif code < 2048 then
            out[#out+1] = string.char(192 + math.floor(code/64), 128 + code%64)
          else
            out[#out+1] = string.char(224 + math.floor(code/4096), 128 + math.floor(code%4096/64), 128 + code%64)
          end
        end
        p = p + 4
      else out[#out+1] = e end
    else out[#out+1] = c end
    p = p + 1
  end
  error('unterminated string')
end
local function parse_number(s, p)
  local b = p
  while p <= #s and s:sub(p,p):match('[%d%+%-%e%E%.]') do p = p + 1 end
  return tonumber(s:sub(b, p-1)), p
end
local function parse_array(s, p)
  local a = {}
  p = skip(s, p + 1)
  if s:sub(p,p) == ']' then return a, p + 1 end
  while true do
    local v; v, p = parse(s, p); a[#a+1] = v; p = skip(s, p)
    local c = s:sub(p,p)
    if c == ']' then return a, p + 1 end
    if c ~= ',' then error('expected , or ]') end
    p = skip(s, p + 1)
  end
end
local function parse_object(s, p)
  local o = {}
  p = skip(s, p + 1)
  if s:sub(p,p) == '}' then return o, p + 1 end
  while true do
    local k; k, p = parse_string(s, p); p = skip(s, p)
    if s:sub(p,p) ~= ':' then error('expected :') end
    local v; v, p = parse(s, skip(s, p + 1)); o[k] = v; p = skip(s, p)
    local c = s:sub(p,p)
    if c == '}' then return o, p + 1 end
    if c ~= ',' then error('expected , or }') end
    p = skip(s, p + 1)
  end
end
parse = function(s, p)
  p = skip(s, p)
  local c = s:sub(p,p)
  if c == '"' then return parse_string(s, p) end
  if c == '{' then return parse_object(s, p) end
  if c == '[' then return parse_array(s, p) end
  if c == 't' then return true, p + 4 end
  if c == 'f' then return false, p + 5 end
  if c == 'n' then return nil, p + 4 end
  return parse_number(s, p)
end
function json.decode(s)
  if s == nil or s == '' then return nil end
  local v = parse(s, 1)
  return v
end
"#;
