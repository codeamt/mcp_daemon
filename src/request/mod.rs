//! Request handling module for the Model Context Protocol (MCP)
//!
//! This module provides utilities for handling JSON-RPC requests in the MCP protocol.
//! It includes components for session management, request cancellation, and other
//! request-related functionality required by MCP protocol implementations.
//!
//! The main components of this module include:
//!
//! - `session`: Utilities for handling session-related aspects of the protocol, such
//!   as request cancellation via the `CancellationHook`.
//!
//! These utilities are designed to be used alongside the jsoncall library to implement
//! MCP-compliant servers and clients.

pub mod session;
