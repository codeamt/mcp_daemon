use ring::{agreement, error::Unspecified, rand, signature};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::Result;

// --- Authentication Handshake Messages ---

// Message sent by the server to initiate authentication
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthChallenge {
    pub public_key: Vec<u8>,
    pub challenge: Vec<u8>,
}

// Message sent by the client with the signed challenge
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

// --- Keypair Management ---

pub struct Keypair {
    signing_key: signature::Ed25519KeyPair,
    public_key_bytes: Vec<u8>,
}

impl Keypair {
    pub fn generate() -> Result<Self> {
        let rng = rand::SystemRandom::new();
        let signing_key = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
        let public_key_bytes = signing_key.public_key().as_ref().to_vec();
        Ok(Self {
            signing_key,
            public_key_bytes,
        })
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key_bytes
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let signature = self.signing_key.sign(message);
        Ok(signature.as_ref().to_vec())
    }

    pub fn verify(&self, public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<()> {
        let peer_public_key = signature::UnparsedPublicKey::new(
            &signature::Ed25519::RING_CONTEXT,
            public_key_bytes,
        );
        peer_public_key.verify(message, signature_bytes).map_err(|_| crate::Error::AuthenticationError("Signature verification failed".into()))
    }
}

// --- Authentication Handshake Logic ---

// Server-side handshake initiation
pub async fn server_auth_handshake(
    sender: &mut actix_ws::Sender,
    stream: &mut actix_ws::MessageStream,
    server_keypair: &Keypair,
) -> Result<()> {
    let mut rng = rand::SystemRandom::new();
    let mut challenge = vec![0u8; 32];
    rng.fill(&mut challenge).map_err(|_| crate::Error::AuthenticationError("Failed to generate challenge".into()))?;

    let auth_challenge = AuthChallenge {
        public_key: server_keypair.public_key().to_vec(),
        challenge,
    };

    let challenge_json = serde_json::to_string(&auth_challenge)
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to serialize challenge: {}", e)))?;

    stream.write_all(challenge_json.as_bytes()).await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to send challenge: {}", e)))?;
    stream.write_all(b"\n").await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to send challenge newline: {}", e)))?;

    let mut client_response_json = String::new();
    let mut reader = tokio::io::BufReader::new(stream);
    reader.read_line(&mut client_response_json).await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to receive client response: {}", e)))?;

    let client_response: AuthResponse = serde_json::from_str(&client_response_json.trim())
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to deserialize client response: {}", e)))?;

    server_keypair.verify(&client_response.public_key, &auth_challenge.challenge, &client_response.signature)?;

    // In a real implementation, you would now associate the client's public key with the connection
    // for future authorization checks.

    Ok(())
}

// Client-side handshake response
pub async fn client_auth_handshake(
    sender: &mut actix_ws::Sender,
    stream: &mut actix_ws::MessageStream,
    client_keypair: &Keypair,
) -> Result<()> {
    let mut server_challenge_json = String::new();
    let mut reader = tokio::io::BufReader::new(stream);
    reader.read_line(&mut server_challenge_json).await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to receive server challenge: {}", e)))?;

    let server_challenge: AuthChallenge = serde_json::from_str(&server_challenge_json.trim())
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to deserialize server challenge: {}", e)))?;

    // In a real implementation, you would verify the server's public key here if you have a trusted list

    let signature = client_keypair.sign(&server_challenge.challenge)?;

    let auth_response = AuthResponse {
        public_key: client_keypair.public_key().to_vec(),
        signature,
    };

    let response_json = serde_json::to_string(&auth_response)
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to serialize client response: {}", e)))?;

    stream.write_all(response_json.as_bytes()).await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to send client response: {}", e)))?;
    stream.write_all(b"\n").await
        .map_err(|e| crate::Error::AuthenticationError(format!("Failed to send client response newline: {}", e)))?;

    Ok(())
}
