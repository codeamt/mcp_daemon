//! HTTP transport types for the Model Context Protocol
//! This module provides the transport layer for HTTP-based communication.
//!
//! Supports both HTTP/1.1 and HTTP/2 with TLS.

use async_trait::async_trait;
use super::{Message, Result, Transport, ServerSseTransport};
use super::websockets::{ClientWsTransport, ServerWsTransport};

/// Server-side HTTP transport variants
#[derive(Debug, Clone)]
pub enum ServerHttpTransport {
    /// Server-Sent Events transport
    #[cfg(feature = "sse")]
    Sse(ServerSseTransport),
    /// WebSocket transport
    Ws(ServerWsTransport),
    /// HTTP/2 transport
    Http2(super::http2::ServerHttp2Transport),
}

/// Client-side HTTP transport variants
#[derive(Debug, Clone)]
pub enum ClientHttpTransport {
    /// WebSocket transport
    Ws(ClientWsTransport),
    /// HTTP/2 transport
    Http2(super::http2::ClientHttp2Transport),
}

#[async_trait]
impl Transport for ServerHttpTransport {
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            #[cfg(feature = "sse")]
            Self::Sse(transport) => transport.send(message).await,
            Self::Ws(transport) => transport.send(message).await,
            Self::Http2(transport) => transport.send(message).await
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            #[cfg(feature = "sse")]
            Self::Sse(transport) => transport.receive().await,
            Self::Ws(transport) => transport.receive().await,
            Self::Http2(transport) => transport.receive().await
        }
    }

    async fn open(&self) -> Result<()> {
        match self {
            #[cfg(feature = "sse")]
            Self::Sse(transport) => transport.open().await,
            Self::Ws(transport) => transport.open().await,
            Self::Http2(transport) => transport.open().await
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            #[cfg(feature = "sse")]
            Self::Sse(transport) => transport.close().await,
            Self::Ws(transport) => transport.close().await,
            Self::Http2(transport) => transport.close().await
        }
    }
}

#[async_trait]
impl Transport for ClientHttpTransport {
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.send(message).await,
            Self::Http2(transport) => transport.send(message).await
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            Self::Ws(transport) => transport.receive().await,
            Self::Http2(transport) => transport.receive().await
        }
    }

    async fn open(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.open().await,
            Self::Http2(transport) => transport.open().await
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.close().await,
            Self::Http2(transport) => transport.close().await
        }
    }
}

/// Configuration for HTTP/2 transport
#[derive(Debug, Clone)]
pub struct Http2Config {
    /// TLS configuration
    pub tls_config: super::http2::ClientTlsConfig,
    /// Port to listen on
    pub port: u16,
    /// Host to listen on
    pub host: String,
}

impl Default for Http2Config {
    fn default() -> Self {
        Self {
            tls_config: super::http2::ClientTlsConfig::None,
            port: 8080,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Builder for HTTP/2 transport
#[derive(Debug, Clone)]
pub struct Http2Builder {
    config: Http2Config,
}

impl Default for Http2Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Http2Builder {
    /// Creates a new HTTP/2 transport builder
    pub fn new() -> Self {
        Self {
            config: Http2Config::default(),
        }
    }

    /// Sets whether to use TLS
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.config.tls_config = if use_tls {
            super::http2::ClientTlsConfig::Default
        } else {
            super::http2::ClientTlsConfig::None
        };
        self
    }

    /// Sets custom TLS configuration with a root certificate
    pub fn with_custom_tls(mut self, root_cert_path: String, verify_server: bool) -> Self {
        self.config.tls_config = super::http2::ClientTlsConfig::Custom {
            root_cert_path,
            verify_server,
            client_cert_path: None,
            client_key_path: None,
            server_name: None,
        };
        self
    }

    /// Sets client certificate for mutual TLS
    pub fn with_client_cert(mut self, cert_path: String, key_path: String) -> Self {
        match &mut self.config.tls_config {
            super::http2::ClientTlsConfig::Custom {
                client_cert_path,
                client_key_path,
                ..
            } => {
                *client_cert_path = Some(cert_path);
                *client_key_path = Some(key_path);
            },
            _ => {
                // If not already using custom TLS, create a new custom config with client cert
                self.config.tls_config = super::http2::ClientTlsConfig::Custom {
                    root_cert_path: "".to_string(), // Empty string will use system roots
                    verify_server: true,
                    client_cert_path: Some(cert_path),
                    client_key_path: Some(key_path),
                    server_name: None,
                };
            }
        }
        self
    }

    /// Sets Server Name Indication (SNI) for TLS
    pub fn with_sni(mut self, sni: String) -> Self {
        match &mut self.config.tls_config {
            super::http2::ClientTlsConfig::Custom {
                server_name,
                ..
            } => {
                *server_name = Some(sni);
            },
            _ => {
                // If not already using custom TLS, create a new custom config with SNI
                self.config.tls_config = super::http2::ClientTlsConfig::Custom {
                    root_cert_path: "".to_string(), // Empty string will use system roots
                    verify_server: true,
                    client_cert_path: None,
                    client_key_path: None,
                    server_name: Some(sni),
                };
            }
        }
        self
    }

    /// Sets the port to listen on
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Sets the host to listen on
    pub fn with_host(mut self, host: String) -> Self {
        self.config.host = host;
        self
    }

    /// Builds the HTTP/2 transport
    pub fn build(self) -> ClientHttpTransport {
        // Determine if TLS is enabled
        let use_tls = !matches!(self.config.tls_config, super::http2::ClientTlsConfig::None);

        // Create the URL
        let url = url::Url::parse(&format!("{}://{}:{}",
            if use_tls { "https" } else { "http" },
            self.config.host,
            self.config.port
        )).expect("Failed to parse URL");

        // Create headers
        let headers = std::collections::HashMap::new();

        // Create the HTTP/2 transport
        let transport = super::http2::ClientHttp2Transport::new(
            url,
            headers,
            self.config.tls_config
        );

        ClientHttpTransport::Http2(transport)
    }
}