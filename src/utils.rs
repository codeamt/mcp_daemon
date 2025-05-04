//! Utility types and re-exports for the MCP protocol implementation.
//!
//! This module contains utility types and re-exports that are commonly used
//! throughout the MCP daemon implementation. It provides a convenient way to access
//! frequently used types without having to import them from their original modules.

/// Re-export Empty and ProtocolVersion from schema module.
///
/// - `Empty`: A type representing an empty response or value in the MCP protocol.
///   This is used for methods that don't return any meaningful data but need to
///   conform to the JSON-RPC response format.
///
/// - `ProtocolVersion`: A type representing the version of the MCP protocol being used.
///   This is used during initialization to ensure compatibility between client and server.
pub use crate::schema::types_ex::{Empty, ProtocolVersion};
