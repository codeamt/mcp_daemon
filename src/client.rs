//! Client implementation for connecting to MCP servers
//!
//! This module provides the `Client` struct and related functionality for
//! connecting to MCP servers and accessing their resources and tools.
//!
//! # Examples
//!
//! ```rust,ignore
//! use mcp_daemon::client::Client;
//! use mcp_daemon::schema::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client
//!     let client = Client::new("http://localhost:8080");
//!
//!     // Initialize the client
//!     let init_result = client.initialize().await?;
//!     println!("Connected to server: {}", init_result.server_info.name);
//!
//!     // List available tools
//!     let tools = client.tools_list(ListToolsRequestParams::default()).await?;
//!     println!("Available tools: {}", tools.tools.len());
//!
//!     Ok(())
//! }
//! ```

use std::sync::Arc;

use derive_ex::Ex;
use jsoncall::{
    Handler, NotificationContext, Params, RequestContext, RequestContextAs, Response, Result,
    Session, SessionOptions, SessionResult,
};
use serde_json::Map;
use tokio::{
    io::{AsyncBufRead, AsyncWrite},
    process::Command,
};

use crate::common::McpCancellationHook;
use crate::schema::{
    CallToolRequestParams, CallToolResult, CancelledNotificationParams, ClientCapabilities,
    ClientCapabilitiesRoots, CompleteRequestParams, CompleteResult, CreateMessageRequestParams,
    CreateMessageResult, GetPromptRequestParams, GetPromptResult, Implementation,
    InitializeRequestParams, InitializeResult, InitializedNotificationParams,
    ListPromptsRequestParams, ListPromptsResult, ListResourceTemplatesRequestParams,
    ListResourceTemplatesResult, ListResourcesRequestParams, ListResourcesResult,
    ListRootsResult, ListToolsRequestParams, ListToolsResult, PingRequestParams,
    ReadResourceRequestParams, ReadResourceResult, Root,
};
use crate::server::{Server, DefaultServer};
use crate::utils::{Empty, ProtocolVersion};
/// Trait for implementing [client features]
///
/// Used with [`ClientBuilder::with_handler`] to create an MCP client that supports client features.
///
/// [client features]: https://spec.modelcontextprotocol.io/specification/2025-03-26/client/
pub trait ClientHandler {
    /// [`sampling/createMessage`](https://spec.modelcontextprotocol.io/specification/2025-03-26/client/sampling/)
    fn create_message_impl(
        &self,
        p: CreateMessageRequestParams,
    ) -> Result<CreateMessageResult>;

    fn create_message(
        &self,
        p: CreateMessageRequestParams,
        cx: RequestContextAs<CreateMessageResult>,
    ) -> Result<Response> {
        cx.handle(self.create_message_impl(p))
    }
}
trait DynSamplingHandler: Send + Sync + 'static {
    fn create_message(
        self: Arc<Self>,
        p: CreateMessageRequestParams,
        cx: RequestContextAs<CreateMessageResult>,
    ) -> Result<Response>;
}
impl<T: ClientHandler + Send + Sync + 'static> DynSamplingHandler for T {
    fn create_message(
        self: Arc<Self>,
        p: CreateMessageRequestParams,
        cx: RequestContextAs<CreateMessageResult>,
    ) -> Result<Response> {
        let handler = self.clone();
        handler.create_message_impl(p).and_then(|result| cx.handle(Ok(result)))
    }
}
/// Builder for creating [`Client`]
///
/// The `ClientBuilder` allows you to configure and create a new `Client` instance
/// with custom settings.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::client::ClientBuilder;
/// use mcp_daemon::schema::Root;
/// use mcp_daemon::server::{Server, DefaultServer};
///
/// struct MyServer;
///
/// impl Server for MyServer {
///     // Implement required methods
///     // ...
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let server = MyServer;
/// let roots = vec![Root {
///     name: Some("my_root".to_string()),
///     uri: "/path/to/root".to_string(),
/// }];
///
/// let client = ClientBuilder::new()
///     .with_expose_internals(true)
///     .with_roots(roots)
///     .build_with_server(server)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Ex)]
#[derive_ex(Default)]
#[default(Self::new())]
pub struct ClientBuilder {
    sampling_handler: Option<Arc<dyn DynSamplingHandler>>,
    roots: Option<Vec<Root>>,
    client_info: Implementation,
    expose_internals: Option<bool>,
}
impl ClientBuilder {
    /// Creates a new [`Client`]
    pub fn new() -> Self {
        Self {
            sampling_handler: None,
            roots: None,
            client_info: Implementation::from_compile_time_env(),
            expose_internals: None,
        }
    }

