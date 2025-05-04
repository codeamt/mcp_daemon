//! Error handling for the MCP protocol
//!
//! This module provides error types and utilities for handling errors in the
//! MCP protocol. It includes functions for creating standardized error
//! responses for common error conditions such as missing resources, prompts,
//! or tools.
//!
//! The error handling is built on top of the jsoncall library's error system,
//! which provides JSON-RPC 2.0 compliant error responses.

pub mod types;

// Re-export error utility functions for easier access
pub use types::{prompt_not_found, resource_not_found, resource_template_not_found, tool_not_found, invalid_request};

// Re-export the old types for backward compatibility
#[deprecated(since = "0.3.0", note = "Import directly from error module or error::types instead")]
pub mod compat {
    pub use super::types::*;
}

// Make the types available at the error::error path for backward compatibility
pub use self::types as error;
