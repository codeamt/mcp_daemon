use std::sync::Arc;

use jsoncall::{
    Handler, Hook, NotificationContext, Params, RequestContextAs, RequestId, Response,
    Result, Session, SessionContext, SessionOptions, SessionResult, bail_public,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Map;

use crate::{
    request::session::CancellationHook,
    schema::{
        CallToolRequestParams, CallToolResult, CancelledNotificationParams, ClientCapabilities,
        CompleteRequestParams, CompleteResult, CreateMessageRequestParams, CreateMessageResult,
        GetPromptRequestParams, GetPromptResult, Implementation, InitializeRequestParams,
        InitializeResult, InitializedNotificationParams, ListPromptsRequestParams,
        ListPromptsResult, ListResourceTemplatesRequestParams, ListResourceTemplatesResult,
        ListResourcesRequestParams, ListResourcesResult, ListRootsRequestParams, ListRootsResult,
        ListToolsRequestParams, ListToolsResult, PingRequestParams, ProgressNotificationParams,
        ProgressToken, ReadResourceRequestParams, ReadResourceResult, Root, ServerCapabilities,
        ServerCapabilitiesPrompts, ServerCapabilitiesResources, ServerCapabilitiesTools,
        SetLevelRequestParams, SubscribeRequestParams, UnsubscribeRequestParams,
    },
    error::{prompt_not_found, resource_not_found, tool_not_found},
    schema::types_ex::{Empty, ProtocolVersion},
};

pub use crate::utility::macros::server;

pub struct SessionData {
    pub initialize: InitializeRequestParams,
    pub protocol_version: ProtocolVersion,
}

struct ServerHandler {
    server: Arc<dyn Server>,
    data: Option<Arc<SessionData>>,
    is_initialized: bool,
}
impl Handler for ServerHandler {
    fn hook(&self) -> Arc<dyn Hook> {
        Arc::new(CancellationHook)
    }
    fn request(
        &mut self,
        method: &str,
        params: Params,
        cx: jsoncall::RequestContext,
    ) -> Result<Response> {
        match method {
            "initialize" => return cx.handle(self.initialize(params.to()?)),
            "ping" => return cx.handle(self.ping(params.to_opt()?)),
            "logging/setLevel" => return cx.handle(self.logging_set_level(params.to()?)),
            _ => {}
        }
        let (Some(data), true) = (&self.data, self.is_initialized) else {
            bail_public!(_, "Server not initialized");
        };
        let d = data.clone();
        match method {
            "prompts/list" => self.call_opt(params, cx, |s, p, cx| s.prompts_list(p, cx, d)),
            "prompts/get" => self.call(params, cx, |s, p, cx| s.prompts_get(p, cx, d)),
            "resources/list" => {
                self.call_opt(params, cx, |s, p, cx| s.resources_list(p, cx, d))
            }
            "resources/templates/list" => self.call_opt(params, cx, |s, p, cx| {
                s.resources_templates_list(p, cx, d)
            }),
            "resources/read" => self.call(params, cx, |s, p, cx| s.resources_read(p, cx, d)),
            "resources/subscribe" => self.call(params, cx, |s, p, cx| s.resources_subscribe(p, cx, d)),
            "resources/unsubscribe" => self.call(params, cx, |s, p, cx| s.resources_unsubscribe(p, cx, d)),
            "tools/list" => self.call_opt(params, cx, |s, p, cx| s.tools_list(p, cx, d)),
            "tools/call" => self.call(params, cx, |s, p, cx| s.tools_call(p, cx, d)),
            "completion/complete" => {
                self.call(params, cx, |s, p, cx| s.completion_complete(p, cx, d))
            }
            _ => cx.method_not_found(),
        }
    }
    fn notification(
        &mut self,
        method: &str,
        params: Params,
        cx: NotificationContext,
    ) -> Result<Response> {
        match method {
            "notifications/initialized" => cx.handle(self.initialized(params.to_opt()?)),
            "notifications/cancelled" => self.notifications_cancelled(params.to()?, cx),
            _ => cx.method_not_found(),
        }
    }
}
impl ServerHandler {
    pub fn new(server: impl Server) -> Self {
        Self {
            server: Arc::new(server),
            data: None,
            is_initialized: false,
        }
    }
}
impl ServerHandler {
    fn initialize(&mut self, p: InitializeRequestParams) -> Result<InitializeResult> {
        self.data = Some(Arc::new(SessionData {
            initialize: p,
            protocol_version: ProtocolVersion::LATEST,
        }));
        Ok(self.server.initialize_result())
    }
    fn initialized(&mut self, _p: Option<InitializedNotificationParams>) -> Result<()> {
        if self.data.is_none() {
            bail_public!(
                _,
                "`initialize` request must be called before `initialized` notification"
            );
        }
        self.is_initialized = true;
        Ok(())
    }
    fn ping(&self, _p: Option<PingRequestParams>) -> Result<Empty> {
        Ok(Empty::default())
    }
    fn notifications_cancelled(
        &self,
        p: CancelledNotificationParams,
        cx: NotificationContext,
    ) -> Result<Response> {
        cx.session().cancel_incoming_request(&p.request_id, None);
        cx.handle(Ok(()))
    }

    /// Handles [`logging/setLevel`]
    ///
    /// [`logging/setLevel`]: https://spec.modelcontextprotocol.io/specification/draft/server/utilities/logging/#setting-log-level
    fn logging_set_level(&self, _p: SetLevelRequestParams) -> Result<Empty> {
        // Store the log level in the session context or a global variable
        // For now, we'll just acknowledge the request
        Ok(Empty::default())
    }

    fn call<P, R>(
        &self,
        p: Params,
        cx: jsoncall::RequestContext,
        f: impl FnOnce(Arc<dyn Server>, P, RequestContextAs<R>) -> Result<Response>,
    ) -> Result<Response>
    where
        P: DeserializeOwned,
        R: Serialize,
    {
        f(self.server.clone(), p.to()?, cx.to())
    }
    fn call_opt<P, R>(
        &self,
        p: Params,
        cx: jsoncall::RequestContext,
        f: impl FnOnce(Arc<dyn Server>, P, RequestContextAs<R>) -> Result<Response>,
    ) -> Result<Response>
    where
        P: DeserializeOwned + Default,
        R: Serialize,
    {
        f(
            self.server.clone(),
            p.to_opt()?.unwrap_or_default(),
            cx.to(),
        )
    }
}

/// Trait for implementing an MCP-compliant server.
///
/// The `Server` trait is the core interface for implementing an MCP server. It defines
/// all the methods that an MCP server must implement to be compliant with the protocol.
/// Most methods have default implementations that return empty results or appropriate errors.
///
/// Implementations of this trait can be used with the `serve_stdio` function to create
/// a complete MCP server that communicates via standard input/output.
///
/// The easiest way to implement this trait is to use the `#[server]` attribute macro,
/// which generates implementations of these methods based on annotated functions.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::server::{server, Server, DefaultServer};
/// use mcp_daemon::Result;
///
/// struct MyServer;
///
/// #[server]
/// impl Server for MyServer {
///     // Implement required methods with attributes
///     #[prompt]
///     async fn hello(&self) -> Result<String> {
///         Ok("Hello, world!".to_string())
///     }
/// }
/// ```
pub trait Server: Send + Sync + 'static {
    /// Returns the initialization result
    fn initialize_result(&self) -> InitializeResult;

    /// Handles prompts/list request
    fn prompts_list(
        self: Arc<Self>,
        p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles prompts/get request
    fn prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles resources/list request
    fn resources_list(
        self: Arc<Self>,
        p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles resources/read request
    fn resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles resources/templates/list request
    fn resources_templates_list(
        self: Arc<Self>,
        p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles tools/list request
    fn tools_list(
        self: Arc<Self>,
        p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles tools/call request
    fn tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles completion/complete request
    fn completion_complete(
        self: Arc<Self>,
        p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles resources/subscribe request
    fn resources_subscribe(
        self: Arc<Self>,
        p: SubscribeRequestParams,
        cx: RequestContextAs<Empty>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    /// Handles resources/unsubscribe request
    fn resources_unsubscribe(
        self: Arc<Self>,
        p: UnsubscribeRequestParams,
        cx: RequestContextAs<Empty>,
        data: Arc<SessionData>,
    ) -> Result<Response>;
}

/// Trait for default implementation of Server methods
pub trait DefaultServer: Server {
    /// Returns server information
    fn server_info(&self) -> Implementation;

    /// Returns instructions
    fn instructions(&self) -> Option<String>;

    /// Returns capabilities
    fn capabilities(&self) -> ServerCapabilities;

    /// Gets the JSON RPC Handler
    fn into_handler(self) -> impl Handler + Send + Sync + 'static
    where
        Self: Sized + Send + Sync + 'static;

    /// Default implementation of initialize_result
    fn initialize_result(&self) -> InitializeResult {
        InitializeResult {
            capabilities: self.capabilities(),
            instructions: self.instructions(),
            meta: Map::new(),
            protocol_version: ProtocolVersion::LATEST.to_string(),
            server_info: self.server_info(),
        }
    }

    fn prompts_list(
        self: Arc<Self>,
        p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn resources_list(
        self: Arc<Self>,
        p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn resources_templates_list(
        self: Arc<Self>,
        p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn tools_list(
        self: Arc<Self>,
        p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;

    fn completion_complete(
        self: Arc<Self>,
        p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        data: Arc<SessionData>,
    ) -> Result<Response>;
}
// Implementation of the Server trait methods
impl<T: Server> Server for T {
    fn initialize_result(&self) -> InitializeResult {
        InitializeResult {
            capabilities: self.capabilities(),
            instructions: self.instructions(),
            meta: Map::new(),
            protocol_version: ProtocolVersion::LATEST.to_string(),
            server_info: self.server_info(),
        }
    }
    fn prompts_list(
        self: Arc<Self>,
        _p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListPromptsResult::default()))
    }

    fn prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(prompt_not_found(&p.name)))
    }

    fn resources_list(
        self: Arc<Self>,
        _p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListResourcesResult::default()))
    }

    fn resources_templates_list(
        self: Arc<Self>,
        _p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListResourceTemplatesResult::default()))
    }

    fn resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(resource_not_found(&p.uri)))
    }

    fn tools_list(
        self: Arc<Self>,
        _p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListToolsResult::default()))
    }

    fn tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(tool_not_found(&p.name)))
    }

    fn completion_complete(
        self: Arc<Self>,
        _p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(CompleteResult::default()))
    }

    /// Handles [`resources/subscribe`]
    ///
    /// [`resources/subscribe`]: https://spec.modelcontextprotocol.io/specification/draft/server/resources/#subscriptions
    #[allow(unused_variables)]
    fn resources_subscribe(
        self: Arc<Self>,
        _p: SubscribeRequestParams,
        cx: RequestContextAs<Empty>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(Empty::default()))
    }

    /// Handles [`resources/unsubscribe`]
    ///
    /// [`resources/unsubscribe`]: https://spec.modelcontextprotocol.io/specification/draft/server/resources/#subscriptions
    #[allow(unused_variables)]
    fn resources_unsubscribe(
        self: Arc<Self>,
        _p: UnsubscribeRequestParams,
        cx: RequestContextAs<Empty>,
        _data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(Empty::default()))
    }
}

