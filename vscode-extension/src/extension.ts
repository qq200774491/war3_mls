import * as fs from "fs";
import * as http from "http";
import * as path from "path";
import { spawn, ChildProcess } from "child_process";
import * as vscode from "vscode";

type Room = {
  id: string;
  status: string;
  script_dir: string;
  players?: Record<string, unknown>;
};

let simulatorProcess: ChildProcess | undefined;
const war3Processes: ChildProcess[] = [];
let currentRoomId = "";
let output: vscode.OutputChannel;
let statusBar: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
  output = vscode.window.createOutputChannel("War3 MLS");
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.command = "mls.openDashboard";
  statusBar.text = "$(cloud) MLS: idle";
  statusBar.tooltip = "Open MLS simulator dashboard";
  statusBar.show();

  context.subscriptions.push(
    output,
    statusBar,
    vscode.commands.registerCommand("mls.startSimulator", startSimulator),
    vscode.commands.registerCommand("mls.stopSimulator", stopSimulator),
    vscode.commands.registerCommand("mls.createRoom", createRoom),
    vscode.commands.registerCommand("mls.launchWar3Multi", launchWar3Multi),
    vscode.commands.registerCommand("mls.openDashboard", openDashboard),
    vscode.commands.registerCommand("mls.sendEvent", sendEvent),
    vscode.commands.registerCommand("mls.openDocs", openDocs),
    vscode.commands.registerCommand("mls.installBridgeFiles", installBridgeFiles),
    vscode.commands.registerCommand("mls.generateBridgeConfig", generateBridgeConfig)
  );

  void refreshStatus();
}

export function deactivate() {
  stopChild(simulatorProcess);
  for (const child of war3Processes.splice(0)) {
    stopChild(child);
  }
}

function config<T>(key: string, fallback: T): T {
  return vscode.workspace.getConfiguration("mls").get<T>(key, fallback);
}

function workspaceRoot(): string {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? path.resolve(__dirname, "..", "..");
}

function simulatorRoot(): string {
  return path.join(workspaceRoot(), "mls-sim");
}

function baseUrl(): string {
  return `http://${config("simulator.host", "127.0.0.1")}:${config("simulator.port", 5000)}`;
}

async function startSimulator() {
  if (simulatorProcess && !simulatorProcess.killed) {
    vscode.window.showInformationMessage("MLS simulator is already running.");
    return;
  }

  const pythonPath = config("simulator.pythonPath", "python");
  const port = String(config("simulator.port", 5000));
  const host = config("simulator.host", "127.0.0.1");
  const appPath = path.join(simulatorRoot(), "app.py");
  if (!fs.existsSync(appPath)) {
    vscode.window.showErrorMessage(`Cannot find simulator: ${appPath}`);
    return;
  }

  output.show(true);
  output.appendLine(`Starting mls-sim: ${pythonPath} ${appPath} ${port} ${host}`);
  simulatorProcess = spawn(pythonPath, [appPath, port, host], {
    cwd: simulatorRoot(),
    shell: false
  });
  simulatorProcess.stdout?.on("data", (chunk) => output.append(chunk.toString()));
  simulatorProcess.stderr?.on("data", (chunk) => output.append(chunk.toString()));
  simulatorProcess.on("exit", (code) => {
    output.appendLine(`mls-sim exited with code ${code ?? "unknown"}`);
    simulatorProcess = undefined;
    void refreshStatus();
  });

  await waitForHealth();
  await refreshStatus();
}

async function stopSimulator() {
  stopChild(simulatorProcess);
  simulatorProcess = undefined;
  statusBar.text = "$(cloud) MLS: stopped";
  vscode.window.showInformationMessage("MLS simulator stopped.");
}

