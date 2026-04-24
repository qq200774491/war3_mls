// MLS Simulator Frontend (Native WebSocket)

const API = '';
let ws = null;
let currentRoomId = null;
let logEntries = [];
let eventEntries = [];
let wsReconnectTimer = null;

// ---- WebSocket ----

function initWebSocket() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${proto}//${location.host}/ws`;
    ws = new WebSocket(url);

    ws.onopen = () => {
        updateWsStatus(true);
        if (currentRoomId) {
            ws.send(JSON.stringify({ type: 'join_room', room_id: currentRoomId }));
        }
    };

    ws.onmessage = (evt) => {
        try {
            const msg = JSON.parse(evt.data);
            if (msg.type === 'log' && msg.data && msg.data.room_id === currentRoomId) {
                appendLog(msg.data);
            } else if (msg.type === 'out_event' && msg.data && msg.data.room_id === currentRoomId) {
                appendOutEvent(msg.data);
            }
        } catch (e) {}
    };

    ws.onclose = () => {
        updateWsStatus(false);
        wsReconnectTimer = setTimeout(initWebSocket, 2000);
    };

    ws.onerror = () => {
        ws.close();
    };
}

function updateWsStatus(connected) {
    const el = document.getElementById('ws-status');
    if (connected) {
        el.textContent = 'Connected';
        el.className = 'ws-status connected';
    } else {
        el.textContent = 'Disconnected';
        el.className = 'ws-status disconnected';
    }
}

function joinRoom(roomId) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        if (currentRoomId) {
            ws.send(JSON.stringify({ type: 'leave_room', room_id: currentRoomId }));
        }
        ws.send(JSON.stringify({ type: 'join_room', room_id: roomId }));
    }
}

// ---- API Helpers ----

async function api(method, path, body) {
    const opts = { method, headers: { 'Content-Type': 'application/json' } };
    if (body) opts.body = JSON.stringify(body);
    const res = await fetch(API + path, opts);
    return res.json();
}

// ---- Room List ----

async function refreshRoomList() {
    const rooms = await api('GET', '/api/rooms');
    const list = document.getElementById('room-list');
    list.innerHTML = '';
    rooms.forEach(r => {
        const div = document.createElement('div');
        div.className = 'room-item' + (r.id === currentRoomId ? ' active' : '');
        div.onclick = () => selectRoom(r.id);
        div.innerHTML = `
            <div class="room-name">${r.id}</div>
            <div class="room-meta">${r.player_count}P | ${r.status} | Mode ${r.mode_id}</div>
        `;
        list.appendChild(div);
    });

    // Auto-select first room if none selected
    if (!currentRoomId && rooms.length > 0) {
        selectRoom(rooms[0].id);
    }
}

// ---- Select Room ----

async function selectRoom(roomId) {
    currentRoomId = roomId;
    logEntries = [];
    eventEntries = [];
    document.getElementById('log-panel').innerHTML = '';
    document.getElementById('event-panel').innerHTML = '';
    joinRoom(roomId);

    document.getElementById('no-room-selected').style.display = 'none';
    document.getElementById('room-detail').style.display = 'block';

    await refreshRoomDetail();
    refreshRoomList();
}

async function refreshRoomDetail() {
    if (!currentRoomId) return;
    const room = await api('GET', `/api/rooms/${currentRoomId}`);

    document.getElementById('room-title').textContent = room.id;
    const badge = document.getElementById('room-status');
    badge.textContent = room.status;
    badge.className = 'badge ' + room.status;

    const info = `Script: ${room.script_dir} | GameTime: ${room.game_time}s`;
    document.getElementById('room-info').textContent = info;

    renderPlayers(room.players);
    updatePlayerSelect(room.players);
}

// ---- Players ----

function renderPlayers(players) {
    const container = document.getElementById('player-list');
    container.innerHTML = '';
    for (const [idx, p] of Object.entries(players)) {
        const card = document.createElement('div');
        card.className = 'player-card';
        const statusClass = p.is_connected ? 'player-status' : 'player-status offline';
        const statusText = p.is_connected ? 'Online' : 'Offline';
        card.innerHTML = `
            <div class="player-name">[${p.index}] ${p.name}</div>
            <div class="${statusClass}">${statusText}</div>
            <div style="color:#666;font-size:11px;margin-top:2px">
                Lv.${p.map_level} | Items: ${Object.keys(p.items).length}
            </div>
            <div class="player-actions">
                <button class="btn btn-small" onclick="simLeave(${p.index})">Leave</button>
                <button class="btn btn-small" onclick="simJoin(${p.index})">Join</button>
                <button class="btn btn-small btn-danger" onclick="simExit(${p.index})">Exit</button>
            </div>
        `;
        container.appendChild(card);
    }
}

function updatePlayerSelect(players) {
    const sel = document.getElementById('event-player');
    sel.innerHTML = '';
    for (const [idx, p] of Object.entries(players)) {
        const opt = document.createElement('option');
        opt.value = p.index;
        opt.textContent = `[${p.index}] ${p.name}`;
        sel.appendChild(opt);
    }
    const optRoom = document.createElement('option');
    optRoom.value = -1;
    optRoom.textContent = '[-1] Room Event';
    sel.appendChild(optRoom);
}

// ---- Player Simulation ----

async function simLeave(idx) {
    await api('POST', `/api/rooms/${currentRoomId}/players/${idx}/leave`);
    setTimeout(refreshRoomDetail, 300);
}

async function simJoin(idx) {
    await api('POST', `/api/rooms/${currentRoomId}/players/${idx}/join`);
    setTimeout(refreshRoomDetail, 300);
}

async function simExit(idx) {
    await api('POST', `/api/rooms/${currentRoomId}/players/${idx}/exit`);
    setTimeout(refreshRoomDetail, 300);
}

