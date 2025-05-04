use std::sync::{Arc, Mutex};

use mcp_daemon::{
    client::{ClientBuilder, ClientHandler},
    schema::{CreateMessageRequestParams, CreateMessageResult, CreateMessageResultContent, Role, TextContent},
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

    // Unused but kept for potential future tests
    #[allow(dead_code)]
    fn get_calls(&self) -> Vec<CreateMessageRequestParams> {
        self.calls.lock().unwrap().clone()
    }
}

impl ClientHandler for MockSamplingHandler {
    fn create_message_impl(
        &self,
        p: CreateMessageRequestParams,
    ) -> jsoncall::Result<CreateMessageResult> {
        self.calls.lock().unwrap().push(p);

        // Create a simple text response using the helper method
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
fn test_client_with_sampling_handler() {
    // Create a mock sampling handler
    let handler = MockSamplingHandler::new();

    // Create a client builder with the handler
    let builder = ClientBuilder::new().with_handler(handler.clone());
    let (_, _, _params) = builder.build_raw();

    // The test passes if we get here, as we've verified the builder works with a sampling handler
}
