//! Default implementations for MCP schema types.
//!
//! This module provides default implementations for various result types used in the Model
//! Context Protocol. These default implementations ensure that results can be created with
//! empty collections and default values when needed.
//!
//! All result types implement the `Default` trait, allowing them to be constructed with
//! `Type::default()` or using the default syntax in struct initialization.

use std::default::Default;
use serde_json::Map;

use super::schema::{
    CompleteResult, CompleteResultCompletion, ListPromptsResult, ListResourcesResult,
    ListResourceTemplatesResult, ListToolsResult,
};

/// Default implementation for [`ListPromptsResult`].
///
/// Creates a result with:
/// - An empty vector of prompts
/// - Empty metadata
/// - No next cursor (pagination is not needed for an empty result)
///
/// This is used for servers that don't implement custom prompt handling
/// or when creating a base result to be modified.
 impl Default for ListPromptsResult {
    fn default() -> Self {
        Self {
            prompts: Vec::new(),
            meta: Map::new(),
            next_cursor: None,
        }
    }
}

/// Default implementation for [`ListResourcesResult`].
///
/// Creates a result with:
/// - An empty vector of resources
/// - Empty metadata
/// - No next cursor (pagination is not needed for an empty result)
///
/// This is used for servers that don't implement custom resource handling
/// or when creating a base result to be modified.
impl Default for ListResourcesResult {
    fn default() -> Self {
        Self {
            resources: Vec::new(),
            meta: Map::new(),
            next_cursor: None,
        }
    }
}

/// Default implementation for [`ListResourceTemplatesResult`].
///
/// Creates a result with:
/// - An empty vector of resource templates
/// - Empty metadata
/// - No next cursor (pagination is not needed for an empty result)
///
/// This is used for servers that don't implement custom resource template handling
/// or when creating a base result to be modified.
impl Default for ListResourceTemplatesResult {
    fn default() -> Self {
        Self {
            resource_templates: Vec::new(),
            meta: Map::new(),
            next_cursor: None,
        }
    }
}

/// Default implementation for [`ListToolsResult`].
///
/// Creates a result with:
/// - An empty vector of tools
/// - Empty metadata
/// - No next cursor (pagination is not needed for an empty result)
///
/// This is used for servers that don't implement custom tool handling
/// or when creating a base result to be modified.
impl Default for ListToolsResult {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            meta: Map::new(),
            next_cursor: None,
        }
    }
}

/// Default implementation for [`CompleteResultCompletion`].
///
/// Creates a completion result with:
/// - An empty vector of completion values
/// - `has_more` set to `false` (no additional completions available)
/// - `total` set to `0` (no completions in total)
///
/// This is used as part of the default implementation for [`CompleteResult`],
/// or when creating a base completion result to be modified.
impl Default for CompleteResultCompletion {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            has_more: Some(false),
            total: Some(0),
        }
    }
}

/// Default implementation for [`CompleteResult`].
///
/// Creates a result with:
/// - A default completion (see [`CompleteResultCompletion::default`])
/// - Empty metadata
///
/// This is used for servers that don't implement completion handling
/// or when creating a base result to be modified.
impl Default for CompleteResult {
    fn default() -> Self {
        Self {
            completion: CompleteResultCompletion::default(),
            meta: Map::new(),
        }
    }
}

// Note: TextResourceContents and BlobResourceContents already have Default trait
// implementations derived via #[derive(Default)]. The derived implementation
// will correctly set:
// - text/blob fields to empty strings
// - mime_type to None
// - uri to empty string (for TextResourceContents and BlobResourceContents)
