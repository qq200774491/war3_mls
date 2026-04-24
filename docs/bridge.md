# Bridge 客户端集成

Bridge 用于让 War3 客户端在本地测试时直接连接 `mls-sim`，替代平台云脚本通道。

## 文件

Bridge 文件位于 `mls-sim/client/`：

- `mls_bridge.lua`
- `mls_bridge_config.lua`
- `mls_server_sim.lua`

可以通过 VSCode 命令 `MLS: Install Bridge Files` 复制到地图脚本目录。

## 配置

插件或接口会生成类似配置：

```lua
return {
    base_url = "http://127.0.0.1:5000",
    room_id = "room-001",
    player_index = 0,
    poll_interval = 0.05,
    req_sign_enable = false,
}
```

## 通信流程

1. War3 客户端调用 Bridge 登录房间。
2. 客户端通过 `/api/bridge/event` 发送事件给云脚本。
3. 客户端通过 `/api/bridge/poll/<room_id>/<player_index>` 轮询云脚本发出的事件。
4. 模拟器房间内的 Lua 脚本通过 `MsSendMlEvent` 将事件写入玩家队列。

## 注意

Bridge 是本地测试适配层，不应直接进入正式平台发布包。