async function createRoom() {
  await ensureSimulator();
  const defaultDir = config("script.defaultDir", "");
  const scriptDir = await vscode.window.showInputBox({
    title: "MLS script directory",
    value: defaultDir || path.join(workspaceRoot(), "参考", "mls-master", "demo", "apidemo", "script"),
    prompt: "Directory containing main.lua"
  });
  if (!scriptDir) return;

  const playerCountText = await vscode.window.showInputBox({
    title: "Player count",
    value: String(config("war3.defaultPlayers", 2)),
    validateInput: (value) => Number.isInteger(Number(value)) && Number(value) > 0 ? undefined : "Enter a positive integer"
  });
  if (!playerCountText) return;
  const playerCount = Number(playerCountText);
  const players = Array.from({ length: playerCount }, (_, index) => ({ index, name: `Player_${index}` }));

  const room = await requestJson<Room>("POST", "/api/rooms", {
    script_dir: scriptDir,
    mode_id: 0,
    players,
    auto_start: true
  });
  currentRoomId = room.id;
  output.appendLine(`Created room ${room.id} for ${scriptDir}`);
  vscode.window.showInformationMessage(`MLS room created: ${room.id}`);
  await refreshStatus();
}

async function launchWar3Multi() {
  if (!currentRoomId) {
    await createRoom();
  }
  if (!currentRoomId) return;

  const exePath = config("war3.exePath", "");
  const mapPath = config("war3.mapPath", "");
  if (!exePath || !fs.existsSync(exePath)) {
    vscode.window.showErrorMessage("Configure mls.war3.exePath before launching War3.");
    return;
  }
  if (!mapPath || !fs.existsSync(mapPath)) {
    vscode.window.showErrorMessage("Configure mls.war3.mapPath before launching War3.");
    return;
  }

  const count = config("war3.defaultPlayers", 2);
  const template = config<string[]>("war3.launchArgsTemplate", ["-loadfile", "${mapPath}"]);
  for (let playerIndex = 0; playerIndex < count; playerIndex++) {
    await writeBridgeConfig(playerIndex);
    const args = template.map((arg) => arg
      .replace(/\$\{mapPath\}/g, mapPath)
      .replace(/\$\{playerIndex\}/g, String(playerIndex))
      .replace(/\$\{roomId\}/g, currentRoomId));
    const child = spawn(exePath, args, { cwd: path.dirname(exePath), detached: false, shell: false });
    war3Processes.push(child);
    child.on("exit", () => {
      const index = war3Processes.indexOf(child);
      if (index >= 0) war3Processes.splice(index, 1);
    });
    output.appendLine(`Launched War3 player ${playerIndex}: ${exePath} ${args.join(" ")}`);
  }
  vscode.window.showInformationMessage(`Launched ${count} War3 instance(s) for ${currentRoomId}.`);
}

async function openDashboard() {
  await ensureSimulator();
  await vscode.env.openExternal(vscode.Uri.parse(baseUrl()));
}

async function sendEvent() {
  if (!currentRoomId) {
    const rooms = await requestJson<Room[]>("GET", "/api/rooms");
    currentRoomId = rooms[0]?.id ?? "";
  }
  if (!currentRoomId) {
    vscode.window.showErrorMessage("Create a room before sending events.");
    return;
  }
  const ename = await vscode.window.showInputBox({ title: "Event name", prompt: "MLS event name" });
  if (!ename) return;
  const evalue = await vscode.window.showInputBox({ title: "Event data", value: "" });
  const playerIndex = Number(await vscode.window.showInputBox({ title: "Player index", value: "0" }) ?? "0");
  await requestJson("POST", `/api/rooms/${currentRoomId}/events`, { ename, evalue: evalue ?? "", player_index: playerIndex });
  vscode.window.showInformationMessage(`Sent ${ename} to ${currentRoomId}.`);
}

async function openDocs() {
  const indexPath = path.join(workspaceRoot(), "docs", "index.md");
  if (fs.existsSync(indexPath)) {
    await vscode.window.showTextDocument(vscode.Uri.file(indexPath));
    return;
  }
  await vscode.env.openExternal(vscode.Uri.parse("https://www.mkdocs.org/"));
}

