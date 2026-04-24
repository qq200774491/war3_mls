# VSCode 插件

插件目录位于 `vscode-extension/`，用于把模拟器、War3 多开、Bridge 配置和文档入口统一到 VSCode。

## 命令

| 命令 | 作用 |
| --- | --- |
| `MLS: Start Simulator` | 启动 `mls-sim/app.py` |
| `MLS: Stop Simulator` | 停止由插件启动的模拟器进程 |
| `MLS: Create Room` | 输入脚本目录和玩家数，创建云脚本房间 |
| `MLS: Launch War3 Multi Instance` | 按配置启动多个 War3 实例 |
| `MLS: Open Dashboard` | 打开模拟器 Web 面板 |
| `MLS: Send Event` | 向当前房间发送测试事件 |
| `MLS: Install Bridge Files` | 复制 Bridge Lua 文件到地图脚本目录 |
| `MLS: Generate Bridge Config` | 生成本地测试 Bridge 配置 |
| `MLS: Open Docs` | 打开文档入口 |

## 配置项

| 配置 | 默认值 | 说明 |
| --- | --- | --- |
| `mls.simulator.pythonPath` | `python` | Python 可执行文件 |
| `mls.simulator.host` | `127.0.0.1` | 模拟器绑定地址 |
| `mls.simulator.port` | `5000` | 模拟器端口 |
| `mls.script.defaultDir` | 空 | 默认云脚本目录 |
| `mls.war3.exePath` | 空 | War3 可执行文件 |
| `mls.war3.mapPath` | 空 | 测试地图路径 |
| `mls.war3.defaultPlayers` | `2` | 默认多开实例数 |
| `mls.war3.launchArgsTemplate` | `["-loadfile", "${mapPath}"]` | War3 启动参数模板 |

## 安全边界

插件只管理自己启动的模拟器和 War3 进程。默认绑定 `127.0.0.1`，如果改成 `0.0.0.0`，请只在可信局域网使用。
