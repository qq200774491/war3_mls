# War3 多开测试

第一版多开由 VSCode 插件负责进程启动和玩家槽位分配。

## 配置

在 VSCode 设置中配置：

```json
{
  "mls.war3.exePath": "D:/Games/Warcraft III/Warcraft III.exe",
  "mls.war3.mapPath": "D:/maps/test.w3x",
  "mls.war3.defaultPlayers": 2,
  "mls.war3.launchArgsTemplate": ["-loadfile", "${mapPath}"]
}
```

`launchArgsTemplate` 支持：

- `${mapPath}`：地图路径。
- `${roomId}`：当前 MLS 房间 ID。
- `${playerIndex}`：当前 War3 实例对应的玩家槽位。

## 流程

1. 执行 `MLS: Start Simulator`。
2. 执行 `MLS: Create Room`，输入云脚本目录和玩家数。
3. 执行 `MLS: Install Bridge Files`，复制 Bridge 文件到地图脚本目录。
4. 执行 `MLS: Launch War3 Multi Instance`。
5. 在模拟器面板或插件输出中查看日志和事件。

## 降级方案

如果当前 War3 版本或平台不支持直接参数启动，插件仍可生成 Bridge 配置和房间信息，War3 实例可手动打开。
