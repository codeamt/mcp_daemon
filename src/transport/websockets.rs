use super::{Message, Transport};
use super::Result;
use super::error::{TransportError, TransportErrorCode};
use actix_ws::{Message as WsMessage, Session};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use reqwest::header::{HeaderName, HeaderValue};
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message as TungsteniteMessage};
use tracing::{debug, info};

// Type aliases to simplify complex types
type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures::stream::SplitSink<WsStream, TungsteniteMessage>;
type MessageSender = broadcast::Sender<Message>;
type MessageReceiver = broadcast::Receiver<Message>;

#[derive(Clone)]
/// WebSocket transport implementation for the server side
pub struct ServerWsTransport {
    session: Arc<Mutex<Option<Session>>>,
    rx: Arc<Mutex<Option<broadcast::Receiver<Message>>>>,
    tx: Arc<Mutex<Option<broadcast::Sender<Message>>>>,
}

impl std::fmt::Debug for ServerWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerWsTransport")
            .field("session", &"<Session>")
            .field("rx", &self.rx)
            .field("tx", &self.tx)
            .finish()
    }
}

impl ServerWsTransport {
    /// Create a new server-side WebSocket transport
    ///
    /// # Arguments
    /// * `session` - The WebSocket session
    /// * `rx` - Channel receiver for incoming messages
    pub fn new(session: Session, rx: broadcast::Receiver<Message>) -> Self {
        // We need to create a new sender since we can't get it from the receiver
        let (tx, _) = broadcast::channel(100);

        Self {
            session: Arc::new(Mutex::new(Some(session))),
            rx: Arc::new(Mutex::new(Some(rx))),
            tx: Arc::new(Mutex::new(Some(tx))),
        }
    }

    /// Create a new server-side WebSocket transport with a new channel
    ///
    /// # Arguments
    /// * `session` - The WebSocket session
    /// * `capacity` - The capacity of the broadcast channel
    pub fn new_with_channel(session: Session, capacity: usize) -> (Self, broadcast::Sender<Message>) {
        let (tx, rx) = broadcast::channel(capacity);

        let transport = Self {
            session: Arc::new(Mutex::new(Some(session))),
            rx: Arc::new(Mutex::new(Some(rx))),
            tx: Arc::new(Mutex::new(Some(tx.clone()))),
        };

        (transport, tx)
    }
}

#[derive(Clone)]
/// WebSocket transport implementation for the client side
pub struct ClientWsTransport {
    ws_tx: Arc<Mutex<Option<MessageSender>>>,
    ws_rx: Arc<Mutex<Option<MessageReceiver>>>,
    url: String,
    headers: HashMap<String, String>,
    ws_write: Arc<Mutex<Option<WsSink>>>,
}

impl std::fmt::Debug for ClientWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientWsTransport")
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("ws_tx", &"<MessageSender>")
            .field("ws_rx", &"<MessageReceiver>")
            .field("ws_write", &"<WsSink>")
            .finish()
    }
}

impl ClientWsTransport {
    /// Create a builder for configuring and creating a client WebSocket transport
    ///
    /// # Arguments
    /// * `url` - The WebSocket server URL to connect to
    pub fn builder(url: String) -> ClientWsTransportBuilder {
        ClientWsTransportBuilder::new(url)
    }
}

/// Builder for configuring and creating a client WebSocket transport
pub struct ClientWsTransportBuilder {
    url: String,
    headers: HashMap<String, String>,
}

