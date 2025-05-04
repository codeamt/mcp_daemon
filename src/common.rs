//! Common utilities and re-exports for the MCP protocol implementation.
//!
//! This module contains common utilities and type re-exports that are used across
//! both the client and server implementations of the MCP protocol. It provides a
//! convenient way to access commonly used components without having to import them
//! from their original modules.

/// Re-export CancellationHook from request module with a more specific name.
///
/// The `McpCancellationHook` is a jsoncall `Hook` implementation that handles
/// request cancellation in the MCP protocol. When a request is cancelled, it
/// sends a `notifications/cancelled` notification to inform clients.
///
/// This hook is used by both client and server implementations to ensure proper
/// handling of request cancellations according to the MCP protocol specification.
pub use crate::request::session::CancellationHook as McpCancellationHook;
