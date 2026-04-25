# Changelog

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
