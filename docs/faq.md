# FAQ

## 模拟器启动失败

先确认 Python 版本和依赖：

```powershell
python --version
pip install -r mls-sim/requirements.txt
```

## `lupa` 安装失败

`lupa` 依赖 Python 版本和平台 wheel。建议使用 Python 3.10+，并优先选择有预编译 wheel 的版本。

## 插件提示模拟器离线

确认 `mls.simulator.host`、`mls.simulator.port` 与实际启动参数一致，并访问：

```text
http://127.0.0.1:5000/api/health
```

## War3 没有自动进入地图

不同 War3 版本和平台启动参数可能不同。调整 `mls.war3.launchArgsTemplate`，或先使用手动打开 War3 的降级流程。

## 中文路径显示异常

项目文档和脚本按 UTF-8 保存。PowerShell 查看时建议显式使用 UTF-8：

```powershell
Get-Content -Encoding utf8 README.md
```
