# War3 MLS 工具集

War3 地图逻辑脚本（MLS）本地开发工具，包含云脚本模拟器、VSCode 插件和文档导航系统。

## 目录

| 目录 | 说明 |
| --- | --- |
| `mls-sim/` | MLS 本地云脚本模拟器，支持多房间、玩家、事件、存档和 Bridge API |
| `vscode-extension/` | VSCode 插件，用于启动模拟器、创建房间、发送事件和 War3 多开 |
| `docs/` | MkDocs 文档内容 |
| `参考/mls-master/` | MLS 官方参考资料、API 文档和 demo 脚本 |

## 快速开始

```powershell
cd mls-sim
pip install -r requirements.txt
python app.py 5000 127.0.0.1
```

打开 Web 面板：

```text
http://127.0.0.1:5000
```

## VSCode 插件

```powershell
cd vscode-extension
npm install
npm run compile
```

在 VSCode Extension Host 中执行：

- `MLS: Start Simulator`
- `MLS: Create Room`
- `MLS: Launch War3 Multi Instance`
- `MLS: Open Dashboard`

## 文档站

```powershell
pip install mkdocs mkdocs-material
mkdocs serve
```

文档入口：`docs/index.md`。
