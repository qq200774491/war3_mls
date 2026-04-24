# 云脚本模拟器

`mls-sim` 是 Python/Flask + Socket.IO + Lupa 实现的本地 MLS 云脚本运行时。

## 核心能力

- 多房间同时运行，每个房间拥有独立 Lua VM。
- 加载指定脚本目录下的 `main.lua`。
- 注入 MLS 全局 API：日志、定时器、事件、房间查询、玩家查询、道具、存档。
- Web 面板管理房间、玩家、事件和日志。
- REST API 供插件和自动化脚本调用。
- Bridge API 供 War3 客户端轮询和发送事件。

## 启动参数

```powershell
python app.py <port> <host>
```

示例：

```powershell
python app.py 5000 127.0.0.1
```

## 新增插件接口

- `GET /api/health`：用于插件探测服务状态。
- `POST /api/profiles`：保存测试 profile。
- `GET /api/profiles`：列出测试 profile。
- `POST /api/bridge/config`：生成 Bridge 配置内容。

详细接口见 [REST API](rest-api.md)。
