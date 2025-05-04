# MCP Daemon: A Rust implementation of the Model Context Protocol (MCP)

[![Crates.io](https://img.shields.io/crates/v/mcp_daemon.svg)](https://crates.io/crates/mcp_daemon)
[![Documentation](https://docs.rs/mcp_daemon/badge.svg)](https://docs.rs/mcp_daemon)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

This crate provides a standards-compliant implementation of the [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/), enabling seamless integration between LLM applications and external data sources and tools.

## Overview

The Model Context Protocol (MCP) is a standardized protocol for communication between LLM applications and external systems. It allows LLM applications to access external data sources, tools, and services in a consistent and standardized way.

This implementation includes both client and server components, along with the necessary schema definitions and utilities for working with the protocol.

## Features

- **Client Implementation**: Connect to MCP servers and access their resources and tools
- **Server Implementation**: Create an MCP server to expose resources and tools to LLM applications
- **Schema Definitions**: Complete schema definitions for the MCP protocol ([03.26.2025](https://spec.modelcontextprotocol.io/specification/2025-03-26/))
- **Error Handling**: Comprehensive error handling for all protocol operations
- **Async Support**: Built on top of the async ecosystem for efficient operation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mcp_daemon = "0.3.0"
```

## Usage

### Client Example

```rust
use mcp_daemon::client::Client;
use mcp_daemon::schema::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = Client::new("http://localhost:8080");

    // Initialize the client
    let init_result = client.initialize().await?;
    println!("Connected to server: {}", init_result.server_info.name);

    // List available tools
    let tools = client.tools_list(ListToolsRequestParams::default()).await?;
    println!("Available tools: {}", tools.tools.len());

    Ok(())
}
```

### Server Example

```rust
use mcp_daemon::server::{Server, DefaultServer};
use mcp_daemon::schema::*;
use std::sync::Arc;

struct MyServer;

impl Server for MyServer {
    // Implement required methods
    // ...
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a server
    let server = Arc::new(MyServer);

    // Start the server
    let addr = "127.0.0.1:8080";
    println!("Starting server on {}", addr);
    // server.listen(addr).await?;

    Ok(())
}
```

## Documentation

The full API documentation is available at [docs.rs/mcp_daemon](https://docs.rs/mcp_daemon).

You can also build the documentation locally:

```bash
cargo doc --open
```

## Key Modules

- **client**: Client implementation for connecting to MCP servers
- **server**: Server implementation for creating MCP servers
- **schema**: Schema definitions for the MCP protocol
- **error**: Error handling for the MCP protocol
- **utility**: Utility functions, macros, and types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Resources

- [Model Context Protocol Specification](https://spec.modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