impl ClientWsTransportBuilder {
    /// Create a new transport builder
    ///
    /// # Arguments
    /// * `url` - The WebSocket server URL to connect to
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
        }
    }

    /// Add a custom header to the WebSocket connection
    ///
    /// # Arguments
    /// * `key` - Header name
    /// * `value` - Header value
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Build the client WebSocket transport with the configured options
    pub fn build(self) -> ClientWsTransport {
        ClientWsTransport {
            ws_tx: Arc::new(Mutex::new(None)),
            ws_rx: Arc::new(Mutex::new(None)),
            url: self.url,
            headers: self.headers,
            ws_write: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Transport for ServerWsTransport {
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx_guard = self.rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            match rx.recv().await {
                Ok(message) => {
                    debug!("Server received WebSocket message: {:?}", message);
                    Ok(Some(message))
                },
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("Server WebSocket channel closed");
                    // Channel is closed, clear our reference to it
                    *rx_guard = None;
                    Ok(None)
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We lagged behind, log a warning but continue
                    debug!("Server WebSocket channel lagged behind by {} messages", n);
                    // Try to receive again with a new subscription
                    if let Some(tx) = self.tx.lock().await.as_ref() {
                        *rx_guard = Some(tx.subscribe());
                        // Return None this time, the next call will get a message
                        Ok(None)
                    } else {
                        // No sender available, channel is effectively closed
                        *rx_guard = None;
                        Ok(None)
                    }
                }
            }
        } else {
            // No receiver available
            debug!("Server WebSocket receive called but no receiver is available");
            Ok(None)
        }
    }

    async fn send(&self, message: &Message) -> Result<()> {
        let mut session_guard = self.session.lock().await;
        if let Some(session) = session_guard.as_mut() {
            // Serialize the message to JSON
            let json = serde_json::to_string(message)
                .map_err(|e| TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("Failed to serialize message: {}", e)
                ))?;

            debug!("Server sending WebSocket message: {}", json);

            // Send the message
            match session.text(json).await {
                Ok(_) => {
                    debug!("Server successfully sent WebSocket message");
                    Ok(())
                },
                Err(e) => {
                    // If sending fails, the session is likely closed
                    debug!("Server failed to send WebSocket message: {}", e);
                    // Clear the session reference
                    *session_guard = None;
                    Err(TransportError::new(
                        TransportErrorCode::MessageSendFailed,
                        format!("Failed to send message: {}", e)
                    ))
                }
            }
        } else {
            debug!("Server attempted to send WebSocket message with no active session");
            Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "No active WebSocket session"
            ))
        }
    }

    async fn open(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        debug!("Closing server WebSocket connection");

        // Take the session to ensure we don't leave dangling references
        let mut session_guard = self.session.lock().await;
        if let Some(session) = session_guard.take() {
            // Create a normal closure frame
            let close_frame = actix_ws::CloseReason {
                code: actix_ws::CloseCode::Normal,
                description: Some("Server initiated close".to_string()),
            };

            // Send the close frame and ignore errors if the connection is already closed
            match session.close(Some(close_frame)).await {
                Ok(_) => {
                    debug!("Server WebSocket connection closed successfully");
                },
                Err(e) => {
                    debug!("Error closing server WebSocket connection (may already be closed): {}", e);
                }
            }
        } else {
            debug!("Server WebSocket connection already closed");
        }

        // Clear the broadcast channel references
        *self.rx.lock().await = None;
        *self.tx.lock().await = None;

        Ok(())
    }
}

