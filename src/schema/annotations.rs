//! Annotation types for resources and resource templates.
//!
//! This module provides annotation types used in the MCP protocol for adding
//! metadata to resources and resource templates. Annotations are key-value pairs
//! that can be used to store additional information about resources and templates
//! that is not part of the core MCP protocol.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Annotations for resources in the MCP protocol.
///
/// This struct represents a collection of key-value annotations that can be attached
/// to resources in the MCP protocol. These annotations can be used to provide additional
/// metadata about a resource, such as its origin, author, version, or any other custom
/// information that might be useful for clients.
///
/// # Examples
///
/// ```no_run
/// use std::collections::HashMap;
/// use mcp_daemon::schema::ResourceAnnotations;
///
/// // Create annotations directly
/// let mut annotations = ResourceAnnotations::default();
/// annotations.custom.insert("author".to_string(), "John Doe".to_string());
/// annotations.custom.insert("version".to_string(), "1.0".to_string());
///
/// // Or create from an existing HashMap
/// let mut custom = HashMap::new();
/// custom.insert("author".to_string(), "John Doe".to_string());
/// custom.insert("version".to_string(), "1.0".to_string());
/// let annotations = ResourceAnnotations::from(custom);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceAnnotations {
    /// Custom annotations as key-value pairs.
    ///
    /// These are flattened in the JSON representation, meaning they appear as top-level
    /// properties rather than nested under a "custom" field.
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl From<HashMap<String, String>> for ResourceAnnotations {
    /// Creates `ResourceAnnotations` from a HashMap of annotations.
    ///
    /// This implementation allows for easy conversion from a simple HashMap
    /// to structured ResourceAnnotations, which is useful when you already
    /// have annotations in a HashMap format.
    ///
    /// # Parameters
    ///
    /// * `custom` - A HashMap containing the custom annotations as key-value pairs
    ///
    /// # Returns
    ///
    /// A new `ResourceAnnotations` instance containing the provided annotations
    fn from(custom: HashMap<String, String>) -> Self {
        Self { custom }
    }
}

/// Annotations for resource templates in the MCP protocol.
///
/// This struct represents a collection of key-value annotations that can be attached
/// to resource templates in the MCP protocol. These annotations can be used to provide
/// additional metadata about a template, such as its purpose, expected usage, or any
/// other custom information that might be useful for clients.
///
/// Resource templates are patterns that can be used to generate resource URIs, and
/// their annotations can help clients understand how to use them correctly.
///
/// # Examples
///
/// ```no_run
/// use std::collections::HashMap;
/// use mcp_daemon::schema::ResourceTemplateAnnotations;
///
/// // Create annotations directly
/// let mut annotations = ResourceTemplateAnnotations::default();
/// annotations.custom.insert("purpose".to_string(), "File access".to_string());
/// annotations.custom.insert("example".to_string(), "my_app://files/example.txt".to_string());
///
/// // Or create from an existing HashMap
/// let mut custom = HashMap::new();
/// custom.insert("purpose".to_string(), "File access".to_string());
/// custom.insert("example".to_string(), "my_app://files/example.txt".to_string());
/// let annotations = ResourceTemplateAnnotations::from(custom);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceTemplateAnnotations {
    /// Custom annotations as key-value pairs.
    ///
    /// These are flattened in the JSON representation, meaning they appear as top-level
    /// properties rather than nested under a "custom" field.
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl From<HashMap<String, String>> for ResourceTemplateAnnotations {
    /// Creates `ResourceTemplateAnnotations` from a HashMap of annotations.
    ///
    /// This implementation allows for easy conversion from a simple HashMap
    /// to structured ResourceTemplateAnnotations, which is useful when you already
    /// have annotations in a HashMap format.
    ///
    /// # Parameters
    ///
    /// * `custom` - A HashMap containing the custom annotations as key-value pairs
    ///
    /// # Returns
    ///
    /// A new `ResourceTemplateAnnotations` instance containing the provided annotations
    fn from(custom: HashMap<String, String>) -> Self {
        Self { custom }
    }
}
