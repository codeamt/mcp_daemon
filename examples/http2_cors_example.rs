use mcp_daemon::transport::{CorsConfig, Http2ServerConfig, TlsConfig, start_http2_server};
use std::net::SocketAddr;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create a custom CORS configuration
    let cors_config = CorsConfig {
        allowed_origins: "*".to_string(), // Allow all origins for testing
        allowed_methods: "GET, POST, OPTIONS".to_string(),
        allowed_headers: "Content-Type, Authorization, Access-Control-Request-Method, Access-Control-Request-Headers".to_string(),
        allow_credentials: true,
        max_age: Some(86400),
        exposed_headers: Some("X-Custom-Header".to_string()),
    };

    // Create TLS configuration with the existing certificate and key
    let tls_config = TlsConfig::Manual {
        cert_path: "certs/localhost.example.crt".to_string(),
        key_path: "certs/localhost.example.key".to_string(),
    };

    // Create the HTTP/2 server configuration
    let config = Http2ServerConfig {
        addr: SocketAddr::from(([127, 0, 0, 1], 8090)),
        tls_config: Some(tls_config),
        cors_config: Some(cors_config),
    };

    println!("Starting HTTP/2 server with CORS support on https://127.0.0.1:8090");
    println!("You can test it with a web browser or tools like curl:");
    println!("curl -v -k -X OPTIONS https://127.0.0.1:8090/message -H \"Origin: https://localhost:3000\"");
    println!("curl -v -k -X POST https://127.0.0.1:8090/message -H \"Origin: https://localhost:3000\" -H \"Content-Type: application/json\" -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}}'");
    println!("Note: The -k flag is used to skip certificate verification for self-signed certificates");

    // Start the HTTP/2 server
    let server_handle = start_http2_server(config, |message| {
        println!("Received message: {:?}", message);

        // Echo the message back
        Ok(message)
    }).await?;

    // Keep the server running until Ctrl+C is pressed
    signal::ctrl_c().await?;
    println!("Shutting down server...");

    // Stop the server
    server_handle.stop().await?;

    Ok(())
}
