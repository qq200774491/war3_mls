# War3 MLS Tools

这个项目把 War3 MLS 云脚本本地测试流程整合为三部分：

- `mls-sim`：本地云脚本服务器，模拟平台 MLS 运行时。
- `vscode-extension`：VSCode 插件入口，管理模拟器、房间、事件和 War3 多开。
- `docs`：MkDocs 文档导航系统。

## 推荐阅读路径

1. [快速开始](getting-started.md)：安装依赖并启动模拟器。
2. [VSCode 插件](vscode-extension.md)：在编辑器内管理本地测试环境。
3. [War3 多开测试](war3-multirun.md)：配置 War3 路径、地图路径和多实例参数。
4. [Bridge 客户端集成](bridge.md)：让地图客户端连接本地云脚本服务器。
5. [REST API](rest-api.md)：插件和外部工具可调用的 HTTP 接口。

## 当前状态

本地模拟器已有房间、玩家、事件、存档、Bridge API 和 Web 面板。插件与文档站为本次新增的一体化入口。
