//! Extensions and convenience methods for MCP schema types.
//!
//! This module provides extensions to the core MCP schema types defined in `schema.rs`.
//! These extensions include:
//!
//! - Conversion implementations between related types (`From` trait implementations)
//! - Builder-style methods for constructing complex types
//! - Utility methods for working with MCP schema types
//! - Helpers for common operations like file path handling and Base64 encoding
//!
//! These extensions make it easier to work with the MCP protocol in idiomatic Rust
//! by providing ergonomic APIs for constructing request parameters and handling responses.

#![allow(missing_docs)]

use base64::Engine;
use jsoncall::{ErrorCode, bail_public};
use schemars::{JsonSchema, schema::Metadata, schema_for};
use serde::Serialize;
use serde_json::{Value, to_value};
use std::collections::HashMap;
use url::Url;

use crate::{
    Result,
    schema::{
        CallToolRequestParams, CompleteRequestParams, CompleteRequestParamsArgument, CompleteRequestParamsRef,
        CompleteResult, CompleteResultCompletion, EmbeddedResource, EmbeddedResourceResource,
        GetPromptResult, ImageContent, Implementation, ListPromptsResult, ListResourceTemplatesResult, 
        ListResourcesResult, ListRootsResult, ListToolsResult, Prompt, PromptMessage, 
        PromptMessageContent, PromptReference, ReadResourceResult, ReadResourceResultContentsItem, Resource,
        ResourceReference, ResourceTemplate, Role, Root, TextContent, Tool, ToolInputSchema,
    },
    schema::Base64Bytes,
};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Creates a `ListPromptsResult` from a vector of prompts.
///
/// This conversion simplifies the creation of a `ListPromptsResult` when you only have
/// a list of prompts. It automatically sets `next_cursor` to `None` and `meta` to an empty map.
///
/// This is particularly useful in server implementations when implementing the `prompts/list`
/// method and you want to return just the prompts without pagination or additional metadata.
///
/// # Parameters
///
/// * `prompts` - A vector of `Prompt` objects to include in the result
///
/// # Examples
///
/// ```no_run
/// use crate::schema::{Prompt, ListPromptsResult, PromptArgument};
///
/// let prompt1 = Prompt::builder().name("example1").build().unwrap();
/// let prompt2 = Prompt::builder().name("example2").build().unwrap();
/// let prompts = vec![prompt1, prompt2];
///
/// // Convert to ListPromptsResult
/// let result: ListPromptsResult = prompts.into();
/// // Or alternatively:
/// let result = ListPromptsResult::from(prompts);
/// ```
impl From<Vec<Prompt>> for ListPromptsResult {
    fn from(prompts: Vec<Prompt>) -> Self {
        ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: Default::default(),
        }
    }
}
/// Creates a `GetPromptResult` from a vector of items that can be converted to `PromptMessage`.
///
/// This generic conversion allows creating a result from any type that can be converted into
/// a `PromptMessage`, providing flexibility when implementing server responses.
///
/// The conversion sets `description` to `None` and `meta` to an empty map, focusing on
/// just the messages content.
///
/// # Type Parameters
///
/// * `T` - Any type that implements `Into<PromptMessage>`
///
/// # Parameters
///
/// * `messages` - A vector of items that can be converted to `PromptMessage`
///
/// # Examples
///
/// ```no_run
/// use crate::schema::{GetPromptResult, PromptMessage, PromptMessageContent, TextContent, Role};
///
/// // Create messages
/// let text_content = TextContent { text: "Hello, world!".to_string(), annotations: None, type_: "text".to_string() };
/// let content = PromptMessageContent::TextContent(text_content);
/// let messages = vec![content];
///
/// // Convert to GetPromptResult
/// let result: GetPromptResult = messages.into();
/// ```
impl<T: Into<PromptMessage>> From<Vec<T>> for GetPromptResult {
    fn from(messages: Vec<T>) -> Self {
        GetPromptResult {
            description: None,
            messages: messages.into_iter().map(|m| m.into()).collect(),
            meta: Default::default(),
        }
    }
}
/// Creates a `GetPromptResult` from a single `PromptMessage`.
///
/// This convenience conversion allows creating a result from a single message,
/// which is a common use case when implementing simple prompts.
///
/// # Parameters
///
/// * `message` - A `PromptMessage` to include in the result
///
/// # Examples
///
/// ```no_run
/// use crate::schema::{GetPromptResult, PromptMessage, PromptMessageContent, TextContent, Role};
///
/// // Create a message
/// let text_content = TextContent { text: "Hello, world!".to_string(), annotations: None, type_: "text".to_string() };
/// let content = PromptMessageContent::TextContent(text_content);
/// let message = PromptMessage { content, role: Role::User };
///
/// // Convert to GetPromptResult
/// let result: GetPromptResult = message.into();
/// ```
impl From<PromptMessage> for GetPromptResult {
    fn from(message: PromptMessage) -> Self {
        vec![message].into()
    }
}
/// Creates a `PromptMessage` from `PromptMessageContent`.
///
/// This conversion simplifies the creation of a `PromptMessage` from just its content
/// by automatically setting the role to `User`. This is useful when you're primarily
/// concerned with the content and want to use the default role.
///
/// # Parameters
///
/// * `content` - The content of the message (can be text, code, image, etc.)
///
/// # Examples
///
/// ```no_run
/// use crate::schema::{PromptMessage, PromptMessageContent, TextContent, Role};
///
/// // Create content
/// let text_content = TextContent { text: "Hello, world!".to_string(), annotations: None, type_: "text".to_string() };
/// let content = PromptMessageContent::TextContent(text_content);
///
/// // Convert to PromptMessage with User role
/// let message: PromptMessage = content.into();
/// ```
impl From<PromptMessageContent> for PromptMessage {
    fn from(content: PromptMessageContent) -> Self {
        PromptMessage {
            content,
            role: Role::User,
        }
    }
}
/// Creates a `ListResourcesResult` from a vector of resources.
///
/// This conversion simplifies the creation of a `ListResourcesResult` when you only have
/// a list of resources. It automatically sets `next_cursor` to `None` and `meta` to an empty map.
///
/// This is particularly useful in server implementations when implementing the `resources/list`
/// method and you want to return just the resources without pagination or additional metadata.
///
/// # Parameters
///
/// * `resources` - A vector of `Resource` objects to include in the result
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{Resource, ListResourcesResult};
///
/// let resource1 = Resource::new("my_app://resources/example1", "Example 1");
/// let resource2 = Resource::new("my_app://resources/example2", "Example 2");
/// let resources = vec![resource1, resource2];
///
/// // Convert to ListResourcesResult
/// let result: ListResourcesResult = resources.into();
/// // Or alternatively:
/// let result = ListResourcesResult::from(resources);
/// ```
impl From<Vec<Resource>> for ListResourcesResult {
    fn from(resources: Vec<Resource>) -> Self {
        ListResourcesResult {
            resources,
            next_cursor: None,
            meta: Default::default(),
        }
    }
}
/// Creates a `ListResourceTemplatesResult` from a vector of resource templates.
///
/// This conversion simplifies the creation of a `ListResourceTemplatesResult` when you only have
/// a list of resource templates. It automatically sets `next_cursor` to `None` and `meta` to an empty map.
///
/// This is particularly useful in server implementations when implementing the `resources/templates/list`
/// method and you want to return just the templates without additional metadata.
///
/// # Parameters
///
/// * `templates` - A vector of `ResourceTemplate` objects to include in the result
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{ResourceTemplate, ListResourceTemplatesResult};
///
/// // Create resource templates
/// let template1 = ResourceTemplate {
///     name: "example1".to_string(),
///     uri_template: "my_app://resources/{id}".to_string(),
///     description: Some("Example resource template".to_string()),
///     annotations: Default::default(),
///     mime_type: None,
/// };
/// let templates = vec![template1];
///
/// // Convert to ListResourceTemplatesResult
/// let result: ListResourceTemplatesResult = templates.into();
/// ```
impl From<Vec<ResourceTemplate>> for ListResourceTemplatesResult {
    fn from(templates: Vec<ResourceTemplate>) -> Self {
        ListResourceTemplatesResult {
            resource_templates: templates,
            next_cursor: None,
            meta: Default::default(),
        }
    }
}
/// Creates a `ReadResourceResult` from a vector of `ReadResourceResultContentsItem`.
///
/// This conversion simplifies the creation of a `ReadResourceResult` when you only have
/// a list of contents items. It automatically sets `meta` to an empty map.
///
/// This is particularly useful in server implementations when implementing the `resources/read`
/// method and you want to return just the contents without additional metadata.
///
/// # Parameters
///
/// * `contents` - A vector of `ReadResourceResultContentsItem` to include in the result
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{ReadResourceResult, ReadResourceResultContentsItem};
///
/// // Create contents items
/// let item1 = ReadResourceResultContentsItem::Text(TextResourceContents {
///     text: "Hello, world!".to_string(),
///     mime_type: Some("text/plain".to_string()),
///     uri: String::new(),
/// });
/// let contents = vec![item1];
///
/// // Convert to ReadResourceResult
/// let result: ReadResourceResult = contents.into();
/// ```
impl From<Vec<ReadResourceResultContentsItem>> for ReadResourceResult {
    fn from(contents: Vec<ReadResourceResultContentsItem>) -> Self {
        ReadResourceResult {
            contents,
            meta: Default::default(),
        }
    }
}
/// Creates a `ReadResourceResult` from a single `ReadResourceResultContentsItem`.
///
/// This conversion simplifies the creation of a resource result from a single content item,
/// automatically wrapping it in a vector and setting `meta` to an empty map.
///
/// This is useful when implementing the `resources/read` method for a single resource item.
///
/// # Parameters
///
/// * `content` - The content item to include in the result
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{ReadResourceResult, ReadResourceResultContentsItem, TextResourceContents};
///
/// // Create a text resource content item
/// let text_contents = TextResourceContents {
///     text: "Hello, world!".to_string(),
///     mime_type: Some("text/plain".to_string()),
///     uri: String::new(),
/// };
/// let content_item = ReadResourceResultContentsItem::Text(text_contents);
///
/// // Convert to ReadResourceResult
/// let result: ReadResourceResult = content_item.into();
/// ```
impl From<ReadResourceResultContentsItem> for ReadResourceResult {
    fn from(content: ReadResourceResultContentsItem) -> Self {
        ReadResourceResult {
            contents: vec![content],
            meta: Default::default(),
        }
    }
}
/// Creates a `ReadResourceResult` from `TextResourceContents`.
///
/// This conversion allows easy creation of a resource result from text contents,
/// automatically wrapping it in the appropriate enum variant and setting `meta` to an empty map.
///
/// This is useful when implementing the `resources/read` method for text-based resources.
///
/// # Parameters
///
/// * `contents` - The text contents of the resource
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{ReadResourceResult, TextResourceContents};
///
/// // Create text resource contents
/// let contents = TextResourceContents {
///     text: "Hello, world!".to_string(),
///     mime_type: Some("text/plain".to_string()),
///     uri: String::new(),
/// };
///
/// // Convert to ReadResourceResult
/// let result: ReadResourceResult = contents.into();
/// ```
// Implementation moved to protocol.rs to avoid conflicting implementations
/// Creates a `ListToolsResult` from a vector of tools.
///
/// This conversion simplifies the creation of a `ListToolsResult` when you only have
/// a list of tools. It automatically sets `meta` to an empty map.
///
/// This is particularly useful in server implementations when implementing the `tools/list`
/// method and you want to return just the tools without additional metadata.
///
/// # Parameters
///
/// * `tools` - A vector of `Tool` objects to include in the result
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::schema::{Tool, ListToolsResult};
///
/// // Create tools
/// let tool1 = Tool {
///     name: "example_tool".to_string(),
///     description: Some("An example tool".to_string()),
///     input_schema: None,
///     annotations: None,
/// };
/// let tools = vec![tool1];
///
/// // Convert to ListToolsResult
/// let result: ListToolsResult = tools.into();
/// ```
impl From<Vec<Tool>> for ListToolsResult {
    fn from(tools: Vec<Tool>) -> Self {
        ListToolsResult {
            tools,
            next_cursor: None,
            meta: Default::default(),
        }
    }
}

