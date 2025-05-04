//! Tests for the MCP server implementation using a mock server.
//!
//! This module tests the server functionality by implementing a mock server
//! that handles a subset of the MCP protocol requests. It demonstrates
//! how to use the `jsoncall` crate to create an MCP server implementation.

use async_trait::async_trait;
use jsoncall::{Handler, Params, RequestContext, Response, Result as JsResult, Session, SessionOptions};
use mcp_daemon::schema::{
    ListPromptsRequestParams, GetPromptRequestParams, SubscribeRequestParams, UnsubscribeRequestParams,
    ListPromptsResult, GetPromptResult, ServerCapabilities,
};

/// Define standalone async handler functions
/// 
/// These don't capture any references so they can be moved freely

/// Handles the prompts/list request.
///
/// This function implements the MCP protocol's prompts/list method, which returns a list
/// of available prompts. For testing purposes, it returns an empty list.
///
/// # Arguments
///
/// * `_params` - The request parameters (unused in this implementation)
///
/// # Returns
///
/// A result containing a `ListPromptsResult` with an empty list of prompts
async fn handle_prompts_list(_params: ListPromptsRequestParams) -> JsResult<ListPromptsResult> {
    // For testing, we ignore the params and return an empty list
    Ok(ListPromptsResult {
        prompts: vec![],
        next_cursor: None,
        meta: Default::default(),
    })
}

/// Handles the prompts/get request.
///
/// This function implements the MCP protocol's prompts/get method, which returns details
/// for a specific prompt. For testing purposes, it returns an empty prompt result.
///
/// # Arguments
///
/// * `_params` - The request parameters containing the prompt ID (unused in this implementation)
///
/// # Returns
///
/// A result containing a minimal `GetPromptResult`
async fn handle_prompts_get(_params: GetPromptRequestParams) -> JsResult<GetPromptResult> {
    // For testing, we ignore the params and return an empty result
    Ok(GetPromptResult {
        description: None,
        messages: vec![],
        meta: Default::default(),
    })
}

/// Handles the resources/subscribe request.
///
/// This function implements the MCP protocol's resources/subscribe method, which subscribes
/// to changes on a specific resource. For testing purposes, it returns a success result.
///
/// # Arguments
///
/// * `_params` - The request parameters containing the resource URI (unused in this implementation)
///
/// # Returns
///
/// A success result
async fn handle_resources_subscribe(_params: SubscribeRequestParams) -> JsResult<()> {
    // For testing, we just return OK
    Ok(())
}

/// Handles the resources/unsubscribe request.
///
/// This function implements the MCP protocol's resources/unsubscribe method, which unsubscribes
/// from changes on a specific resource. For testing purposes, it returns a success result.
///
/// # Arguments
///
/// * `_params` - The request parameters containing the resource URI (unused in this implementation)
///
/// # Returns
///
/// A success result
async fn handle_resources_unsubscribe(_params: UnsubscribeRequestParams) -> JsResult<()> {
    // For testing, we just return OK
    Ok(())
}

/// MockServer is a mock implementation of the MCP server.
///
/// This struct implements a simple mock MCP server for testing purposes.
/// It holds the server capabilities and instructions, though these are not
/// used in the current implementation.
#[derive(Clone)]
struct MockServer {
    /// Server capabilities (unused but maintained for potential future use)
    _capabilities: ServerCapabilities,
    /// Server instructions (unused but maintained for potential future use)
    _instructions: Option<String>,
}

impl MockServer {
    /// Creates a new MockServer instance.
    ///
    /// Initializes a MockServer with default server capabilities and
    /// a test server instruction string.
    ///
    /// # Returns
    ///
    /// A new MockServer instance
    fn new() -> Self {
        Self {
            _capabilities: ServerCapabilities::default(),
            _instructions: Some("Test server".to_string()),
        }
    }
}

#[async_trait]
impl Handler for MockServer {
    /// Handles incoming requests.
    ///
    /// This is the main request handler for the MockServer, implementing the
    /// jsoncall Handler trait. It dispatches incoming requests to the appropriate
    /// handler functions based on the method name.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name being requested
    /// * `params` - The parameters for the request
    /// * `cx` - The request context
    ///
    /// # Returns
    ///
    /// A result containing the response or an error
    fn request(&mut self, method: &str, params: Params, cx: RequestContext) -> JsResult<Response> {
        // Clone self for use in async blocks
        match method {
            "prompts/list" => {
                let params: ListPromptsRequestParams = params.to()?;
                cx.handle_async(handle_prompts_list(params))
            }
            "prompts/get" => {
                let params: GetPromptRequestParams = params.to()?;
                cx.handle_async(handle_prompts_get(params))
            }
            "resources/subscribe" => {
                let params: SubscribeRequestParams = params.to()?;
                cx.handle_async(handle_resources_subscribe(params))
            }
            "resources/unsubscribe" => {
                let params: UnsubscribeRequestParams = params.to()?;
                cx.handle_async(handle_resources_unsubscribe(params))
            }
            // For unimplemented methods, just return method_not_found
            _ => cx.method_not_found()
        }
    }
}

/// DummyClientHandler is a dummy implementation of the MCP client.
///
/// This struct implements a minimal Handler that always returns method_not_found.
/// It is used as the client-side handler when creating a test channel.
#[derive(Clone)]
struct DummyClientHandler;

#[async_trait]
impl Handler for DummyClientHandler {
    /// Handles incoming requests by always returning method_not_found.
    ///
    /// This is a minimal implementation that simply rejects all requests,
    /// as the client side in our tests doesn't need to handle any requests.
    fn request(&mut self, _method: &str, _params: Params, cx: RequestContext) -> JsResult<Response> {
        cx.method_not_found()
    }
}

/// Tests the mock server request handling.
///
/// This test creates a mock server, sets up a channel between the server and a
/// client, and verifies that the server correctly handles prompts/list and
/// resources/subscribe requests.
#[tokio::test]
async fn test_mock_server_requests() {
    // Create handlers
    let server_handler = MockServer::new();
    let client_handler = DummyClientHandler;

    // Create channel with default options
    let (server_session, client_session) = Session::new_channel(server_handler, client_handler, &SessionOptions::default());

    // Test prompts/list
    let params = ListPromptsRequestParams::default();
    let result = client_session.request::<ListPromptsResult>("prompts/list", Some(&params)).await;
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert!(res.prompts.is_empty());
        assert!(res.next_cursor.is_none());
    }

    // Test resources/subscribe
    let sub_params = SubscribeRequestParams { uri: "test://uri".to_string() };
    let sub_result = client_session.request::<()>("resources/subscribe", Some(&sub_params)).await;
    assert!(sub_result.is_ok());

    // TODO: Add tests for other methods like prompts/get once implemented

    server_session.shutdown();
    client_session.shutdown();
}