async function installBridgeFiles() {
  const targetDir = await vscode.window.showInputBox({
    title: "Target Lua directory",
    prompt: "Directory where Bridge Lua files should be copied"
  });
  if (!targetDir) return;
  fs.mkdirSync(targetDir, { recursive: true });
  const sourceDir = path.join(simulatorRoot(), "client");
  for (const fileName of ["mls_bridge.lua", "mls_bridge_config.lua", "mls_server_sim.lua"]) {
    fs.copyFileSync(path.join(sourceDir, fileName), path.join(targetDir, fileName));
  }
  vscode.window.showInformationMessage(`Bridge files installed to ${targetDir}.`);
}

async function generateBridgeConfig() {
  if (!currentRoomId) {
    const roomId = await vscode.window.showInputBox({ title: "Room ID", value: currentRoomId });
    if (!roomId) return;
    currentRoomId = roomId;
  }
  const playerIndex = Number(await vscode.window.showInputBox({ title: "Player index", value: "0" }) ?? "0");
  const targetPath = await vscode.window.showInputBox({
    title: "Bridge config path",
    value: path.join(workspaceRoot(), "mls_bridge_config.lua")
  });
  if (!targetPath) return;
  const content = await bridgeConfigContent(playerIndex);
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  fs.writeFileSync(targetPath, content, "utf8");
  vscode.window.showInformationMessage(`Bridge config written: ${targetPath}`);
}

async function writeBridgeConfig(playerIndex: number) {
  const mapDir = path.dirname(config("war3.mapPath", workspaceRoot()));
  const target = path.join(mapDir, "mls_bridge_config.lua");
  fs.writeFileSync(target, await bridgeConfigContent(playerIndex), "utf8");
}

async function bridgeConfigContent(playerIndex: number): Promise<string> {
  const result = await requestJson<{ content: string }>("POST", "/api/bridge/config", {
    room_id: currentRoomId,
    player_index: playerIndex,
    base_url: baseUrl()
  });
  return result.content;
}

async function ensureSimulator() {
  try {
    await requestJson("GET", "/api/health");
  } catch {
    await startSimulator();
  }
}

async function refreshStatus() {
  try {
    const health = await requestJson<{ room_count: number; rooms: Room[] }>("GET", "/api/health");
    currentRoomId = currentRoomId || health.rooms[0]?.id || "";
    statusBar.text = `$(cloud) MLS: ${health.room_count} room(s)`;
  } catch {
    statusBar.text = "$(cloud) MLS: offline";
  }
}

async function waitForHealth() {
  const started = Date.now();
  while (Date.now() - started < 8000) {
    try {
      await requestJson("GET", "/api/health");
      return;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 300));
    }
  }
  vscode.window.showWarningMessage("MLS simulator did not respond to /api/health yet.");
}

function requestJson<T = unknown>(method: string, route: string, body?: unknown): Promise<T> {
  const url = new URL(route, baseUrl());
  const payload = body === undefined ? undefined : JSON.stringify(body);
  return new Promise<T>((resolve, reject) => {
    const req = http.request(url, {
      method,
      headers: payload ? {
        "Content-Type": "application/json",
        "Content-Length": Buffer.byteLength(payload)
      } : undefined
    }, (res) => {
      let data = "";
      res.setEncoding("utf8");
      res.on("data", (chunk) => data += chunk);
      res.on("end", () => {
        try {
          const parsed = data ? JSON.parse(data) : {};
          if ((res.statusCode ?? 500) >= 400) {
            reject(new Error(parsed.error ?? `HTTP ${res.statusCode}`));
            return;
          }
          resolve(parsed as T);
        } catch (error) {
          reject(error);
        }
      });
    });
    req.on("error", reject);
    if (payload) req.write(payload);
    req.end();
  });
}

function stopChild(child: ChildProcess | undefined) {
  if (!child || child.killed) return;
  if (process.platform === "win32" && child.pid) {
    spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"]);
    return;
  }
  child.kill();
}