// This implementation was moved to protocol.rs to ensure proper namespace resolution

// This implementation was moved to protocol.rs to ensure proper namespace resolution

// This implementation was moved to protocol.rs to ensure proper namespace resolution
impl Tool {
    /// Creates a new `Tool` with the specified name and input schema.
    ///
    /// This constructor initializes a tool with the given name and input schema, setting default values
    /// for other fields. The tool will have no description and no annotations.
    /// These can be set using the builder methods like `with_description` and `with_annotations`.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the tool
    /// * `input_schema` - The input schema of the tool
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{Tool, ToolInputSchema};
    /// let tool = Tool::new("example_tool", ToolInputSchema::new());
    /// ```
    pub fn new(name: &str, input_schema: ToolInputSchema) -> Self {
        Tool {
            name: name.to_string(),
            description: None,
            input_schema,
            annotations: None,
        }
    }
    /// Sets the description for this tool.
    ///
    /// This method returns the modified tool, allowing for method chaining.
    ///
    /// # Parameters
    ///
    /// * `description` - The description for this tool
    ///
    /// # Returns
    ///
    /// The modified `Tool` for method chaining
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Tool;
    /// let tool = Tool::new("example_tool", ToolInputSchema::new())
    ///     .with_description("This is an example tool.");
    /// ```
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    /// Sets the annotations for this tool.
    ///
    /// This method returns the modified tool, allowing for method chaining.
    ///
    /// # Parameters
    ///
    /// * `annotations` - The annotations for this tool
    ///
    /// # Returns
    ///
    /// The modified `Tool` for method chaining
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{Tool, ToolInputSchema, ToolAnnotations};
    /// let tool = Tool::new("example_tool", ToolInputSchema::new())
    ///     .with_annotations(ToolAnnotations::default());
    /// ```
    pub fn with_annotations(mut self, annotations: crate::schema::ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }
}
impl ToolInputSchema {
    /// Creates a new `ToolInputSchema` with default values.
    ///
    /// This constructor initializes a tool input schema with default values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::ToolInputSchema;
    /// let schema = ToolInputSchema::new();
    /// ```
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            required: vec![],
            type_: "object".to_string(),
        }
    }
    /// Inserts a property into the schema.
    ///
    /// This method inserts a property into the schema with the given name, description, and required status.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property
    /// * `description` - The description of the property
    /// * `required` - Whether the property is required
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the property was inserted successfully
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{ToolInputSchema, JsonSchema};
    /// let mut schema = ToolInputSchema::new();
    /// schema.insert_property::<String>("example_property", "An example property", true);
    /// ```
    pub fn insert_property<T: JsonSchema>(
        &mut self,
        name: &str,
        description: &str,
        required: bool,
    ) -> Result<()> {
        let mut root = schema_for!(T);
        if !description.is_empty() {
            let metadata = root
                .schema
                .metadata
                .get_or_insert(Box::new(Metadata::default()));
            metadata.description = Some(description.to_string());
        }
        let value = to_value(root.schema)?;
        let Value::Object(obj) = value else {
            bail_public!(
                ErrorCode::INVALID_PARAMS,
                "schema for `{name}` is not an object"
            );
        };
        self.properties.insert(name.to_string(), obj);
        if required {
            self.required.push(name.to_string());
        }
        Ok(())
    }
    /// Inserts a property into the schema and returns the modified schema.
    ///
    /// This method inserts a property into the schema with the given name, description, and required status,
    /// and returns the modified schema.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property
    /// * `description` - The description of the property
    /// * `required` - Whether the property is required
    ///
    /// # Returns
    ///
    /// The modified `ToolInputSchema`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{ToolInputSchema, JsonSchema};
    /// let schema = ToolInputSchema::new()
    ///     .with_property::<String>("example_property", "An example property", true);
    /// ```
    pub fn with_property<T: JsonSchema>(
        mut self,
        name: &str,
        description: &str,
        required: bool,
    ) -> Result<Self> {
        self.insert_property::<T>(name, description, required)?;
        Ok(self)
    }
}
impl Default for ToolInputSchema {
    fn default() -> Self {
        Self::new()
    }
}
impl CallToolRequestParams {
    /// Creates a new `CallToolRequestParams` with the specified tool name.
    ///
    /// This constructor initializes a call tool request parameters object with the given tool name,
    /// setting default values for other fields.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the tool to call
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::CallToolRequestParams;
    /// let params = CallToolRequestParams::new("example_tool");
    /// ```
    pub fn new(name: &str) -> Self {
        CallToolRequestParams {
            name: name.to_string(),
            arguments: ::serde_json::Map::new(),
        }
    }
    /// Adds an argument to these parameters and returns the modified parameters.
    ///
    /// This method adds an argument to the parameters with the given name and value,
    /// and returns the modified parameters.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the argument
    /// * `value` - The value of the argument
    ///
    /// # Returns
    ///
    /// The modified `CallToolRequestParams`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{CallToolRequestParams, Value};
    /// let params = CallToolRequestParams::new("example_tool")
    ///     .with_argument("example_argument", Value::String("example_value".to_string()));
    /// ```
    pub fn with_argument(mut self, name: &str, value: impl Serialize) -> Result<Self> {
        self.arguments.insert(name.to_string(), to_value(value)?);
        Ok(self)
    }
}
impl TextContent {
    /// Creates a new `TextContent` with the specified text.
    ///
    /// This constructor initializes a text content object with the given text,
    /// setting default values for other fields.
    ///
    /// # Parameters
    ///
    /// * `text` - The text content
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::TextContent;
    /// let content = TextContent::new("Hello, world!");
    /// ```
    pub fn new(text: impl std::fmt::Display) -> Self {
        Self {
            text: text.to_string(),
            annotations: None,
            type_: "text".to_string(),
        }
    }
}
impl ImageContent {
    /// Creates a new `ImageContent` with the specified data and MIME type.
    ///
    /// This constructor initializes an image content object with the given data and MIME type,
    /// setting default values for other fields.
    ///
    /// # Parameters
    ///
    /// * `data` - The image data
    /// * `mime_type` - The MIME type of the image
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{ImageContent, Base64Bytes};
    /// let data = Base64Bytes(vec![1, 2, 3]);
    /// let content = ImageContent::new(data, "image/jpeg");
    /// ```
    pub fn new(data: Base64Bytes, mime_type: &str) -> Self {
        Self {
            data: base64::prelude::BASE64_STANDARD.encode(&data.0),
            mime_type: mime_type.to_string(),
            annotations: None,
            type_: "image".to_string(),
        }
    }
}
impl EmbeddedResource {
    /// Creates a new `EmbeddedResource` with the specified resource.
    ///
    /// This constructor initializes an embedded resource object with the given resource,
    /// setting default values for other fields.
    ///
    /// # Parameters
    ///
    /// * `resource` - The embedded resource
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::{EmbeddedResource, EmbeddedResourceResource};
    /// let resource = EmbeddedResourceResource::new("example_resource");
    /// let embedded_resource = EmbeddedResource::new(resource);
    /// ```
    pub fn new(resource: impl Into<EmbeddedResourceResource>) -> Self {
        Self {
            annotations: None,
            resource: resource.into(),
            type_: "resource".to_string(),
        }
    }
}
// Note: These implementations were moved to protocol.rs to avoid conflicts
impl Implementation {
    /// Creates a new `Implementation` with the specified name and version.
    ///
    /// This constructor initializes an implementation info object with the given name and version.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the implementation (e.g., "mcp_daemon")
    /// * `version` - The version of the implementation (e.g., "0.1.0")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Implementation;
    /// let impl_info = Implementation::new("my_mcp_client", "1.0.0");
    /// ```
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
    /// Creates an `Implementation` using the compile-time crate name and version.
    ///
    /// This method uses the Rust build system's `CARGO_PKG_NAME` and `CARGO_PKG_VERSION`
    /// environment variables, which are set at compile time, to create an implementation info
    /// object that accurately reflects the current crate.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Implementation;
    /// let impl_info = Implementation::from_compile_time_env();
    /// // The name and version will match the current crate's
    /// ```
    pub fn from_compile_time_env() -> Self {
        Self::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}
impl Root {
    /// Creates a new `Root` with the specified URI.
    ///
    /// This constructor initializes a root with the given URI and no name.
    /// The name can be set using the `with_name` method if needed.
    ///
    /// # Parameters
    ///
    /// * `uri` - The URI that identifies this root
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Root;
    /// let root = Root::new("file:///home/user/documents");
    /// ```
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
            name: None,
        }
    }
    /// Sets the name for this root and returns the modified root.
    ///
    /// This is a builder-style method that takes ownership of self and returns it
    /// after modification, allowing for method chaining.
    ///
    /// # Parameters
    ///
    /// * `name` - The name to set for this root, which can be any type that implements Display
    ///
    /// # Returns
    ///
    /// The modified `Root` for method chaining
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Root;
    /// let root = Root::new("file:///home/user/documents").with_name("Documents");
    /// ```
    pub fn from_file_path(path: impl AsRef<Path>) -> Option<Self> {
        Some(Self::new(Url::from_file_path(path).ok()?.as_str()))
    }
    /// Converts this root's URI to a file system path if possible.
    ///
    /// This method attempts to convert the URI to a file path and returns None
    /// if the URI is not a valid file URI or cannot be converted to a path.
    ///
    /// # Returns
    ///
    /// An Option containing the file path if the URI could be converted, or None otherwise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_daemon::schema::Root;
    /// let root = Root::new("file:///tmp");
    /// let path = root.to_file_path();
    /// ```
    pub fn to_file_path(&self) -> Option<PathBuf> {
        Url::from_str(&self.uri).ok()?.to_file_path().ok()
    }
}
impl From<Vec<Root>> for ListRootsResult {
    fn from(roots: Vec<Root>) -> Self {
        ListRootsResult {
            roots,
            meta: Default::default(),
        }
    }
}
impl From<CompleteResultCompletion> for CompleteResult {
    fn from(completion: CompleteResultCompletion) -> Self {
        Self {
            completion,
            meta: Default::default(),
        }
    }
}
impl CompleteResultCompletion {
    pub const MAX_VALUES: usize = 100;
}

impl From<Vec<String>> for CompleteResultCompletion {
    fn from(mut values: Vec<String>) -> Self {
        let total = Some(values.len() as i64);
        let has_more = if values.len() > Self::MAX_VALUES {
            values.truncate(Self::MAX_VALUES);
            Some(true)
        } else {
            None
        };
        Self {
            has_more,
            total,
            values,
        }
    }
}
impl From<&[&str]> for CompleteResultCompletion {
    fn from(values: &[&str]) -> Self {
        values
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .into()
    }
}

impl CompleteRequestParams {
    pub fn new(r: CompleteRequestParamsRef, argument: CompleteRequestParamsArgument) -> Self {
        Self { argument, ref_: r }
    }
}
impl CompleteRequestParamsArgument {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl CompleteRequestParamsRef {
    pub fn new_prompt(name: &str) -> Self {
        CompleteRequestParamsRef::PromptReference(PromptReference::new(name))
    }
    pub fn new_resource(uri: &str) -> Self {
        CompleteRequestParamsRef::ResourceReference(ResourceReference::new(uri))
    }
}
impl PromptReference {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_: "ref/prompt".to_string(),
        }
    }
}
impl ResourceReference {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
            type_: "ref/resource".to_string(),
        }
    }
}