#[async_trait]
impl Transport for ClientWsTransport {
    async fn receive(&self) -> Result<Option<Message>> {
        // Check if we have a valid receiver
        let mut rx_guard = self.ws_rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            match rx.recv().await {
                Ok(message) => {
                    debug!("WebSocket received message: {:?}", message);
                    Ok(Some(message))
                },
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("WebSocket channel closed");
                    // Channel is closed, clear our reference to it
                    *rx_guard = None;
                    Ok(None)
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We lagged behind, log a warning but continue
                    debug!("WebSocket channel lagged behind by {} messages", n);
                    // Try to receive again with a new subscription
                    if let Some(tx) = self.ws_tx.lock().await.as_ref() {
                        *rx_guard = Some(tx.subscribe());
                        // Return None this time, the next call will get a message
                        Ok(None)
                    } else {
                        // No sender available, channel is effectively closed
                        *rx_guard = None;
                        Ok(None)
                    }
                }
            }
        } else {
            // No receiver available
            debug!("WebSocket receive called but no receiver is available");
            Ok(None)
        }
    }

    async fn send(&self, message: &Message) -> Result<()> {
        let mut ws_write = self.ws_write.lock().await;
        if let Some(ws_write) = ws_write.as_mut() {
            let json = serde_json::to_string(message)?;
            ws_write
                .send(TungsteniteMessage::Text(json.into()))
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::SendError, e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::SendError,
                "No active WebSocket connection",
            ))
        }
    }

    async fn open(&self) -> Result<()> {
        // Check if we're already open
        if self.ws_write.lock().await.is_some() {
            debug!("WebSocket connection already open");
            return Ok(());
        }

        debug!("Opening WebSocket connection to {}", self.url);

        // Prepare the request with headers
        let mut request = self.url.as_str().into_client_request()
            .map_err(|e| TransportError::new(
                TransportErrorCode::OpenError,
                format!("Invalid WebSocket URL: {}", e)
            ))?;

        // Add headers
        for (key, value) in &self.headers {
            request.headers_mut().insert(
                HeaderName::from_str(key).map_err(|e| {
                    TransportError::new(TransportErrorCode::OpenError, format!("Invalid header key: {}", e))
                })?,
                HeaderValue::from_str(value).map_err(|e| {
                    TransportError::new(
                        TransportErrorCode::OpenError,
                        format!("Invalid header value: {}", e),
                    )
                })?,
            );
        }

        // Connect to the WebSocket server with timeout
        let connect_future = tokio_tungstenite::connect_async(request);
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second timeout
            connect_future
        ).await;

        // Handle timeout
        let connect_result = match connect_result {
            Ok(result) => result,
            Err(_) => return Err(TransportError::new(
                TransportErrorCode::ConnectionTimeout,
                "WebSocket connection timed out after 30 seconds"
            )),
        };

        // Handle connection errors
        let (ws_stream, response) = connect_result
            .map_err(|e| TransportError::new(
                TransportErrorCode::ConnectionFailed,
                format!("WebSocket connection failed: {}", e)
            ))?;

        // Log successful connection
        debug!("WebSocket connection established with status: {}", response.status());

        // Split the WebSocket stream
        let (write, mut read) = ws_stream.split();

        // Create broadcast channel for message distribution
        // Increase buffer size to 1000 to handle more messages
        let (tx, rx) = broadcast::channel(1000);

        // Store the sender, receiver, and write half
        *self.ws_tx.lock().await = Some(tx.clone());
        *self.ws_rx.lock().await = Some(rx);
        *self.ws_write.lock().await = Some(write);

        // Spawn a task to handle incoming messages
        let tx = tx.clone();
        let url = self.url.clone(); // Clone URL for the task
        tokio::spawn(async move {
            debug!("Starting WebSocket message handler for {}", url);

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(TungsteniteMessage::Text(text)) => {
                        match serde_json::from_str::<Message>(&text) {
                            Ok(message) => {
                                debug!("Received WebSocket message: {:?}", message);
                                if tx.send(message).is_err() {
                                    debug!("All receivers dropped, stopping message handling");
                                    break;
                                }
                            },
                            Err(e) => {
                                debug!("Failed to parse WebSocket message: {}", e);
                                debug!("Message content: {}", text);
                                // Continue processing other messages
                            }
                        }
                    },
                    Ok(TungsteniteMessage::Binary(data)) => {
                        debug!("Received binary WebSocket message of {} bytes", data.len());
                        // We don't handle binary messages currently
                    },
                    Ok(TungsteniteMessage::Ping(_)) => {
                        debug!("Received WebSocket ping");
                        // The WebSocket library automatically responds with pong
                    },
                    Ok(TungsteniteMessage::Pong(_)) => {
                        // Ignore pong messages
                    },
                    Ok(TungsteniteMessage::Close(frame)) => {
                        if let Some(frame) = frame {
                            info!("WebSocket connection closed by server: {} - {}",
                                  frame.code, frame.reason);
                        } else {
                            info!("WebSocket connection closed by server");
                        }
                        break;
                    },
                    Ok(TungsteniteMessage::Frame(_)) => {
                        // Raw frames are not expected in normal operation
                        debug!("Received raw WebSocket frame");
                    },
                    Err(e) => {
                        debug!("Error reading from WebSocket: {}", e);
                        break;
                    }
                }
            }

            debug!("WebSocket message handler for {} terminated", url);
        });

        debug!("WebSocket connection setup complete");
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Take the write half of the WebSocket to ensure we don't leave dangling references
        if let Some(mut write) = self.ws_write.lock().await.take() {
            // Send a close frame with normal closure status
            let close_frame = tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: "Client initiated close".into(),
            };

            // Send the close frame and ignore errors if the connection is already closed
            if let Err(e) = write.send(TungsteniteMessage::Close(Some(close_frame))).await {
                debug!("Error sending close frame (connection may already be closed): {}", e);
            }

            // Flush any pending messages
            if let Err(e) = write.flush().await {
                debug!("Error flushing WebSocket stream: {}", e);
            }
        }

        // Clear the broadcast channels to prevent memory leaks
        *self.ws_tx.lock().await = None;
        *self.ws_rx.lock().await = None;

        Ok(())
    }
}

