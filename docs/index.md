# War3 MLS 云脚本模拟器

Rust 实现的 MLS 云脚本本地测试环境，单二进制部署，内置 Web 仪表盘。

## 推荐阅读路径

1. [快速开始](getting-started.md)：编译并启动模拟器。
2. [模拟器说明](simulator.md)：架构、Lua API 注入和配置。
3. [REST API](rest-api.md)：HTTP 接口参考。
4. [Bridge 客户端集成](bridge.md)：让 War3 客户端连接本地模拟器。
5. [MLS API 参考](mls-api.md)：Lua 脚本可用的全部 API。
6. [FAQ](faq.md)：常见问题。

## 技术栈

| 组件 | 技术 |
| --- | --- |
| 语言 | Rust |
| HTTP 框架 | axum + tokio |
| Lua 运行时 | mlua (Lua 5.4, vendored) |
| WebSocket | axum 内置 |
| 前端 | HTML/JS/CSS (rust-embed 编译嵌入) |
| 配置 | JSON |
