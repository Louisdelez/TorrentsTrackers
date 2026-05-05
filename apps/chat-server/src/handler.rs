//! WebSocket connection handler — auth handshake then message loop.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, info, warn};
use tt_chat::ChatMessage;
use tt_chat::protocol::{ClientMessage, NONCE_LEN, ServerMessage, auth_payload, message_payload};

use crate::state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut ws: WebSocket, state: Arc<AppState>) {
    let pubkey_hex = match handshake(&mut ws, &state).await {
        Ok(pk) => pk,
        Err(e) => {
            warn!(target: "chat::handler", "handshake failed: {e}");
            return;
        }
    };

    info!(target: "chat::handler", "client {pubkey_hex} connected");
    if let Err(e) = run(&mut ws, &state, &pubkey_hex).await {
        warn!(target: "chat::handler", "session for {pubkey_hex} ended: {e}");
    }
    let _ = ws.close().await;
}

async fn handshake(ws: &mut WebSocket, state: &AppState) -> anyhow::Result<String> {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    send(
        ws,
        &ServerMessage::AuthChallenge {
            nonce_hex: hex::encode(nonce),
        },
    )
    .await?;

    let hello = recv_text(ws).await?;
    let cm: ClientMessage = serde_json::from_str(&hello)?;
    let (pubkey_hex, sig_hex) = match cm {
        ClientMessage::Hello {
            pubkey_hex,
            signature_hex,
        } => (pubkey_hex, signature_hex),
        _ => {
            send(
                ws,
                &ServerMessage::AuthRejected {
                    reason: "expected hello".into(),
                },
            )
            .await?;
            anyhow::bail!("client sent non-hello first");
        }
    };

    if state.db.is_banned(&pubkey_hex)? {
        send(
            ws,
            &ServerMessage::AuthRejected {
                reason: "banned".into(),
            },
        )
        .await?;
        anyhow::bail!("banned pubkey {pubkey_hex} attempted to connect");
    }

    let pubkey_bytes = hex::decode(&pubkey_hex)?;
    if pubkey_bytes.len() != 32 {
        send(
            ws,
            &ServerMessage::AuthRejected {
                reason: "bad pubkey length".into(),
            },
        )
        .await?;
        anyhow::bail!("bad pubkey length");
    }
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pubkey_bytes);
    let vk = VerifyingKey::from_bytes(&pk_arr)?;

    let sig_bytes = hex::decode(&sig_hex)?;
    if sig_bytes.len() != 64 {
        send(
            ws,
            &ServerMessage::AuthRejected {
                reason: "bad signature length".into(),
            },
        )
        .await?;
        anyhow::bail!("bad signature length");
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&sig_arr);

    if vk.verify(&auth_payload(&nonce), &sig).is_err() {
        send(
            ws,
            &ServerMessage::AuthRejected {
                reason: "bad signature".into(),
            },
        )
        .await?;
        anyhow::bail!("bad signature");
    }

    send(
        ws,
        &ServerMessage::AuthAccepted {
            server_id: state.config.server_id.clone(),
            server_name: state.config.server_name.clone(),
        },
    )
    .await?;
    Ok(pubkey_hex)
}

