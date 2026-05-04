//! Low-level WebSocket connection helpers.

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::error::{ChatError, Result};
use crate::protocol::{ClientMessage, ServerMessage};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect(url: &str) -> Result<WsStream> {
    let parsed = url::Url::parse(url).map_err(|e| ChatError::InvalidUrl(e.to_string()))?;
    if !matches!(parsed.scheme(), "ws" | "wss") {
        return Err(ChatError::InvalidUrl(format!(
            "scheme must be ws or wss, got '{}'",
            parsed.scheme()
        )));
    }
    let (ws, _resp) = tokio_tungstenite::connect_async(parsed.as_str()).await?;
    Ok(ws)
}

pub async fn send(ws: &mut WsStream, msg: &ClientMessage) -> Result<()> {
    let text = serde_json::to_string(msg)?;
    ws.send(WsMessage::Text(text.into())).await?;
    Ok(())
}

pub async fn recv(ws: &mut WsStream) -> Result<ServerMessage> {
    loop {
        match ws.next().await {
            Some(Ok(WsMessage::Text(t))) => {
                return serde_json::from_str::<ServerMessage>(&t).map_err(ChatError::from);
            }
            Some(Ok(WsMessage::Binary(_))) => {
                return Err(ChatError::Protocol("unexpected binary frame".into()));
            }
            Some(Ok(WsMessage::Ping(p))) => {
                ws.send(WsMessage::Pong(p)).await?;
                continue;
            }
            Some(Ok(WsMessage::Pong(_))) | Some(Ok(WsMessage::Frame(_))) => continue,
            Some(Ok(WsMessage::Close(_))) => return Err(ChatError::Closed),
            Some(Err(e)) => return Err(ChatError::Ws(e)),
            None => return Err(ChatError::Closed),
        }
    }
}
