#![doc(test(attr(exclude(tests))))]

//! # MCP Daemon: A Rust implementation of the Model Context Protocol (MCP)
//!
//! This crate provides a standards-compliant implementation of the [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/),
//! enabling seamless integration between LLM applications and external data sources and tools.
//!
//! ## Overview
//!
//! The Model Context Protocol (MCP) is a standardized protocol for communication between
//! LLM applications and external systems. It allows LLM applications to access external
//! data sources, tools, and services in a consistent and standardized way.
//!
//! This implementation includes both client and server components, along with the necessary
//! schema definitions and utilities for working with the protocol.
//!
//! ## Features
//!
//! - **Client Implementation**: Connect to MCP servers and access their resources and tools
//! - **Server Implementation**: Create an MCP server to expose resources and tools to LLM applications
//! - **Schema Definitions**: Complete schema definitions for the MCP protocol
//! - **Error Handling**: Comprehensive error handling for all protocol operations
//! - **Async Support**: Built on top of the async ecosystem for efficient operation
//!
//! ## Usage
//!
//! ### Client Example
//!
//! ```rust,ignore
//! use mcp_daemon::client::Client;
//! use mcp_daemon::schema::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client
//!     let client = Client::new("http://localhost:8080");
//!
//!     // Initialize the client
//!     let init_result = client.initialize().await?;
//!     println!("Connected to server: {}", init_result.server_info.name);
//!
//!     // List available tools
//!     let tools = client.tools_list(ListToolsRequestParams::default()).await?;
//!     println!("Available tools: {}", tools.tools.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Server Example
//!
//! ```rust,ignore
//! use mcp_daemon::server::{Server, DefaultServer};
//! use mcp_daemon::schema::*;
//! use std::sync::Arc;
//!
//! struct MyServer;
//!
//! impl Server for MyServer {
//!     // Implement required methods
//!     // ...
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a server
//!     let server = Arc::new(MyServer);
//!
//!     // Start the server
//!     let addr = "127.0.0.1:8080";
//!     println!("Starting server on {}", addr);
//!     // server.listen(addr).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules

/// Client implementation for connecting to MCP servers
///
/// This module provides the `Client` struct and related functionality for
/// connecting to MCP servers and accessing their resources and tools.
pub mod client;

/// Server implementation for creating MCP servers
///
/// This module provides the `Server` trait and related functionality for
/// creating MCP servers that expose resources and tools to LLM applications.
pub mod server;

/// Request handling and session management
///
/// This module provides functionality for handling requests and managing sessions
/// in the MCP protocol.
pub mod request;

/// Schema definitions for the MCP protocol
///
/// This module provides the complete schema definitions for the MCP protocol,
/// including all message types, parameters, and results.
pub mod schema;

/// Error handling for the MCP protocol
///
/// This module provides error types and utilities for handling errors in the
/// MCP protocol.
pub mod error;

/// Utility functions, macros, and types
///
/// This module provides utility functions, macros, and types for working with the
/// MCP protocol.
pub mod utility;

/// Common utilities and types
///
/// This module provides common utilities and types used throughout the crate.
pub mod common;

/// Utility functions and types
///
/// This module provides utility functions and types for working with the
/// MCP protocol.
pub mod utils;

// Re-export dependencies and common types

/// Re-export of the jsoncall crate
///
/// This provides the core JSON-RPC 2.0 functionality used by the MCP protocol.
pub use jsoncall;

/// Error type for MCP operations
pub use jsoncall::Error;

/// Error codes for MCP operations
pub use jsoncall::ErrorCode;

/// Result type for MCP operations
pub use jsoncall::Result;

/// Session error type for MCP operations
pub use jsoncall::SessionError;

/// Session result type for MCP operations
pub use jsoncall::SessionResult;

/// Macro for returning an error from a function
pub use jsoncall::bail;

/// Macro for returning a public error from a function
pub use jsoncall::bail_public;