/// Default implementation of the Server trait
impl<T: Server> DefaultServer for T {
    /// Returns `server_info` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle/#initialization
    fn server_info(&self) -> Implementation {
        Implementation::from_compile_time_env()
    }

    /// Returns `instructions` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle/#initialization
    fn instructions(&self) -> Option<String> {
        None
    }

    /// Returns `capabilities` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle/#initialization
    fn capabilities(&self) -> ServerCapabilities {
        let mut logging = Map::new();
        logging.insert("setLevel".to_string(), serde_json::Value::Bool(true));

        ServerCapabilities {
            prompts: Some(ServerCapabilitiesPrompts {
                ..Default::default()
            }),
            resources: Some(ServerCapabilitiesResources {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ServerCapabilitiesTools {
                list_changed: Some(true),
            }),
            logging,
            ..Default::default()
        }
    }

    /// Handles [`prompts/list`]
    ///
    /// [`prompts/list`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/prompts/#listing-prompts
    #[allow(unused_variables)]
    fn prompts_list(
        self: Arc<Self>,
        p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListPromptsResult::default()))
    }

    /// Handles [`prompts/get`]
    ///
    /// [`prompts/get`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/prompts/#getting-a-prompt
    #[allow(unused_variables)]
    fn prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(prompt_not_found(&p.name)))
    }

    /// Handles [`resources/list`]
    ///
    /// [`resources/list`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/resources/#listing-resources
    #[allow(unused_variables)]
    fn resources_list(
        self: Arc<Self>,
        p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListResourcesResult::default()))
    }

    /// Handles [`resources/templates/list`]
    ///
    /// [`resources/templates/list`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/resources/#resource-templates
    #[allow(unused_variables)]
    fn resources_templates_list(
        self: Arc<Self>,
        p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListResourceTemplatesResult::default()))
    }

    /// Handles [`resources/read`]
    ///
    /// [`resources/read`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/resources/#reading-resources
    #[allow(unused_variables)]
    fn resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(resource_not_found(&p.uri)))
    }

    /// Handles [`tools/list`]
    ///
    /// [`tools/list`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/tools/#listing-tools
    #[allow(unused_variables)]
    fn tools_list(
        self: Arc<Self>,
        p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(ListToolsResult::default()))
    }

    /// Handles [`tools/call`]
    ///
    /// [`tools/call`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/tools/#calling-a-tool
    #[allow(unused_variables)]
    fn tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Err(tool_not_found(&p.name)))
    }

    /// Handles [`completion/complete`]
    ///
    /// [`completion/complete`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/utilities/completion/#completing-a-prompt
    #[allow(unused_variables)]
    fn completion_complete(
        self: Arc<Self>,
        p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        data: Arc<SessionData>,
    ) -> Result<Response> {
        cx.handle(Ok(CompleteResult::default()))
    }

    /// Gets the JSON RPC `Handler`
    fn into_handler(self) -> impl Handler + Send + Sync + 'static
    where
        Self: Sized + Send + Sync + 'static,
    {
        ServerHandler::new(self)
    }
}

