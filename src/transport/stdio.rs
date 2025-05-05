use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use crate::Result;

pub struct StdioTransport {
    reader: BufReader<ChildStdout>, // Reading from the child process's stdout
    writer: ChildStdin, // Writing to the child process's stdin
}

impl StdioTransport {
    pub fn new(stdout: ChildStdout, stdin: ChildStdin) -> Self {
        Self {
            reader: BufReader::new(stdout),
            writer: stdin,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: &str) -> Result<()> {
        // Send message to the child process's stdin
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        // Receive message from the child process's stdout
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            Ok(None)
        } else {
            Ok(Some(line.trim().to_string()))
        }
    }

    async fn perform_auth(&self) -> Result<Option<()>> {
        // Keypair authentication is not applicable to stdio
        Ok(None)
    }
}