    /// Creates a `ClientBuilder` with a specified [`ClientHandler`]
    pub fn with_handler(
        mut self,
        sampling_handler: impl ClientHandler + Send + Sync + 'static,
    ) -> Self {
        self.sampling_handler = Some(Arc::new(sampling_handler));
        self
    }

    /// Specifies the values to be returned by [`roots/list`]
    ///
    /// Also sets the roots capabilities that the MCP client will return.
    ///
    /// [`roots/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/roots/#listing-roots
    pub fn with_roots(mut self, roots: Vec<Root>) -> Self {
        self.roots = Some(roots);
        self
    }

    /// Sets whether to expose internal information in errors
    ///
    /// See [`Error`](crate::Error) for details about internal information
    pub fn with_expose_internals(mut self, expose_internals: bool) -> Self {
        self.expose_internals = Some(expose_internals);
        self
    }

    /// Builds a [`Client`] client using the specified reader and writer
    pub async fn build(
        self,
        reader: impl AsyncBufRead + Send + Sync + 'static,
        writer: impl AsyncWrite + Send + Sync + 'static,
    ) -> SessionResult<Client> {
        let (handler, options, p) = self.build_raw();
        Client::initialize(Session::new(handler, reader, writer, &options), p).await
    }
    /// Launches a MCP server process with the specified command and builds [`Client`] that communicates with it using stdio transport
    pub async fn build_with_command(self, command: &mut Command) -> SessionResult<Client> {
        let (handler, options, p) = self.build_raw();
        Client::initialize(Session::from_command(handler, command, &options)?, p).await
    }

    /// Builds a [`Client`] client that communicates with the specified MCP server
    ///
    /// The specified `McpServer` will be owned by the returned Client.
    pub async fn build_with_server(self, server: impl Server) -> SessionResult<Client> {
        let (client_handler, options, p) = self.build_raw();
        let server_handler = server.into_handler();

        let (client, server) = Session::new_channel(client_handler, server_handler, &options);
        let mut client = Client::initialize(client, p).await?;
        client.server = Some(server);
        Ok(client)
    }

    /// Builds a [`Client`] using a custom method
    ///
    /// This method returns the values needed for [`Client::initialize`].
    /// It is provided for using transports that cannot be handled by [`build`](Self::build), [`build_with_command`](Self::build_with_command), or [`build_with_server`](Self::build_with_server).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use mcp_daemon::client::{ClientBuilder, Client};
    /// use tokio::process::Command;
    /// use jsoncall::Session;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut command = Command::new("cargo");
    /// command.args(&["run", "--bin", "mcp-attr", "--example", "char_count"]);
    ///
    /// let builder = ClientBuilder::new();
    /// let (handler, options, initialize_params) = builder.build_raw();
    /// let client = Client::initialize(Session::from_command(handler, &mut command, &options)?, initialize_params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_raw(self) -> (impl Handler, SessionOptions, InitializeRequestParams) {
        let mut capabilities = ClientCapabilities::default();
        if self.roots.is_some() {
            capabilities.roots = Some(ClientCapabilitiesRoots {
                list_changed: Some(true),
            });
        }
        if self.sampling_handler.is_some() {
            capabilities.sampling = Map::new();
        }
        let handler = ClientJsonRpcHandler {
            sampling_handler: self.sampling_handler,
            roots: self.roots,
        };
        let options = SessionOptions {
            expose_internals: self.expose_internals,
        };
        let p = InitializeRequestParams {
            capabilities,
            client_info: self.client_info,
            protocol_version: ProtocolVersion::LATEST.to_string(),
        };
        (handler, options, p)
    }
}

