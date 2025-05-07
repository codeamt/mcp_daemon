use mcp_daemon::Result;
use mcp_daemon::transport::{Transport, JsonRpcMessage, JsonRpcRequest, JsonRpcVersion};
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use actix_cors::Cors;
use actix_ws::Message;
use futures::StreamExt;
use std::thread;

// Helper function to create a test message
fn create_test_message(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        id,
        method: "test".to_string(),
        params: Some(serde_json::json!({"hello": "world", "id": id})),
        jsonrpc: JsonRpcVersion::default(),
    }
}

async fn run_websocket_server() {
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

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting WebSocket transport test...");

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
    let message = create_test_message(1);

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
