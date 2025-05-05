use async_trait::async_trait;
use crate::Result;

#[async_trait]
pub trait Transport {
    // Method to send a message
    async fn send(&self, message: &str) -> Result<()>;

    // Method to receive a message
    async fn receive(&mut self) -> Result<Option<String>>;

    // Method to handle optional keypair authentication
    // This will be called during connection establishment
    async fn perform_auth(&self) -> Result<Option<()>>;

    // You might add other methods here later, such as for
    // handling connection closure or errors.
}