struct ClientJsonRpcHandler {
    sampling_handler: Option<Arc<dyn DynSamplingHandler>>,
    roots: Option<Vec<Root>>,
}
impl Handler for ClientJsonRpcHandler {
    fn hook(&self) -> Arc<dyn jsoncall::Hook> {
        Arc::new(McpCancellationHook)
    }
    fn request(&mut self, method: &str, params: Params, cx: RequestContext) -> Result<Response> {
        match method {
            "sampling/createMessage" => {
                if let Some(h) = &self.sampling_handler {
                    return h.clone().create_message(params.to()?, cx.to());
                }
            }
            "ping" => return cx.handle(self.ping(params.to()?)),
            "roots/list" => {
                return self.roots_list(cx.to());
            }
            _ => {}
        }
        cx.method_not_found()
    }

    fn notification(
        &mut self,
        method: &str,
        params: Params,
        cx: NotificationContext,
    ) -> Result<Response> {
        match method {
            "notifications/cancelled" => self.notifications_cancelled(params.to()?, cx),
            _ => cx.method_not_found(),
        }
    }
}
impl ClientJsonRpcHandler {
    fn ping(&self, _p: PingRequestParams) -> Result<Empty> {
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
    fn roots_list(&self, cx: RequestContextAs<ListRootsResult>) -> Result<Response> {
        if let Some(roots) = &self.roots {
            cx.handle(Ok(roots.clone().into()))
        } else {
            cx.method_not_found()
        }
    }
}
/// MCP client
///
/// The `Client` struct provides methods for interacting with an MCP server,
/// including initializing the connection, listing and accessing resources,
/// calling tools, and more.
///
/// To create a `Client`, use the [`with_server`](Self::with_server) method or [`ClientBuilder`].
/// The method to create a `Client` performs an [`initialize`] request to the server and returns control when the request completes.
///
/// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2025-03-26/client/initialize/
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::client::Client;
/// use mcp_daemon::schema::*;
/// use mcp_daemon::server::{Server, DefaultServer};
///
/// struct MyServer;
///
/// impl Server for MyServer {
///     // Implement required methods
///     // ...
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let server = MyServer;
/// let client = Client::with_server(server).await?;
///
/// // Get prompt list
/// let prompts = client.prompts_list(None).await?;
///
/// // Get resource list
/// let resources = client.resources_list(None).await?;
///
/// // Call tool
/// let params = CallToolRequestParams {
///     name: "tool_name".to_string(),
///     arguments: Some(serde_json::Map::new()),
/// };
/// let result = client.tools_call(params).await?;
/// # Ok(())
/// # }
/// ```
/// Client for interacting with MCP servers.
///
/// The `Client` struct provides methods for interacting with an MCP server,
/// including initializing the connection, listing and retrieving prompts,
/// accessing resources, and calling tools.
///
/// It handles the communication details of the Model Context Protocol and provides
/// a high-level API for working with MCP servers.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::client::Client;
/// use mcp_daemon::schema::*;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client that connects to a server at the specified URL
///     let client = Client::new("http://localhost:8080");
///
///     // Initialize the client with the server
///     let init_result = client.initialize().await?;
///     println!("Connected to server: {}", init_result.server_info.name);
///
///     // List available prompts
///     let prompts = client.prompts_list(None).await?;
///     for prompt in prompts.prompts {
///         println!("Found prompt: {}", prompt.name);
///     }
///
///     // Call a tool
///     let tool_result = client.tools_call(CallToolRequestParams {
///         name: "example_tool".to_string(),
///         arguments: serde_json::Map::new(),
///     }).await?;
///
///     Ok(())
/// }
/// ```
pub struct Client {
    session: Session,
    init: InitializeResult,
    server: Option<Session>,
}

impl Client {
    /// Connects to the specified `McpServer` in-process
    ///
    /// Performs an [`initialize`] request to the server and returns the result
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/initialize/
    pub async fn with_server(server: impl Server) -> SessionResult<Self> {
        ClientBuilder::new().build_with_server(server).await
    }