// ---- Room Actions ----

async function startRoom() {
    await api('POST', `/api/rooms/${currentRoomId}/start`);
    setTimeout(refreshRoomDetail, 500);
    setTimeout(refreshRoomList, 500);
}

async function stopRoom() {
    await api('POST', `/api/rooms/${currentRoomId}/stop`);
    setTimeout(refreshRoomDetail, 500);
    setTimeout(refreshRoomList, 500);
}

async function destroyRoom() {
    if (!confirm('Destroy this room?')) return;
    await api('DELETE', `/api/rooms/${currentRoomId}`);
    currentRoomId = null;
    document.getElementById('room-detail').style.display = 'none';
    document.getElementById('no-room-selected').style.display = 'flex';
    refreshRoomList();
}

// ---- Create Room ----

function showCreateRoom() {
    document.getElementById('create-modal').style.display = 'flex';
}

function hideCreateRoom() {
    document.getElementById('create-modal').style.display = 'none';
}

async function createRoom() {
    const scriptDir = document.getElementById('new-script-dir').value.trim();
    const modeId = parseInt(document.getElementById('new-mode-id').value) || 0;
    const autoStart = document.getElementById('new-auto-start').checked;

    let players;
    try {
        players = JSON.parse(document.getElementById('new-players').value);
    } catch (e) {
        alert('Players JSON parse error: ' + e.message);
        return;
    }

    const result = await api('POST', '/api/rooms', {
        script_dir: scriptDir,
        mode_id: modeId,
        players: players,
        auto_start: autoStart
    });

    if (result.error) {
        alert('Error: ' + result.error);
        return;
    }

    hideCreateRoom();
    await refreshRoomList();
    selectRoom(result.id);
}

// ---- Send Event ----

async function sendEvent() {
    const ename = document.getElementById('event-name').value.trim();
    const evalue = document.getElementById('event-data').value;
    const playerIndex = parseInt(document.getElementById('event-player').value);

    if (!ename) { alert('Event name is required'); return; }

    await api('POST', `/api/rooms/${currentRoomId}/events`, {
        ename, evalue, player_index: playerIndex
    });
}

function addPreset(name, data) {
    const container = document.getElementById('event-presets');
    const btn = document.createElement('button');
    btn.className = 'preset-btn';
    btn.textContent = name;
    btn.onclick = () => {
        document.getElementById('event-name').value = name;
        document.getElementById('event-data').value = data || '';
    };
    container.appendChild(btn);
}

// ---- State Inspector ----

async function refreshState() {
    if (!currentRoomId) return;
    const state = await api('GET', `/api/rooms/${currentRoomId}/state`);
    document.getElementById('state-json').textContent = JSON.stringify(state, null, 2);
}

// ---- Logs ----

function appendLog(entry) {
    logEntries.push(entry);
    if (logEntries.length > 2000) logEntries.shift();

    const panel = document.getElementById('log-panel');
    const div = document.createElement('div');
    div.className = `log-line ${entry.level}`;

    const ts = new Date(entry.timestamp * 1000).toLocaleTimeString();
    const player = entry.player_index >= 0 ? `P${entry.player_index}` : '';
    div.textContent = `[${ts}][${entry.level}][${entry.source}]${player ? '[' + player + ']' : ''} ${entry.message}`;
    div.dataset.level = entry.level;
    div.dataset.text = entry.message.toLowerCase();

    applyFilter(div);
    panel.appendChild(div);

    if (document.getElementById('log-autoscroll').checked) {
        panel.scrollTop = panel.scrollHeight;
    }
}

function appendOutEvent(ev) {
    eventEntries.push(ev);
    if (eventEntries.length > 2000) eventEntries.shift();

    const panel = document.getElementById('event-panel');
    const div = document.createElement('div');
    div.className = 'log-line event';
    const ts = new Date(ev.timestamp * 1000).toLocaleTimeString();
    const target = ev.player_index === -1 ? 'ALL' : `P${ev.player_index}`;
    div.textContent = `[${ts}] -> ${target} | ${ev.ename}: ${ev.evalue}`;
    panel.appendChild(div);

    if (document.getElementById('log-autoscroll').checked) {
        panel.scrollTop = panel.scrollHeight;
    }
}

function filterLogs() {
    const level = document.getElementById('log-level-filter').value;
    const search = document.getElementById('log-search').value.toLowerCase();
    const panel = document.getElementById('log-panel');
    for (const line of panel.children) {
        applyFilter(line, level, search);
    }
}

function applyFilter(div, level, search) {
    level = level || document.getElementById('log-level-filter').value;
    search = search || document.getElementById('log-search').value.toLowerCase();
    let show = true;
    if (level !== 'all' && div.dataset.level !== level) show = false;
    if (search && div.dataset.text && !div.dataset.text.includes(search)) show = false;
    div.style.display = show ? '' : 'none';
}

function clearLogs() {
    document.getElementById('log-panel').innerHTML = '';
    document.getElementById('event-panel').innerHTML = '';
    logEntries = [];
    eventEntries = [];
}

// ---- Tabs ----

function switchTab(btn, panelId) {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('log-panel').style.display = panelId === 'log-panel' ? '' : 'none';
    document.getElementById('event-panel').style.display = panelId === 'event-panel' ? '' : 'none';
}

// ---- Init ----

document.addEventListener('DOMContentLoaded', () => {
    initWebSocket();
    refreshRoomList();

    addPreset('testapi', '');
    addPreset('buy_tower', '');
    addPreset('kill_unit', '1');
    addPreset('msdata', '');
    addPreset('pong', '');

    setInterval(refreshRoomList, 3000);
});
