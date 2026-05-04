//! High-level chat client: connect, auth, subscribe, send, and stream
//! incoming events.
//!
//! Spawns a background task that owns the WebSocket and forwards
//! [`ChatEvent`]s through a tokio channel. The caller drives outgoing messages
//! through the returned [`ChatClient`] handle.

use chrono::Utc;
use ed25519_dalek::Signature;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;
use tt_identity::LocalKeypair;
use uuid::Uuid;

use crate::connection::{self, WsStream};
use crate::error::{ChatError, Result};
use crate::protocol::{ChatMessage, ClientMessage, ServerMessage, auth_payload, message_payload};

/// Events surfaced to the consumer of [`ChatClient`].
#[derive(Debug, Clone)]
pub enum ChatEvent {
    Authenticated {
        server_id: String,
        server_name: String,
    },
    Message(ChatMessage),
    History {
        channel: String,
        messages: Vec<ChatMessage>,
    },
    Error {
        code: String,
        message: String,
    },
    Disconnected {
        reason: String,
    },
}

pub struct ChatClient {
    out: mpsc::Sender<ClientMessage>,
    join: JoinHandle<()>,
    pubkey_hex: String,
}

impl ChatClient {
    /// Connect to `url`, perform the ed25519 challenge-response, then return
    /// a handle plus a receiver of [`ChatEvent`]s.
    pub async fn connect(
        url: &str,
        keypair: &LocalKeypair,
    ) -> Result<(Self, mpsc::Receiver<ChatEvent>)> {
        let mut ws = connection::connect(url).await?;

        // Wait for AuthChallenge.
        let challenge = match connection::recv(&mut ws).await? {
            ServerMessage::AuthChallenge { nonce_hex } => nonce_hex,
            other => {
                return Err(ChatError::Protocol(format!(
                    "expected auth_challenge, got {other:?}"
                )));
            }
        };
        let nonce_bytes =
            hex::decode(&challenge).map_err(|e| ChatError::Protocol(format!("nonce hex: {e}")))?;
        let payload = auth_payload(&nonce_bytes);
        let signature: Signature = keypair.sign(&payload);
        let pubkey_hex = hex::encode(keypair.public_bytes().0);
        connection::send(
            &mut ws,
            &ClientMessage::Hello {
                pubkey_hex: pubkey_hex.clone(),
                signature_hex: hex::encode(signature.to_bytes()),
            },
        )
        .await?;

        let (out_tx, mut out_rx) = mpsc::channel::<ClientMessage>(64);
        let (event_tx, event_rx) = mpsc::channel::<ChatEvent>(128);

        // Wait for AuthAccepted / AuthRejected.
        match connection::recv(&mut ws).await? {
            ServerMessage::AuthAccepted {
                server_id,
                server_name,
            } => {
                let _ = event_tx
                    .send(ChatEvent::Authenticated {
                        server_id,
                        server_name,
                    })
                    .await;
            }
            ServerMessage::AuthRejected { reason } => {
                return Err(ChatError::AuthRejected(reason));
            }
            other => {
                return Err(ChatError::Protocol(format!(
                    "expected auth_accepted/rejected, got {other:?}"
                )));
            }
        }

        let join = tokio::spawn(async move {
            run_loop(ws, &mut out_rx, event_tx).await;
        });

        Ok((
            Self {
                out: out_tx,
                join,
                pubkey_hex,
            },
            event_rx,
        ))
    }

    pub async fn subscribe(&self, channel: impl Into<String>) -> Result<()> {
        self.out
            .send(ClientMessage::Subscribe {
                channel: channel.into(),
            })
            .await
            .map_err(|_| ChatError::Closed)
    }

    pub async fn unsubscribe(&self, channel: impl Into<String>) -> Result<()> {
        self.out
            .send(ClientMessage::Unsubscribe {
                channel: channel.into(),
            })
            .await
            .map_err(|_| ChatError::Closed)
    }

