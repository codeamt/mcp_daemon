//! Utility functions and macros for MCP daemon implementation.
//!
//! This module contains utility functions and macros that are used throughout the
//! MCP daemon implementation. The primary functionality provided is the `#[server]`
//! macro for implementing the `Daemon` trait, which greatly simplifies the creation
//! of MCP-compliant servers.
//!
//! ## Contents
//!
//! - `macros`: Contains the `#[server]` attribute macro and related macros for
//!   implementing MCP protocol server components like prompts, resources, and tools.

pub mod macros;
pub use macros::*;