/// Context for retrieving request-related information and calling client features
pub struct RequestContext {
    session: SessionContext,
    id: RequestId,
    data: Arc<SessionData>,
}

impl RequestContext {
    pub fn new(cx: &RequestContextAs<impl Serialize>, data: Arc<SessionData>) -> Self {
        Self {
            session: cx.session(),
            id: cx.id().clone(),
            data,
        }
    }

    /// Gets client information
    pub fn client_info(&self) -> &Implementation {
        &self.data.initialize.client_info
    }

    /// Gets client capabilities
    pub fn client_capabilities(&self) -> &ClientCapabilities {
        &self.data.initialize.capabilities
    }

    /// Protocol version of the current session
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.data.protocol_version
    }

    /// Notifies progress of the request associated with this context
    ///
    /// See [`notifications/progress`]
    ///
    /// [`notifications/progress`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/notifications/#progress-notification
    pub fn progress(&self, progress: f64, total: Option<f64>) {
        self.session
            .notification(
                "notifications/progress",
                Some(&ProgressNotificationParams {
                    progress,
                    total,
                    progress_token: ProgressToken::String(self.id.to_string()),
                    message: None,
                }),
            )
            .unwrap();
    }

    /// Calls [`sampling/createMessage`]
    pub async fn sampling_create_message(
        &self,
        p: CreateMessageRequestParams,
    ) -> SessionResult<CreateMessageResult> {
        self.session
            .request("sampling/createMessage", Some(&p))
            .await
    }

    /// Calls [`roots/list`]
    pub async fn roots_list(&self) -> SessionResult<Vec<Root>> {
        let res: ListRootsResult = self
            .session
            .request("roots/list", Some(&ListRootsRequestParams::default()))
            .await?;
        Ok(res.roots)
    }
}

/// Runs an MCP server using stdio transport
pub async fn serve_stdio(server: impl Server) -> SessionResult<()> {
    Session::from_stdio(ServerHandler::new(server), &SessionOptions::default())
        .wait()
        .await
}

/// Runs an MCP server using stdio transport with specified options
pub async fn serve_stdio_with(
    server: impl Server,
    options: &SessionOptions,
) -> SessionResult<()> {
    Session::from_stdio(ServerHandler::new(server), options)
        .wait()
        .await
}
