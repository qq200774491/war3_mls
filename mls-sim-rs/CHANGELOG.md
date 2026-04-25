# Changelog

## v0.4.6 (2026-04-25)

### 新功能
- 新增本地调试控制接口 `/api/debug/*`（由 Syh1906 贡献，PR #2）
  - `GET /api/debug/rooms/{room_id}/logs` — 查看房间日志
  - `POST /api/debug/rooms/{room_id}/logs/clear` — 清除房间日志
  - `POST /api/debug/rooms/{room_id}/restart` — 重启房间
  - `POST /api/debug/restart` — 服务重启（预留）
- 提取 `build_bridge_router()` 为可复用的路由构建函数
- 新增 `lib.rs` 库入口，支持集成测试
- 新增 debug API 集成测试 (`tests/debug_api.rs`)

## v0.4.5 (2026-04-25)

### 修复
- 修复 json.encode 分页传输中文键名损坏的问题
  - send_paged 按字节切割 JSON 字符串时可能断在 UTF-8 多字节字符中间，v0.4.4 的 serde_json 编码器会将不完整 UTF-8 字节转为 Latin-1 字符（单字节 0xE6 → 双字节 C3 A6），导致客户端拼接后 json.decode 失败或键名对不上
  - json.encode 改为直接构建原始字节输出（Vec\<u8\>），字符串逐字节透传只做 JSON 必要转义，与 cjson/KK 平台行为一致
  - json.decode 保持 serde_json 不变

## v0.4.4 (2026-04-25)

### 改进
- 重写 json.encode / json.decode：从 Lua 手写实现替换为 Rust (serde_json) 实现，对齐 cjson 标准
  - 空 Lua 表 `{}` 序列化为 JSON object `{}`（与 KK 对战平台一致），不再错误地序列化为 `[]`
  - JSON 合规性由 serde_json 保证（Unicode 转义、数字精度、嵌套结构等）
  - 非 UTF-8 Lua 字符串安全降级处理

## v0.4.3 (2026-04-25)

### 新功能
- 新增 `--console-notwrte` 启动参数，可隐藏 Windows 控制台窗口，仅保留 GUI 界面

### 修复
- Lua 脚本中使用 `print()` 现在会输出到 GUI 控制台（INF 级别），不再只写 stdout

## v0.4.2

### 修复
- 修复非 UTF-8 字节转义格式导致客户端 JSON 解析失败的问题

## v0.4.1

### 修复
- 修复存档保存问题——立即落盘、优雅关闭、退出前持久化
- lua_value_to_string 逐字节解析 UTF-8，分页切断处不再整串转义
- lua_value_to_string() 改为先尝试 to_str() 处理合法 UTF-8，非 UTF-8 时用无损转义
- esc() 函数补充转义 JSON 规范不允许的控制字符，产生合规 JSON 输出
