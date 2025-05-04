use std::sync::{Arc, Mutex};

use mcp_daemon::{
    client::{ClientBuilder, ClientHandler},
    schema::{
        CreateMessageRequestParams, CreateMessageResult, CreateMessageResultContent,
        Root, Role, TextContent,
    },
    utils::ProtocolVersion,
};

// A simple mock sampling handler for testing
#[derive(Clone)]
struct MockSamplingHandler {
    calls: Arc<Mutex<Vec<CreateMessageRequestParams>>>,
}

impl MockSamplingHandler {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ClientHandler for MockSamplingHandler {
    fn create_message_impl(
        &self,
        p: CreateMessageRequestParams,
    ) -> jsoncall::Result<CreateMessageResult> {
        self.calls.lock().unwrap().push(p);

        // Create a simple text response
        let content = CreateMessageResultContent::TextContent(TextContent::new("Test response"));

        Ok(CreateMessageResult {
            role: Role::Assistant,
            content,
            model: "test-model".to_string(),
            stop_reason: Some("end".to_string()),
            meta: Default::default(),
        })
    }
}

#[test]
fn test_client_builder_new() {
    // Test default client builder
    let builder = ClientBuilder::new();
    let (_, _options, params) = builder.build_raw();

    // Verify default capabilities
    assert!(params.capabilities.roots.is_none());
    assert!(params.capabilities.sampling.is_empty());

    // Verify client info
    assert_eq!(params.client_info.name, "mcp_daemon");

    // Verify protocol version
    assert_eq!(params.protocol_version, ProtocolVersion::LATEST.to_string());
}

#[test]
fn test_client_builder_with_roots() {
    // Test with roots
    let roots = vec![
        Root {
            name: Some("test_root".to_string()),
            uri: "file:///test/path".to_string(),
        }
    ];

    let builder = ClientBuilder::new().with_roots(roots.clone());
    let (_, _, params) = builder.build_raw();

    // Verify roots capability
    assert!(params.capabilities.roots.is_some());
    let roots_capability = params.capabilities.roots.unwrap();
    assert_eq!(roots_capability.list_changed, Some(true));
}

#[test]
fn test_client_builder_with_handler() {
    // Test with sampling handler
    let handler = MockSamplingHandler::new();
    let builder = ClientBuilder::new().with_handler(handler);
    let (_, _, _) = builder.build_raw();

    // The test passes if we get here
    // The sampling capability is set as an empty map in ClientBuilder.build_raw()
}

#[test]
fn test_client_builder_with_expose_internals() {
    // Test with expose_internals
    let builder = ClientBuilder::new().with_expose_internals(true);
    let (_, options, _) = builder.build_raw();

    // Verify expose_internals option
    assert_eq!(options.expose_internals, Some(true));
}

// The ClientBuilder doesn't have a with_client_info method, so we'll skip this test

#[test]
fn test_client_builder_default() {
    // Test default implementation
    let builder = ClientBuilder::default();
    let (_, _, params) = builder.build_raw();

    // Verify default capabilities
    assert!(params.capabilities.roots.is_none());
    assert!(params.capabilities.sampling.is_empty());

    // Verify client info
    assert_eq!(params.client_info.name, "mcp_daemon");
}
