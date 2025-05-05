use async_trait::async_trait;
use tokio::sync::mpsc;
use actix_web_lab::sse;
use crate::Result;

pub struct SseTransport {
    sender: mpsc::Sender<sse::Event>,
}

impl SseTransport {
    pub fn new(sender: mpsc::Sender<sse::Event>) -> Self {
        Self { sender }
    }

    /// Create a new SSE transport pair (transport and SSE responder)
    pub fn from_channel() -> (Self, sse::Sse<sse::ChannelStream>) {
        let (tx, rx) = mpsc::channel(10);
        let transport = Self::new(tx);
        let sse = sse::Sse::from_infallible_receiver(rx)
            .with_retry_duration(std::time::Duration::from_secs(10));
        (transport, sse)
    }
}

#[async_trait]
impl Transport for SseTransport {
    async fn send(&self, message: &str) -> Result<()> {
        self.sender.send(sse::Data::new(message).into())
            .await
            .map_err(|e| crate::Error::TransportError(format!("SSE send failed: {}", e)))?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        // SSE is primarily server-to-client, so receive is not typically used.
        // We can leave this as None or add logic for client messages if needed later.
        Ok(None)
    }

    async fn perform_auth(&self) -> Result<Option<()>> {
        // Keypair authentication integration for SSE will need to be designed
        // based on how the initial connection is established.
        Ok(None)
    }
}

// Note: This is a simplified implementation. A full implementation would involve
// proper error handling for sse::SseSender::send and potentially a way for the
// server to manage multiple SSE connections.