async fn run(ws: &mut WebSocket, state: &Arc<AppState>, pubkey_hex: &str) -> anyhow::Result<()> {
    let mut subscribed: std::collections::HashSet<String> = Default::default();
    let mut rx = state.broadcast.subscribe();

    let rate_window = Duration::from_secs(60);
    let rate_max = state.config.rate_limit_per_min as usize;
    let mut window_start = Instant::now();
    let mut window_count = 0usize;

    loop {
        tokio::select! {
            biased;

            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(m) if subscribed.contains(&m.channel) => {
                        send(ws, &ServerMessage::MessageNew { message: m }).await?;
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(_)) => {
                        send(ws, &ServerMessage::Error {
                            code: "lagged".into(),
                            message: "broadcast lagged; please reconnect".into(),
                        }).await?;
                    }
                    Err(RecvError::Closed) => break,
                }
            }

            ws_msg = ws.recv() => {
                let Some(msg) = ws_msg else { break };
                let msg = msg?;
                match msg {
                    Message::Text(t) => {
                        let cm: ClientMessage = match serde_json::from_str(&t) {
                            Ok(v) => v,
                            Err(e) => {
                                send(ws, &ServerMessage::Error {
                                    code: "bad_json".into(),
                                    message: e.to_string(),
                                }).await?;
                                continue;
                            }
                        };
                        match cm {
                            ClientMessage::Subscribe { channel } => {
                                subscribed.insert(channel.clone());
                                debug!(target: "chat::handler", "{pubkey_hex} subscribed to {channel}");
                            }
                            ClientMessage::Unsubscribe { channel } => {
                                subscribed.remove(&channel);
                            }
                            ClientMessage::History { channel, before, limit } => {
                                let limit = limit.unwrap_or(state.config.history_default_limit);
                                let messages = state.db.history(&channel, before, limit)?;
                                send(ws, &ServerMessage::History { channel, messages }).await?;
                            }
                            ClientMessage::Send { mut message } => {
                                if !verify_message(&message) {
                                    send(ws, &ServerMessage::Error {
                                        code: "bad_signature".into(),
                                        message: "message signature invalid".into(),
                                    }).await?;
                                    continue;
                                }
                                if message.author_pubkey != pubkey_hex {
                                    send(ws, &ServerMessage::Error {
                                        code: "wrong_author".into(),
                                        message: "author_pubkey must match the connected key".into(),
                                    }).await?;
                                    continue;
                                }
                                if rate_max > 0 {
                                    if window_start.elapsed() > rate_window {
                                        window_start = Instant::now();
                                        window_count = 0;
                                    }
                                    if window_count >= rate_max {
                                        send(ws, &ServerMessage::Error {
                                            code: "rate_limited".into(),
                                            message: "slow down".into(),
                                        }).await?;
                                        continue;
                                    }
                                    window_count += 1;
                                }
                                // Server-trusted timestamp — keep author timestamp inside the
                                // signed payload, but tag arrival time on the wire object.
                                message.signature = message.signature.to_lowercase();
                                state.db.insert_message(&message)?;
                                let _ = state.broadcast.send(message);
                            }
                            ClientMessage::Ping => {
                                send(ws, &ServerMessage::Pong).await?;
                            }
                            ClientMessage::Hello { .. } => {
                                send(ws, &ServerMessage::Error {
                                    code: "already_authenticated".into(),
                                    message: "already authenticated".into(),
                                }).await?;
                            }
                        }
                    }
                    Message::Ping(p) => { ws.send(Message::Pong(p)).await?; }
                    Message::Pong(_) | Message::Binary(_) => {}
                    Message::Close(_) => break,
                }
            }
        }
    }
    Ok(())
}

fn verify_message(m: &ChatMessage) -> bool {
    let Ok(pk) = hex::decode(&m.author_pubkey) else {
        return false;
    };
    if pk.len() != 32 {
        return false;
    }
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pk);
    let Ok(vk) = VerifyingKey::from_bytes(&pk_arr) else {
        return false;
    };
    let Ok(sig) = hex::decode(&m.signature) else {
        return false;
    };
    if sig.len() != 64 {
        return false;
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig);
    let signature = Signature::from_bytes(&sig_arr);
    vk.verify(&message_payload(m), &signature).is_ok()
}

async fn send(ws: &mut WebSocket, m: &ServerMessage) -> anyhow::Result<()> {
    let text = serde_json::to_string(m)?;
    ws.send(Message::Text(text)).await?;
    Ok(())
}

async fn recv_text(ws: &mut WebSocket) -> anyhow::Result<String> {
    loop {
        match ws.recv().await {
            Some(Ok(Message::Text(t))) => return Ok(t.to_string()),
            Some(Ok(Message::Ping(p))) => {
                ws.send(Message::Pong(p)).await?;
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(e.into()),
            None => anyhow::bail!("connection closed"),
        }
    }
}
