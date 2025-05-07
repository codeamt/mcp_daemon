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

OR

Run `cargo add mcp_daemon`

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
- **transport**: Transport layer implementations (HTTP/2, WebSockets, etc.)

## TLS Configuration

### Server TLS Configuration

#### Self-Signed Certificates for Development

For local development, you can generate self-signed certificates using the included script:

```bash
# Generate certificates for localhost
./scripts/generate_cert.sh

# Generate certificates with custom options
./scripts/generate_cert.sh --dir my_certs --domain example.com --days 730
```

Then use the certificates in your server configuration:

```rust
use mcp_daemon::transport::http2::{Http2ServerConfig, TlsConfig};
use std::net::SocketAddr;

let config = Http2ServerConfig {
    addr: "127.0.0.1:8443".parse().unwrap(),
    tls_config: Some(TlsConfig::Manual {
        cert_path: "certs/localhost.example.crt".to_string(),
        key_path: "certs/localhost.example.key".to_string(),
    }),
};
```

#### Let's Encrypt Integration

For production environments, you can use Let's Encrypt for automatic certificate management by enabling the `acme` feature:

```toml
[dependencies]
mcp_daemon = { version = "0.3.0", features = ["acme"] }
```

Then configure your server to use ACME:

```rust
use mcp_daemon::transport::http2::{Http2ServerConfig, TlsConfig};
use std::net::SocketAddr;
use std::path::PathBuf;

let config = Http2ServerConfig {
    addr: "0.0.0.0:443".parse().unwrap(),
    tls_config: Some(TlsConfig::Acme {
        domains: vec!["example.com".to_string()],
        contact_email: "admin@example.com".to_string(),
        cache_dir: Some(PathBuf::from(".certificates")),
        use_staging: false, // Set to true for testing
    }),
};
```

### Client TLS Configuration

The client supports various TLS configurations for secure connections to MCP servers:

#### Basic TLS (System Root Certificates)

```rust
use mcp_daemon::transport::http::Http2Builder;

// Create a client with TLS enabled (using system root certificates)
let transport = Http2Builder::new()
    .with_tls(true)
    .with_host("example.com".to_string())
    .with_port(443)
    .build()?;
```

#### Custom Root Certificate

```rust
use mcp_daemon::transport::http::Http2Builder;

// Create a client with a custom root certificate
let transport = Http2Builder::new()
    .with_custom_tls("path/to/root.crt".to_string(), true)
    .with_host("example.com".to_string())
    .with_port(443)
    .build()?;
```

#### Client Certificate (Mutual TLS)

```rust
use mcp_daemon::transport::http::Http2Builder;

// Create a client with a client certificate for mutual TLS
let transport = Http2Builder::new()
    .with_custom_tls("path/to/root.crt".to_string(), true)
    .with_client_cert("path/to/client.crt".to_string(), "path/to/client.key".to_string())
    .with_host("example.com".to_string())
    .with_port(443)
    .build()?;
```

#### Server Name Indication (SNI)

```rust
use mcp_daemon::transport::http::Http2Builder;

// Create a client with SNI
let transport = Http2Builder::new()
    .with_custom_tls("path/to/root.crt".to_string(), true)
    .with_sni("example.com".to_string())
    .with_host("192.168.1.100".to_string()) // IP address
    .with_port(443)
    .build()?;
```

> **Note**: The client TLS implementation is currently in development. While the API is in place, some advanced features like client certificates and SNI may not be fully functional in all scenarios. We're actively working on improving this and welcome feedback from users. Please see our [GitHub Discussions](https://github.com/entrepeneur4lyf/mcp_daemon/discussions/2) for more information and to provide input.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Resources

- [Model Context Protocol Specification](https://spec.modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
