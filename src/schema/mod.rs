//! Schema definitions for the Model Context Protocol (MCP).
//!
//! This module contains all the type definitions and structures required to work
//! with the MCP protocol. It's organized into several submodules:
//!
//! - `types`: Core type definitions for MCP requests and responses
//! - `types_ex`: Extended types with additional functionality 
//! - `schema_ext`: Extensions and convenience methods for core schema types
//! - `default_impls`: Default implementations for schema types
//! - `protocol`: Protocol-specific constants and definitions
//! - `annotations`: Type definitions for resource and template annotations
//!
//! Most types from `types`, `types_ex` and `annotations` are re-exported at the schema module
//! level for convenience.

pub mod schema;
pub mod schema_ext;
pub mod types_ex;
pub mod default_impls;
pub mod protocol;
pub mod annotations;

pub use schema::*;
pub use types_ex::*;
pub use annotations::*;
