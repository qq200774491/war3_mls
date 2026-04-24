# FAQ

## 编译失败

确认 Rust 工具链已安装：

```powershell
rustup --version
cargo --version
```

推荐使用 stable 最新版。如果 `mlua` 编译报错，检查是否安装了 MSVC C++ 构建工具（`rustup` 通常自动安装）。

## 启动后浏览器没有自动打开

在配置文件中设置 `"auto_open_browser": true`，或手动访问：

```text
http://127.0.0.1:5000
```

## 端口被占用

更换端口：

```powershell
mls-sim.exe --port 5001
```

## 云脚本没有加载

检查脚本目录下是否存在 `main.lua`。模拟器加载的入口文件固定为 `main.lua`。

## WebSocket 显示 Disconnected

Web 仪表盘使用原生 WebSocket（非 Socket.IO）。确认模拟器正在运行且端口正确。WebSocket 会自动重连。

## 中文路径

Rust 原生支持 Unicode 路径，中文路径可以直接使用，无需特殊处理。

## Bridge 轮询没有返回事件

1. 确认房间状态为 `running`：`curl http://127.0.0.1:5000/api/health`
2. 确认 `player_index` 正确。
3. 轮询会清空队列，重复轮询空队列会返回空数组。
4. 先发送一个事件触发脚本响应。

## 存档没有保存

存档在房间停止时自动保存。调用 `POST /api/rooms/{room_id}/stop` 停止房间后，存档写入 `archives/` 目录。

## 日志被熔断

模拟器限制 100 秒内最多 1000 条日志。减少日志频率，或仅在关键位置打印。
