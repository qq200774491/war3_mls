# 快速开始

## 安装模拟器依赖

```powershell
cd mls-sim
pip install -r requirements.txt
```

## 启动本地云脚本服务器

```powershell
python app.py 5000 127.0.0.1
```

启动后访问：

```text
http://127.0.0.1:5000
```

## 创建测试房间

使用 Web 面板或 REST API 创建房间：

```powershell
curl -X POST http://127.0.0.1:5000/api/rooms `
  -H "Content-Type: application/json" `
  -d "{\"script_dir\":\"D:/code2/mls/参考/mls-master/demo/apidemo/script\",\"players\":[{\"index\":0,\"name\":\"Player_0\"},{\"index\":1,\"name\":\"Player_1\"}]}"
```

## 使用 VSCode 插件

```powershell
cd vscode-extension
npm install
npm run compile
```

然后在 VSCode 中按 `F5` 启动 Extension Host，执行 `MLS: Start Simulator` 与 `MLS: Create Room`。
