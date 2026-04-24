# MLS API 类型提示

`mls-sim/types/mls_api.lua` 提供 LuaLS 类型提示。

## VSCode 设置

```json
{
  "Lua.workspace.library": [
    "${workspaceFolder}/mls-sim/types"
  ],
  "Lua.runtime.version": "Lua 5.3"
}
```

## 覆盖范围

- `Log.Debug`、`Log.Info`、`Log.Error`
- `Timer.After`、`Timer.Loop`
- `RegisterEvent`、`UnregisterEvent`
- `MsSendMlEvent`、`MsEnd`
- 玩家查询 API
- 房间查询 API
- 道具 API
- 存档 API
- `json.encode`、`json.decode`

官方 API 参考见 `参考/mls-master/API.md`。
