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
        use crate::transport::auth::{server_auth_handshake, client_auth_handshake, Keypair};
        use tokio::io::{AsyncRead, AsyncWrite};
        
        // Get the underlying transport stream
        let stream = self.session.get_mut().get_mut();
        
        // Check if we're acting as server or client
        if self.session.is_server() {
            // Server-side authentication
            let server_keypair = Keypair::generate()?;
            server_auth_handshake(stream, &server_keypair).await?;
        } else {
            // Client-side authentication
            let client_keypair = Keypair::generate()?;
            client_auth_handshake(stream, &client_keypair).await?;
        }
        
        Ok(Some(()))
    }
}

use super::traits::Transport; // Import the Transport trait
