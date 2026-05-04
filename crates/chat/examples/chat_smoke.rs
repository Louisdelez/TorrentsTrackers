//! End-to-end smoke test: connect to a running chat server, auth, send a
//! message, receive it via the subscription.
//!
//! Usage: `cargo run -p tt-chat --example chat_smoke -- ws://127.0.0.1:6970/ws`

use std::time::Duration;

use tt_chat::{ChatClient, ChatEvent};
use tt_identity::LocalKeypair;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:6970/ws".into());

    let kp = LocalKeypair::generate();
    println!("client npub: {}", kp.npub());

    let (client, mut events) = ChatClient::connect(&url, &kp).await?;
    println!("connected, awaiting auth_accepted...");

    // Drain auth event.
    while let Some(ev) = tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .ok()
        .flatten()
    {
        match ev {
            ChatEvent::Authenticated {
                server_id,
                server_name,
            } => {
                println!("auth ok: server={server_name} ({server_id})");
                break;
            }
            ChatEvent::Disconnected { reason } => {
                anyhow::bail!("disconnected during auth: {reason}");
            }
            other => println!("unexpected pre-auth event: {other:?}"),
        }
    }

    client.subscribe("main").await?;
    println!("subscribed to 'main'");

    let payload = format!("hello at {}", chrono::Utc::now().to_rfc3339());
    let sent = client.send_text("main", &payload, &kp).await?;
    println!("sent {} (len {})", sent.id, sent.content.len());

    // Wait for the broadcast back to us.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    let mut got_back = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), events.recv()).await {
            Ok(Some(ChatEvent::Message(m))) if m.id == sent.id => {
                println!("✓ received own broadcast: {}", m.content);
                got_back = true;
                break;
            }
            Ok(Some(other)) => println!("event: {other:?}"),
            Ok(None) => {
                anyhow::bail!("event channel closed unexpectedly");
            }
            Err(_) => continue,
        }
    }

    if !got_back {
        anyhow::bail!("did not receive own broadcast within 3s");
    }

    client.shutdown().await;
    println!("OK");
    Ok(())
}
