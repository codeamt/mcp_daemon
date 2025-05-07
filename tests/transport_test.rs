use mcp_daemon::transport::{
    Transport, JsonRpcMessage, JsonRpcRequest, JsonRpcVersion,
    ClientInMemoryTransport
};
use mcp_daemon::Result;

// Helper function to create a test message
fn create_test_message() -> JsonRpcRequest {
    JsonRpcRequest {
        id: 1,
        method: "test".to_string(),
        params: Some(serde_json::json!({"hello": "world"})),
        jsonrpc: JsonRpcVersion::default(),
    }
}

// Helper function to verify a received message
fn verify_message(received: JsonRpcMessage) {
    match received {
        JsonRpcMessage::Request(req) => {
            assert_eq!(req.id, 1);
            assert_eq!(req.method, "test");
            assert_eq!(req.params.unwrap()["hello"], "world");
        },
        _ => panic!("Expected Request variant"),
    }
}

#[tokio::test]
async fn test_inmemory_transport() -> Result<()> {
    println!("Testing InMemory Transport");

    // Create an in-memory transport for testing
    let client_transport = ClientInMemoryTransport::new(|server_transport| {
        // This is a simple echo server that will echo back any message it receives
        tokio::spawn(async move {
            println!("Server task started");
            while let Ok(Some(message)) = server_transport.receive().await {
                println!("Server received: {:?}", message);
                if let Err(e) = server_transport.send(&message).await {
                    println!("Error sending response: {:?}", e);
                    break;
                }
            }
            println!("Server task completed");
        })
    });

    // Open the transport
    client_transport.open().await?;

    // Create a test message
    let message = create_test_message();

    // Send the message from client to server
    println!("Sending message from client to server...");
    client_transport.send(&JsonRpcMessage::Request(message)).await?;

    // Receive the echoed message back from the server
    if let Some(received) = client_transport.receive().await? {
        println!("Client received response: {:?}", received);
        // Verify that the response is the same as the request
        verify_message(received);
    } else {
        panic!("Client did not receive any response");
    }

    // Close the transport
    client_transport.close().await?;

    Ok(())
}

// Note: This test is commented out because it requires a WebSocket server to be running
// Uncomment and modify it if you want to test WebSocket transport
/*
#[tokio::test]
async fn test_websocket_transport() -> Result<()> {
    println!("Testing WebSocket Transport");

    // This test requires a WebSocket server to be running
    // For testing purposes, you would need to set up a WebSocket server
    // and modify this test to connect to it

    // Create a WebSocket client transport
    let client_transport = ClientWsTransport::builder()
        .url("ws://localhost:8080")
        .build();

    // Open the transport
    client_transport.open().await?;

    // Create a test message
    let message = create_test_message();

    // Send the message from client to server
    println!("Sending message from client to server...");
    client_transport.send(&JsonRpcMessage::Request(message)).await?;

    // Receive the echoed message back from the server
    if let Some(received) = client_transport.receive().await? {
        println!("Client received response: {:?}", received);
        // Verify that the response is the same as the request
        verify_message(received);
    } else {
        println!("Client did not receive any response");
    }

    // Close the transport
    client_transport.close().await?;

    Ok(())
}
*/