    /// Compose, sign and ship a message. Pass `reply_to = Some(parent_id)`
    /// to thread the message under another one.
    pub async fn send_text(
        &self,
        channel: impl Into<String>,
        content: impl Into<String>,
        reply_to: Option<Uuid>,
        keypair: &LocalKeypair,
    ) -> Result<ChatMessage> {
        let mut m = ChatMessage {
            id: Uuid::new_v4(),
            channel: channel.into(),
            author_pubkey: self.pubkey_hex.clone(),
            content: content.into(),
            reply_to,
            sent_at: Utc::now(),
            signature: String::new(),
        };
        let payload = message_payload(&m);
        let sig: Signature = keypair.sign(&payload);
        m.signature = hex::encode(sig.to_bytes());
        self.out
            .send(ClientMessage::Send { message: m.clone() })
            .await
            .map_err(|_| ChatError::Closed)?;
        Ok(m)
    }

    pub async fn fetch_history(
        &self,
        channel: impl Into<String>,
        limit: Option<u32>,
    ) -> Result<()> {
        self.out
            .send(ClientMessage::History {
                channel: channel.into(),
                before: None,
                limit,
            })
            .await
            .map_err(|_| ChatError::Closed)
    }

    pub fn pubkey_hex(&self) -> &str {
        &self.pubkey_hex
    }

    pub async fn shutdown(self) {
        drop(self.out);
        let _ = self.join.await;
    }
}

async fn run_loop(
    mut ws: WsStream,
    out_rx: &mut mpsc::Receiver<ClientMessage>,
    event_tx: mpsc::Sender<ChatEvent>,
) {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

    loop {
        tokio::select! {
            biased;
            outgoing = out_rx.recv() => {
                let Some(msg) = outgoing else { break };
                let json = match serde_json::to_string(&msg) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(target: "tt_chat::client", "serde error sending: {e}");
                        continue;
                    }
                };
                if let Err(e) = ws.send(WsMessage::Text(json.into())).await {
                    let _ = event_tx
                        .send(ChatEvent::Disconnected { reason: e.to_string() })
                        .await;
                    return;
                }
            }
            incoming = ws.next() => {
                match incoming {
                    Some(Ok(WsMessage::Text(t))) => {
                        match serde_json::from_str::<ServerMessage>(&t) {
                            Ok(server_msg) => dispatch(&event_tx, server_msg).await,
                            Err(e) => warn!(target: "tt_chat::client", "bad json: {e}"),
                        }
                    }
                    Some(Ok(WsMessage::Ping(p))) => {
                        let _ = ws.send(WsMessage::Pong(p)).await;
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        let _ = event_tx
                            .send(ChatEvent::Disconnected { reason: "closed".into() })
                            .await;
                        return;
                    }
                    Some(Err(e)) => {
                        let _ = event_tx
                            .send(ChatEvent::Disconnected { reason: e.to_string() })
                            .await;
                        return;
                    }
                    _ => continue,
                }
            }
        }
    }

    let _ = event_tx
        .send(ChatEvent::Disconnected {
            reason: "outgoing channel closed".into(),
        })
        .await;
}

async fn dispatch(tx: &mpsc::Sender<ChatEvent>, msg: ServerMessage) {
    match msg {
        ServerMessage::MessageNew { message } => {
            let _ = tx.send(ChatEvent::Message(message)).await;
        }
        ServerMessage::History { channel, messages } => {
            let _ = tx.send(ChatEvent::History { channel, messages }).await;
        }
        ServerMessage::Error { code, message } => {
            let _ = tx.send(ChatEvent::Error { code, message }).await;
        }
        ServerMessage::Pong => {}
        ServerMessage::AuthChallenge { .. }
        | ServerMessage::AuthAccepted { .. }
        | ServerMessage::AuthRejected { .. } => {
            warn!(target: "tt_chat::client", "unexpected auth message after handshake");
        }
    }
}
