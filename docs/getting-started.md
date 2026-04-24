# 快速开始

## 前置要求

- Rust 工具链（`rustup` + `cargo`），推荐 stable 最新版
- Windows 10 x86_64

## 编译

```powershell
cd mls-sim-rs
cargo build --release
```

编译产物：`target/release/mls-sim.exe`（约 5MB），无外部 DLL 依赖。

## 启动模拟器

### 方式一：命令行指定脚本目录

```powershell
./target/release/mls-sim.exe --script-dir "D:/code2/mls/参考/mls-master/demo/apidemo/script"
```

启动后自动创建一个房间并加载 `main.lua`，浏览器自动打开仪表盘。

### 方式二：使用配置文件

将 `config.example.json` 复制为 `config.json`，修改 `auto_room.script_dir` 为你的脚本目录：

```powershell
cp config.example.json config.json
./target/release/mls-sim.exe
```

### 方式三：Web 面板手动创建

不指定 `--script-dir`，直接启动：

```powershell
./target/release/mls-sim.exe
```

打开 `http://127.0.0.1:5000`，点击 **+ New** 按钮手动创建房间。

## 验证

启动后访问健康检查接口：

```powershell
curl http://127.0.0.1:5000/api/health
```

返回示例：

```json
{
  "ok": true,
  "name": "mls-sim",
  "version": "0.3.0",
  "room_count": 1,
  "rooms": [...]
}
```

## 发送测试事件

```powershell
curl -X POST http://127.0.0.1:5000/api/rooms/room-001/events `
  -H "Content-Type: application/json" `
  -d '{"ename":"testapi","evalue":"","player_index":0}'
```

## Bridge 轮询

模拟客户端轮询云脚本返回的事件：

```powershell
curl http://127.0.0.1:5000/api/bridge/poll/room-001/0
```
