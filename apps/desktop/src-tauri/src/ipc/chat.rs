//! IPC commands for the chat client.
//!
//! Each connection runs in a background task that pumps [`ChatEvent`]s into
//! a single Tauri event named `chat-event`. The frontend listens once at
//! startup and routes by `server_id`.

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::task::JoinHandle;
use tracing::warn;
use tt_chat::{ChatClient, ChatEvent, ChatMessage};
use tt_identity::{DefaultStore, IdentityStore, LocalKeypair};

use crate::state::{AppState, ChatConnection};

const EVENT_NAME: &str = "chat-event";

#[derive(Debug, Serialize, Clone)]
pub struct ChatServerDto {
    pub server_id: String,
    pub server_name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatEventDto {
    Authenticated {
        server_id: String,
        server_name: String,
        url: String,
    },
    Message {
        server_id: String,
        message: ChatMessage,
    },
    History {
        server_id: String,
        channel: String,
        messages: Vec<ChatMessage>,
    },
    Error {
        server_id: String,
        code: String,
        message: String,
    },
    Disconnected {
        server_id: String,
        reason: String,
    },
}

#[tauri::command]
pub async fn chat_list(state: State<'_, AppState>) -> Result<Vec<ChatServerDto>, String> {
    let chats = state.chats.lock().await;
    Ok(chats
        .values()
        .map(|c| ChatServerDto {
            server_id: c.server_id.clone(),
            server_name: c.server_name.clone(),
            url: c.url.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn chat_connect(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<ChatServerDto, String> {
    let kp = load_keypair()?;

    let (client, mut events) = ChatClient::connect(&url, &kp)
        .await
        .map_err(|e| e.to_string())?;

    // First event is always Authenticated.
    let (server_id, server_name) = match events.recv().await {
        Some(ChatEvent::Authenticated {
            server_id,
            server_name,
        }) => (server_id, server_name),
        Some(ChatEvent::Disconnected { reason }) => {
            return Err(format!("disconnected before auth: {reason}"));
        }
        Some(other) => {
            return Err(format!("unexpected first event: {other:?}"));
        }
        None => return Err("event channel closed during auth".into()),
    };

    let _ = app_handle.emit(
        EVENT_NAME,
        ChatEventDto::Authenticated {
            server_id: server_id.clone(),
            server_name: server_name.clone(),
            url: url.clone(),
        },
    );

    let app_for_task = app_handle.clone();
    let server_id_for_task = server_id.clone();
    let join = tokio::spawn(async move {
        while let Some(ev) = events.recv().await {
            let dto = match ev {
                ChatEvent::Message(m) => ChatEventDto::Message {
                    server_id: server_id_for_task.clone(),
                    message: m,
                },
                ChatEvent::History { channel, messages } => ChatEventDto::History {
                    server_id: server_id_for_task.clone(),
                    channel,
                    messages,
                },
                ChatEvent::Error { code, message } => ChatEventDto::Error {
                    server_id: server_id_for_task.clone(),
                    code,
                    message,
                },
                ChatEvent::Disconnected { reason } => ChatEventDto::Disconnected {
                    server_id: server_id_for_task.clone(),
                    reason,
                },
                ChatEvent::Authenticated { .. } => continue, // already handled
            };
            if let Err(e) = app_for_task.emit(EVENT_NAME, dto) {
                warn!(target: "tt_desktop::chat", "emit failed: {e}");
            }
        }
    });

    let mut chats = state.chats.lock().await;
    chats.insert(
        server_id.clone(),
        ChatConnection {
            client,
            server_id: server_id.clone(),
            server_name: server_name.clone(),
            url: url.clone(),
            _join: join,
        },
    );

    Ok(ChatServerDto {
        server_id,
        server_name,
        url,
    })
}

#[tauri::command]
pub async fn chat_disconnect(state: State<'_, AppState>, server_id: String) -> Result<(), String> {
    let mut chats = state.chats.lock().await;
    if let Some(conn) = chats.remove(&server_id) {
        // ChatClient::shutdown consumes self.
        conn.client.shutdown().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn chat_send(
    state: State<'_, AppState>,
    server_id: String,
    channel: String,
    content: String,
    reply_to: Option<String>,
) -> Result<ChatMessage, String> {
    let kp = load_keypair()?;
    let reply_to_uuid = match reply_to {
        Some(s) => Some(uuid::Uuid::parse_str(&s).map_err(|e| e.to_string())?),
        None => None,
    };
    let chats = state.chats.lock().await;
    let conn = chats
        .get(&server_id)
        .ok_or_else(|| "no such chat server".to_string())?;
    conn.client
        .send_text(&channel, &content, reply_to_uuid, &kp)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_history(
    state: State<'_, AppState>,
    server_id: String,
    channel: String,
    limit: Option<u32>,
) -> Result<(), String> {
    let chats = state.chats.lock().await;
    let conn = chats
        .get(&server_id)
        .ok_or_else(|| "no such chat server".to_string())?;
    conn.client
        .subscribe(&channel)
        .await
        .map_err(|e| e.to_string())?;
    conn.client
        .fetch_history(&channel, limit)
        .await
        .map_err(|e| e.to_string())
}

fn load_keypair() -> Result<LocalKeypair, String> {
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    let seed = store
        .load()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "identity not initialised".to_string())?;
    Ok(LocalKeypair::from_seed(&seed))
}

// Silence unused-import warning if Drop happens to be implicit.
#[allow(dead_code)]
fn _used(_: JoinHandle<()>) {}
