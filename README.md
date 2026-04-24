# War3 MLS 云脚本模拟器

War3 地图逻辑脚本（MLS）本地测试环境，Rust 单二进制实现。

## 目录

| 目录 | 说明 |
| --- | --- |
| `mls-sim-rs/` | Rust 实现的 MLS 本地云脚本模拟器（axum + mlua） |
| `docs/` | 使用文档 |
| `参考/mls-master/` | MLS 官方参考资料、API 文档和 demo 脚本 |

## 快速开始

### 编译

```powershell
cd mls-sim-rs
cargo build --release
```

输出文件：`target/release/mls-sim.exe`（约 5MB）。

### 运行

```powershell
# 指定云脚本目录自动创建房间
mls-sim.exe --script-dir "D:/code2/mls/参考/mls-master/demo/apidemo/script"

# 或使用配置文件
mls-sim.exe --config config.json
```

启动后自动打开浏览器，访问 Web 仪表盘：

```text
http://127.0.0.1:5000
```

### 命令行参数

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--host` | `127.0.0.1` | 监听地址 |
| `--port` / `-p` | `5000` | 监听端口 |
| `--script-dir` / `-s` | - | 云脚本目录，启动后自动创建房间 |
| `--config` | `config.json` | 配置文件路径 |

## 配置文件

`config.json` 示例：

```json
{
  "host": "127.0.0.1",
  "port": 5000,
  "auto_open_browser": true,
  "archive_dir": "./archives",
  "auto_room": {
    "script_dir": "D:/code2/mls/参考/mls-master/demo/apidemo/script",
    "mode_id": 0,
    "players": [
      {"index": 0, "name": "Alice", "items": {"VIP001": 1}},
      {"index": 1, "name": "Bob"}
    ]
  }
}
```

## 文档

详细使用说明见 `docs/` 目录，入口：`docs/index.md`。
