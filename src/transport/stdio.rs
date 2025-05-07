use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use crate::transport::{Transport, Message, Result, TransportError, TransportErrorCode};

/// Transport implementation for communicating with a child process via stdin/stdout
///
/// This transport allows bidirectional communication with a child process by
/// reading from its stdout and writing to its stdin.
pub struct StdioTransport {
    /// Reader for the child process's stdout
    reader: Mutex<BufReader<ChildStdout>>,
    /// Writer for the child process's stdin
    writer: Mutex<ChildStdin>,
    /// Flag to track if the transport is open
    is_open: Arc<AtomicBool>,
    /// Buffer size for reading lines
    buffer_size: usize,
}

impl StdioTransport {
    /// Creates a new stdio transport
    ///
    /// # Arguments
    /// * `stdout` - The child process's stdout
    /// * `stdin` - The child process's stdin
    ///
    /// # Returns
    /// A new StdioTransport instance
    pub fn new(stdout: ChildStdout, stdin: ChildStdin) -> Self {
        Self {
            reader: Mutex::new(BufReader::new(stdout)),
            writer: Mutex::new(stdin),
            is_open: Arc::new(AtomicBool::new(true)),
            buffer_size: 64 * 1024, // 64KB buffer size by default
        }
    }

    /// Creates a new stdio transport with a custom buffer size
    ///
    /// # Arguments
    /// * `stdout` - The child process's stdout
    /// * `stdin` - The child process's stdin
    /// * `buffer_size` - The buffer size for reading lines
    ///
    /// # Returns
    /// A new StdioTransport instance
    pub fn with_buffer_size(stdout: ChildStdout, stdin: ChildStdin, buffer_size: usize) -> Self {
        Self {
            reader: Mutex::new(BufReader::new(stdout)),
            writer: Mutex::new(stdin),
            is_open: Arc::new(AtomicBool::new(true)),
            buffer_size,
        }
    }

    /// Checks if the transport is open
    ///
    /// # Returns
    /// `true` if the transport is open, `false` otherwise
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Sets the open state of the transport
    ///
    /// # Arguments
    /// * `open` - The new open state
    pub fn set_open(&self, open: bool) {
        self.is_open.store(open, Ordering::Relaxed);
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: &Message) -> Result<()> {
        // Check if the transport is open
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "Stdio transport is closed"
            ));
        }

        // Serialize the message to JSON
        let message_str = serde_json::to_string(message)
            .map_err(|e| TransportError::new(
                TransportErrorCode::MessageSendFailed,
                format!("Failed to serialize message: {}", e)
            ))?;

        // Send the message to the child process's stdin
        let mut writer = self.writer.lock().await;

        // Write the message, followed by a newline
        match writer.write_all(message_str.as_bytes()).await {
            Ok(_) => {},
            Err(e) => {
                // If writing fails, mark the transport as closed
                self.set_open(false);
                return Err(TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("Failed to write message: {}", e)
                ));
            }
        }

        // Write a newline to terminate the message
        match writer.write_all(b"\n").await {
            Ok(_) => {},
            Err(e) => {
                // If writing fails, mark the transport as closed
                self.set_open(false);
                return Err(TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("Failed to write newline: {}", e)
                ));
            }
        }

        // Flush the writer to ensure the message is sent
        match writer.flush().await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If flushing fails, mark the transport as closed
                self.set_open(false);
                Err(TransportError::new(
                    TransportErrorCode::MessageSendFailed,
                    format!("Failed to flush writer: {}", e)
                ))
            }
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        // Check if the transport is open
        if !self.is_open() {
            return Err(TransportError::new(
                TransportErrorCode::ConnectionClosed,
                "Stdio transport is closed"
            ));
        }

        // Allocate a buffer for the message with the configured buffer size
        let mut line = String::with_capacity(self.buffer_size);

        // Lock the reader
        let mut reader = self.reader.lock().await;

        // Read a line from the child process's stdout
        let bytes_read = match reader.read_line(&mut line).await {
            Ok(bytes) => bytes,
            Err(e) => {
                // If reading fails, mark the transport as closed
                self.set_open(false);
                return Err(TransportError::new(
                    TransportErrorCode::MessageReceiveFailed,
                    format!("Failed to read line: {}", e)
                ));
            }
        };

        // If we read 0 bytes, the stream is closed
        if bytes_read == 0 {
            // Mark the transport as closed
            self.set_open(false);
            return Ok(None);
        }

        // Parse the message from JSON
        match serde_json::from_str::<Message>(line.trim()) {
            Ok(message) => Ok(Some(message)),
            Err(e) => Err(TransportError::new(
                TransportErrorCode::InvalidMessage,
                format!("Failed to parse message: {}", e)
            ))
        }
    }

    async fn open(&self) -> Result<()> {
        // Mark the transport as open
        self.set_open(true);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Mark the transport as closed
        self.set_open(false);

        // We don't actually close the stdin/stdout handles here
        // They will be closed when the child process terminates
        // or when the StdioTransport is dropped
        Ok(())
    }
}
