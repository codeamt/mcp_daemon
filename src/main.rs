//! Entry point for the MCP daemon executable.
//!
//! This file provides a simple command-line application that can be used to test
//! or demonstrate the functionality of the MCP daemon library.

use mcp_daemon::Result;
use mcp_daemon::transport::{Transport, JsonRpcMessage, JsonRpcRequest, JsonRpcVersion};

// Helper function to create a test message
fn create_test_message(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        id,
        method: "test".to_string(),
        params: Some(serde_json::json!({"hello": "world", "id": id})),
        jsonrpc: JsonRpcVersion::default(),
    }
}

async fn test_inmemory_transport() -> Result<()> {
    eprintln!("\n=== Testing InMemory Transport ===");

    // Create an in-memory transport for testing
    let client_transport = mcp_daemon::transport::ClientInMemoryTransport::new(|server_transport| {
        // This is a simple echo server that will echo back any message it receives
        tokio::spawn(async move {
            eprintln!("Server task started");
            while let Ok(Some(message)) = server_transport.receive().await {
                eprintln!("Server received: {:?}", message);
                if let Err(e) = server_transport.send(&message).await {
                    eprintln!("Error sending response: {:?}", e);
                    break;
                }
            }
            eprintln!("Server task completed");
        })
    });

    // Open the transport
    eprintln!("Opening transport...");
    client_transport.open().await?;

    // Create a test message
    let message = create_test_message(1);

    // Send the message from client to server
    eprintln!("Sending message from client to server...");
    client_transport.send(&JsonRpcMessage::Request(message)).await?;

    // Add a small delay to ensure the message is processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Receive the echoed message back from the server
    eprintln!("Waiting for response...");
    if let Some(received) = client_transport.receive().await? {
        eprintln!("Client received response: {:?}", received);
    } else {
        eprintln!("Client did not receive any response");
    }

    // Close the transport
    eprintln!("Closing transport...");
    client_transport.close().await?;

    eprintln!("InMemory transport test complete.");

    Ok(())
}

async fn run_websocket_server() {
    use actix_web::{web, App, HttpServer};
    use actix_web::middleware::Logger;
    use actix_cors::Cors;
    use actix_ws::Message;
    use futures::StreamExt;

    use std::thread;

    eprintln!("Starting WebSocket server on localhost:8081...");

    // Run the server in a separate thread so it doesn't block the main thread
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            HttpServer::new(|| {
                App::new()
                    .wrap(Logger::default())
                    .wrap(Cors::permissive()) // Allow all origins for testing
                    .route("/ws", web::get().to(|req: actix_web::HttpRequest, stream: web::Payload| async move {
                        let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream).unwrap();

                        // Spawn a task to handle the WebSocket connection
                        actix_web::rt::spawn(async move {
                            eprintln!("WebSocket connection established");

                            // Echo server - just send back whatever we receive
                            while let Some(Ok(msg)) = msg_stream.next().await {
                                match msg {
                                    Message::Text(text) => {
                                        eprintln!("WebSocket server received: {}", text);
                                        if session.text(text.clone()).await.is_err() {
                                            break;
                                        }
                                    },
                                    Message::Close(_) => break,
                                    _ => {}
                                }
                            }

                            eprintln!("WebSocket connection closed");
                        });

                        Ok::<_, actix_web::Error>(response)
                    }))
            })
            .bind("127.0.0.1:8081").unwrap()
            .run()
            .await
            .unwrap();
        });
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    eprintln!("WebSocket server started");
}

async fn test_websocket_transport() -> Result<()> {
    eprintln!("\n=== Testing WebSocket Transport ===");

    // Start the WebSocket server
    run_websocket_server().await;

    // Create a WebSocket client transport
    let client_transport = mcp_daemon::transport::ClientWsTransport::builder(
        "ws://127.0.0.1:8081/ws".to_string()
    ).build();

    // Open the transport
    eprintln!("Opening transport...");
    client_transport.open().await?;

    // Create a test message
    let message = create_test_message(2);

    // Send the message from client to server
    eprintln!("Sending message from client to server...");
    client_transport.send(&JsonRpcMessage::Request(message)).await?;

    // Add a small delay to ensure the message is processed
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Receive the echoed message back from the server
    eprintln!("Waiting for response...");
    if let Some(received) = client_transport.receive().await? {
        eprintln!("Client received response: {:?}", received);
    } else {
        eprintln!("Client did not receive any response");
    }

    // Close the transport
    eprintln!("Closing transport...");
    client_transport.close().await?;

    eprintln!("WebSocket transport test complete.");

    Ok(())
}

// HTTP/2 transport test is commented out until TLS implementation is complete
/*
async fn test_http2_transport() -> Result<()> {
    eprintln!("\n=== Testing HTTP/2 Transport ===");

    // Start an HTTP/2 server in the background
    let server_addr = "127.0.0.1:8082".parse().unwrap();
    let server_config = mcp_daemon::transport::Http2ServerConfig {
        addr: server_addr,
        tls_config: None, // Use plain HTTP for testing
        cors_config: Some(mcp_daemon::transport::CorsConfig::default()),
    };

    // Start the HTTP/2 server
    let server_handle = mcp_daemon::transport::start_http2_server(
        server_config,
        |message| {
            // Simple echo server
            eprintln!("HTTP/2 server received: {:?}", message);
            Ok(message)
        },
    ).await?;

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Create an HTTP/2 client transport
    let client_transport = mcp_daemon::transport::Http2Builder::new()
        .with_tls(false) // Use plain HTTP for testing
        .with_host("127.0.0.1".to_string())
        .with_port(8082)
        .build();

    // Open the transport
    eprintln!("Opening transport...");
    client_transport.open().await?;

    // Create a test message
    let message = create_test_message(3);

    // Send the message from client to server
    eprintln!("Sending message from client to server...");
    client_transport.send(&JsonRpcMessage::Request(message)).await?;

    // Add a small delay to ensure the message is processed
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Receive the echoed message back from the server
    eprintln!("Waiting for response...");
    if let Some(received) = client_transport.receive().await? {
        eprintln!("Client received response: {:?}", received);
    } else {
        eprintln!("Client did not receive any response");
    }

    // Close the transport
    eprintln!("Closing transport...");
    client_transport.close().await?;

    // Stop the server
    server_handle.stop().await?;

    eprintln!("HTTP/2 transport test complete.");

    Ok(())
}
*/

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting MCP Daemon server...");
    eprintln!("Testing transport implementations...");

    // Test InMemory transport
    test_inmemory_transport().await?;

    // Test WebSocket transport
    test_websocket_transport().await?;

    // Skip HTTP/2 transport test for now
    eprintln!("\n=== Skipping HTTP/2 Transport Test ===");
    eprintln!("HTTP/2 transport test is skipped until TLS implementation is complete.");

    eprintln!("\nAll transport tests complete.");

    Ok(())
}