    /// Connects to an MCP server using the specified JSON RPC Session
    ///
    /// This `Session` uses the values returned from [`ClientBuilder::build_raw`].
    ///
    /// Performs an [`initialize`] request to the server and returns the result
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/initialize/
    pub async fn initialize(session: Session, p: InitializeRequestParams) -> SessionResult<Self> {
        let init = session
            .request::<InitializeResult>("initialize", Some(&p))
            .await?;
        session.notification(
            "notifications/initialized",
            Some(&InitializedNotificationParams::default()),
        )?;
        Ok(Self {
            session,
            init,
            server: None,
        })
    }

    /// Gets the JSON RPC Session
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Gets the `instructions` obtained from the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/initialize/
    pub fn instructions(&self) -> Option<&str> {
        self.init.instructions.as_deref()
    }

    /// Gets the `server_info` obtained from the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/initialize/
    pub fn server_info(&self) -> &Implementation {
        &self.init.server_info
    }

    /// Calls [`prompts/list`]
    ///
    /// [`prompts/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/prompts/#listing-prompts
    pub async fn prompts_list(
        &self,
        params: Option<ListPromptsRequestParams>,
    ) -> SessionResult<ListPromptsResult> {
        self.session.request("prompts/list", params.as_ref()).await
    }

    /// Calls [`prompts/get`]
    ///
    /// [`prompts/get`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/prompts/#getting-a-prompt
    pub async fn prompts_get(
        &self,
        params: GetPromptRequestParams,
    ) -> SessionResult<GetPromptResult> {
        self.session.request("prompts/get", Some(&params)).await
    }

    /// Calls [`resources/list`]
    ///
    /// [`resources/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/resources/#listing-resources
    pub async fn resources_list(
        &self,
        params: Option<ListResourcesRequestParams>,
    ) -> SessionResult<ListResourcesResult> {
        self.session
            .request("resources/list", params.as_ref())
            .await
    }

    /// Calls [`resources/templates/list`]
    ///
    /// [`resources/templates/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/resources/#listing-resource-templates
    pub async fn resources_templates_list(
        &self,
        params: Option<ListResourceTemplatesRequestParams>,
    ) -> SessionResult<ListResourceTemplatesResult> {
        self.session
            .request("resources/templates/list", params.as_ref())
            .await
    }

    /// Calls [`resources/read`]
    ///
    /// [`resources/read`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/resources/#reading-a-resource
    pub async fn resources_read(
        &self,
        params: ReadResourceRequestParams,
    ) -> SessionResult<ReadResourceResult> {
        self.session.request("resources/read", Some(&params)).await
    }

    /// Calls [`tools/list`]
    ///
    /// [`tools/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/tools/#listing-tools
    pub async fn tools_list(
        &self,
        params: Option<ListToolsRequestParams>,
    ) -> SessionResult<ListToolsResult> {
        self.session.request("tools/list", params.as_ref()).await
    }

    /// Calls [`tools/call`]
    ///
    /// [`tools/call`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/tools/#calling-a-tool
    pub async fn tools_call(&self, params: CallToolRequestParams) -> SessionResult<CallToolResult> {
        self.session.request("tools/call", Some(&params)).await
    }

    /// Calls [`completion/complete`]
    ///
    /// [`completion/complete`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/client/completion/#completing-a-prompt
    pub async fn completion_complete(
        &self,
        params: CompleteRequestParams,
    ) -> SessionResult<CompleteResult> {
        self.session
            .request("completion/complete", Some(&params))
            .await
    }
    /// Calls [`ping`]
    ///
    /// [`ping`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/utilities/ping/
    pub async fn ping(&self) -> SessionResult<()> {
        let _: Empty = self
            .session
            .request("ping", Some(&PingRequestParams::default()))
            .await?;
        Ok(())
    }
}
