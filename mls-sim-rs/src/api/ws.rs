use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::room::{LogEntry, OutEvent};
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut log_rx: Option<broadcast::Receiver<LogEntry>> = None;
    let mut event_rx: Option<broadcast::Receiver<OutEvent>> = None;

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&*text) {
                            let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            match msg_type {
                                "join_room" => {
                                    if let Some(room_id) = parsed.get("room_id").and_then(|r| r.as_str()) {
                                        // Collect buffered data and subscribers while holding the lock, then drop it
                                        let (buffered_logs, buffered_events, new_log_rx, new_event_rx) = {
                                            let manager = state.manager.read().unwrap();
                                            if let Some(room) = manager.get_room(room_id) {
                                                let shared = room.shared.read().unwrap();
                                                let logs: Vec<String> = shared.log_buffer.iter()
                                                    .map(|entry| serde_json::json!({"type": "log", "data": entry}).to_string())
                                                    .collect();
                                                let events: Vec<String> = shared.event_buffer.iter()
                                                    .map(|ev| serde_json::json!({"type": "out_event", "data": ev}).to_string())
                                                    .collect();
                                                drop(shared);
                                                let lr = room.log_tx.subscribe();
                                                let er = room.out_event_tx.subscribe();
                                                (logs, events, Some(lr), Some(er))
                                            } else {
                                                (vec![], vec![], None, None)
                                            }
                                        };

                                        // Now send buffered data without holding any locks
                                        for msg_str in buffered_logs {
                                            let _ = sender.send(Message::Text(msg_str.into())).await;
                                        }
                                        for msg_str in buffered_events {
                                            let _ = sender.send(Message::Text(msg_str.into())).await;
                                        }
                                        log_rx = new_log_rx;
                                        event_rx = new_event_rx;
                                    }
                                }
                                "leave_room" => {
                                    log_rx = None;
                                    event_rx = None;
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            log_entry = async {
                match log_rx.as_mut() {
                    Some(rx) => rx.recv().await.ok(),
                    None => {
                        let () = std::future::pending().await;
                        None
                    }
                }
            } => {
                if let Some(entry) = log_entry {
                    let msg = serde_json::json!({"type": "log", "data": entry});
                    if sender.send(Message::Text(msg.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
            out_event = async {
                match event_rx.as_mut() {
                    Some(rx) => rx.recv().await.ok(),
                    None => {
                        let () = std::future::pending().await;
                        None
                    }
                }
            } => {
                if let Some(ev) = out_event {
                    let msg = serde_json::json!({"type": "out_event", "data": ev});
                    if sender.send(Message::Text(msg.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}
