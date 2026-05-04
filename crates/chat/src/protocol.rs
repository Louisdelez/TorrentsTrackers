//! Wire-format types for the TorrentsTrackers chat protocol (`tt-chat-v1`).
//!
//! The transport is a single WebSocket connection per client. Messages are
//! JSON envelopes tagged with `kind`. Both directions can be sent at any
//! time after the auth handshake completes.
//!
//! ## Handshake
//!
//! ```text
//! S → C   AuthChallenge { nonce }
//! C → S   Hello { pubkey, signature(domain || nonce) }
//! S → C   AuthAccepted { server_id, server_name } | AuthRejected { reason }
//! ```
//!
//! Domain string for the signature is `b"tt-chat-v1\0"` followed by the
//! 32-byte raw nonce. This makes the auth signatures distinct from any
//! entry signatures so a stolen entry signature can't be replayed as a
//! login.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_DOMAIN: &[u8] = b"tt-chat-v1\0";
pub const NONCE_LEN: usize = 32;

/// A persisted chat message, identical on the wire and at rest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: Uuid,
    pub channel: String,
    /// 64-char lowercase hex.
    pub author_pubkey: String,
    pub content: String,
    pub reply_to: Option<Uuid>,
    pub sent_at: DateTime<Utc>,
    /// 128-char lowercase hex. Signature of
    /// `tt-chat-msg-v1\0 || id || channel || author_pubkey || content
    ///  || reply_to_or_zero || sent_at_rfc3339`.
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServerMessage {
    /// First message after handshake. The client must reply with a [`Hello`]
    /// signing this nonce.
    AuthChallenge { nonce_hex: String },
    AuthAccepted {
        server_id: String,
        server_name: String,
    },
    AuthRejected {
        reason: String,
    },
    /// Realtime broadcast of a message in a subscribed channel.
    MessageNew {
        message: ChatMessage,
    },
    /// Reply to a History request.
    History {
        channel: String,
        messages: Vec<ChatMessage>,
    },
    Pong,
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello {
        pubkey_hex: String,
        signature_hex: String,
    },
    Subscribe {
        channel: String,
    },
    Unsubscribe {
        channel: String,
    },
    Send {
        message: ChatMessage,
    },
    History {
        channel: String,
        before: Option<DateTime<Utc>>,
        limit: Option<u32>,
    },
    Ping,
}

/// Build the byte payload that `Hello.signature_hex` covers.
pub fn auth_payload(nonce: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PROTOCOL_DOMAIN.len() + nonce.len());
    out.extend_from_slice(PROTOCOL_DOMAIN);
    out.extend_from_slice(nonce);
    out
}

/// Build the deterministic byte payload that `ChatMessage.signature` covers.
pub fn message_payload(m: &ChatMessage) -> Vec<u8> {
    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(b"tt-chat-msg-v1\0");
    out.extend_from_slice(m.id.as_bytes());
    push_str(&mut out, &m.channel);
    push_str(&mut out, &m.author_pubkey);
    push_str(&mut out, &m.content);
    match m.reply_to {
        Some(id) => {
            out.push(1);
            out.extend_from_slice(id.as_bytes());
        }
        None => out.push(0),
    }
    push_str(&mut out, &m.sent_at.to_rfc3339());
    out
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_payload_uses_domain() {
        let p = auth_payload(&[1u8; 32]);
        assert!(p.starts_with(PROTOCOL_DOMAIN));
        assert_eq!(p.len(), PROTOCOL_DOMAIN.len() + 32);
    }

    #[test]
    fn message_payload_is_deterministic() {
        let m = ChatMessage {
            id: Uuid::nil(),
            channel: "main".into(),
            author_pubkey: "00".repeat(32),
            content: "hello".into(),
            reply_to: None,
            sent_at: Utc::now(),
            signature: String::new(),
        };
        let a = message_payload(&m);
        let b = message_payload(&m);
        assert_eq!(a, b);
    }

    #[test]
    fn message_payload_changes_with_content() {
        let mut m = ChatMessage {
            id: Uuid::nil(),
            channel: "main".into(),
            author_pubkey: "00".repeat(32),
            content: "a".into(),
            reply_to: None,
            sent_at: Utc::now(),
            signature: String::new(),
        };
        let a = message_payload(&m);
        m.content = "b".into();
        let b = message_payload(&m);
        assert_ne!(a, b);
    }

    #[test]
    fn server_message_serde_roundtrip() {
        let s = ServerMessage::AuthAccepted {
            server_id: "id".into(),
            server_name: "Test".into(),
        };
        let j = serde_json::to_string(&s).unwrap();
        let r: ServerMessage = serde_json::from_str(&j).unwrap();
        match r {
            ServerMessage::AuthAccepted { server_name, .. } => assert_eq!(server_name, "Test"),
            _ => panic!("wrong variant"),
        }
    }
}
