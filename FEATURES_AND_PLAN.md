# MCP Daemon New Features and Implementation Plan

## New Features

*   **HTTP/2 with TLS Transport:** Implement a new transport layer using HTTP/2 secured with TLS for encrypted and efficient communication between clients and servers.
*   **WebSockets Transport:** Add support for WebSocket connections, allowing for full-duplex communication and real-time data exchange.
*   **Server-Sent Events (SSE) Transport:** Implement SSE for enabling servers to push updates to clients over a single HTTP connection.
*   **CORS Support:** Add Cross-Origin Request Sharing (CORS) support to the HTTP/2 and WebSocket transports to allow web-based clients from different origins to interact with the daemon.
*   **Keypair Authentication:** Implement a keypair-based authentication mechanism to secure connections and verify the identity of connected clients and servers.

## Implementation Plan

This plan outlines the steps to integrate the new features into the MCP Daemon.

1.  **Transport Layer Abstraction:**
    *   Refactor the existing transport handling to introduce a clear abstraction layer. This will make it easier to add new transport types without significant changes to the core daemon logic.
    *   Define a common interface or trait for all transport implementations.

2.  **HTTP/2 with TLS Implementation:**
    *   Research and select a suitable Rust crate for implementing HTTP/2 servers and clients (e.g., `hyper`).
    *   Integrate TLS support using a crate like `rustls` or `native-tls`.
    *   Implement the server-side HTTP/2 listener and client-side connector based on the transport abstraction.

3.  **WebSockets Implementation:**
    *   Integrate the `actix-ws` crate for handling WebSocket connections.
    *   Implement the WebSocket handshake and message handling logic according to the transport abstraction.

4.  **Server-Sent Events (SSE) Implementation:**
    *   Select a suitable Rust crate for implementing SSE (you suggested using an SSE crate, I will find a good one like `actix-web-lab` which has SSE support built-in if you are already using actix-web, or find another suitable standalone crate).
    *   Implement the SSE endpoint and event-pushing mechanism.

5.  **CORS Implementation:**
    *   Integrate a CORS middleware for `actix-web` or the chosen HTTP framework to handle CORS headers for HTTP/2, WebSockets, and SSE.

6.  **Keypair Authentication Implementation:**
    *   Define a keypair structure and the authentication protocol.
    *   Implement key generation and management (consider using a crate like `ring` or `libsodium-sys`).
    *   Integrate the authentication handshake into each transport type during connection establishment.
    *   **Implement keypair authentication as an optional feature.** The server will include information in its initial JSON response to the client indicating if keypair authentication is required or available.
    *   The client implementation will be updated to handle this information and perform the keypair authentication handshake if specified by the server.

7.  **CLI Enhancements with Ratatui:**
    *   Integrate the `ratatui` crate to build the new interactive command-line interface.
    *   Implement the routing capabilities within the CLI to direct requests and responses between clients and servers.
    *   Develop modules for activity and network monitoring, displaying connected clients and servers, and visualizing data flow.

8.  **Integration and Testing:**
    *   Integrate the new transport implementations and authentication into the core MCP Daemon logic.
    *   Write comprehensive unit, integration, and end-to-end tests for all new features and the CLI.

9.  **Documentation:**
    *   Update the project documentation to reflect the new features, installation instructions, and usage of the enhanced CLI.

## Chosen Technologies

*   **Web Framework (for HTTP/2, WebSockets, SSE, CORS):** `actix-web` with `actix-ws` (as you suggested).
*   **SSE Crate:** `actix-web-lab` for seamless integration with `actix-web`, or a suitable standalone crate if needed.
*   **TLS:** `rustls` or `native-tls`.
*   **Keypair Authentication:** A cryptography crate like `ring`.
*   **CLI:** `ratatui` (as you suggested).

This plan provides a roadmap for implementing the requested features. Each step will require detailed design and implementation, with thorough testing to ensure correctness and stability.