/// Handle a WebSocket connection, managing message flow between client and server
///
/// This function sets up bidirectional communication between a WebSocket connection
/// and broadcast channels for message passing.
///
/// # Arguments
/// * `session` - The WebSocket session
/// * `stream` - Stream of incoming WebSocket messages
/// * `tx` - Channel sender for outgoing messages
/// * `rx` - Channel receiver for incoming messages
///
/// # Returns
/// * `Result<()>` - Ok if the connection was handled successfully, Err otherwise
pub async fn handle_ws_connection(
    mut session: Session,
    mut stream: actix_ws::MessageStream,
    tx: broadcast::Sender<Message>,
    mut rx: broadcast::Receiver<Message>,
) -> Result<()> {
    debug!("Starting WebSocket connection handler");

    // Send messages from rx to the WebSocket
    let mut send_task = actix_web::rt::spawn(async move {
        debug!("Starting WebSocket send task");

        while let Ok(message) = rx.recv().await {
            debug!("Sending message to WebSocket: {:?}", message);

            match serde_json::to_string(&message) {
                Ok(json) => {
                    if let Err(e) = session.text(json).await {
                        debug!("Error sending message to WebSocket: {}", e);
                        break;
                    }
                },
                Err(e) => {
                    debug!("Error serializing message to JSON: {}", e);
                    continue;
                }
            }
        }

        debug!("WebSocket send task completed");
        Ok::<_, anyhow::Error>(())
    });

    // Receive messages from the WebSocket and send them to tx
    let mut recv_task = actix_web::rt::spawn(async move {
        debug!("Starting WebSocket receive task");

        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(WsMessage::Text(text)) => {
                    debug!("Received text message from WebSocket: {}", text);

                    match serde_json::from_str::<Message>(&text) {
                        Ok(message) => {
                            debug!("Parsed message: {:?}", message);
                            if tx.send(message).is_err() {
                                debug!("Error sending message to channel (no receivers)");
                                break;
                            }
                        },
                        Err(e) => {
                            debug!("Error parsing message from WebSocket: {}", e);
                            // Continue processing other messages
                        }
                    }
                },
                Ok(WsMessage::Binary(bytes)) => {
                    debug!("Received binary message from WebSocket ({} bytes)", bytes.len());
                    // We don't handle binary messages currently
                },
                Ok(WsMessage::Ping(_)) => {
                    debug!("Received ping from WebSocket");
                    // Handled automatically by actix-ws
                },
                Ok(WsMessage::Pong(_)) => {
                    // Ignore pong messages
                },
                Ok(WsMessage::Close(reason)) => {
                    if let Some(reason) = reason {
                        debug!("WebSocket closed by client: {:?} - {}", reason.code, reason.description.unwrap_or_default());
                    } else {
                        debug!("WebSocket closed by client");
                    }
                    break;
                },
                Ok(WsMessage::Continuation(_)) => {
                    debug!("Received continuation frame from WebSocket");
                    // We don't handle continuation frames explicitly
                },
                Ok(WsMessage::Nop) => {
                    // No operation, ignore
                },
                Err(e) => {
                    debug!("Error receiving message from WebSocket: {}", e);
                    break;
                }
            }
        }

        debug!("WebSocket receive task completed");
        Ok::<_, anyhow::Error>(())
    });

    // Wait for either task to complete
    let result = tokio::select! {
        res = (&mut send_task) => match res {
            Ok(Ok(())) => {
                debug!("Send task completed successfully");
                Ok(())
            },
            Ok(Err(e)) => {
                debug!("Send task failed: {}", e);
                Err(TransportError::new(
                    TransportErrorCode::SendError,
                    format!("Send task failed: {}", e)
                ))
            },
            Err(e) => {
                debug!("Send task join error: {}", e);
                Err(TransportError::new(
                    TransportErrorCode::SendError,
                    format!("Send task join error: {}", e)
                ))
            },
        },
        res = (&mut recv_task) => match res {
            Ok(Ok(())) => {
                debug!("Receive task completed successfully");
                Ok(())
            },
            Ok(Err(e)) => {
                debug!("Receive task failed: {}", e);
                Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Receive task failed: {}", e)
                ))
            },
            Err(e) => {
                debug!("Receive task join error: {}", e);
                Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Receive task join error: {}", e)
                ))
            },
        },
    };

    // Cancel the other task if one completes
    send_task.abort();
    recv_task.abort();

    debug!("WebSocket connection handler completed");
    result
}
