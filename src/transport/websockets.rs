use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use actix_ws::Session;
use crate::Result;

pub struct WebSocketsTransport {
    session: Session,
}

impl WebSocketsTransport {
    pub fn new(session: Session) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Transport for WebSocketsTransport {
    async fn send(&self, message: &str) -> Result<()> {
        self.session.text(message).await.map_err(|e| crate::Error::TransportError(format!("Failed to send websocket message: {}", e)))?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        while let Some(msg) = self.session.next().await {
            match msg {
                Ok(actix_ws::Message::Text(text)) => return Ok(Some(text.to_string())),
                Ok(actix_ws::Message::Close(_)) => return Ok(None), // Connection closed
                Ok(_) => {},
                Err(e) => return Err(crate::Error::TransportError(format!("Websocket receive error: {}", e))),
            }
        }
        Ok(None) // Stream ended
    }

    async fn perform_auth(&self) -> Result<Option<()>> {
        // Keypair authentication integration for WebSockets will be implemented here.
        Ok(None)
    }
}

use super::traits::Transport; // Import the Transport trait
