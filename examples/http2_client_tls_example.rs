use mcp_daemon::transport::Http2Builder;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    println!("HTTP/2 Client TLS Example");
    println!("=========================");
    println!("This example demonstrates different TLS configurations for the HTTP/2 client.");
    println!("The implementation now supports TLS connections with hyper-rustls.");

    // Example 1: No TLS (plain HTTP)
    println!("\n1. Creating a client with no TLS (plain HTTP)");
    let _client_transport = Http2Builder::new()
        .with_tls(false)
        .with_host("localhost".to_string())
        .with_port(8080)
        .build();

    println!("   Client created with URL: http://localhost:8080");
    println!("   TLS: Disabled");

    // Example 2: Default TLS with system root certificates
    println!("\n2. Creating a client with default TLS (system root certificates)");
    let _client_transport = Http2Builder::new()
        .with_tls(true)
        .with_host("localhost".to_string())
        .with_port(8443)
        .build();

    println!("   Client created with URL: https://localhost:8443");
    println!("   TLS: Enabled with system root certificates");

    // Example 3: Custom TLS with specific root certificate
    println!("\n3. Creating a client with custom TLS (specific root certificate)");
    let _client_transport = Http2Builder::new()
        .with_custom_tls("localhost.example.crt".to_string(), true)
        .with_host("localhost".to_string())
        .with_port(8443)
        .build();

    println!("   Client created with URL: https://localhost:8443");
    println!("   TLS: Enabled with custom root certificate");
    println!("   Certificate: localhost.example.crt");
    println!("   Verify Server: Yes");

    // Example 4: Custom TLS with verification disabled
    println!("\n4. Creating a client with custom TLS (verification disabled)");
    let _client_transport = Http2Builder::new()
        .with_custom_tls("localhost.example.crt".to_string(), false)
        .with_host("localhost".to_string())
        .with_port(8443)
        .build();

    println!("   Client created with URL: https://localhost:8443");
    println!("   TLS: Enabled with custom root certificate");
    println!("   Certificate: localhost.example.crt");
    println!("   Verify Server: No");

    // Example 5: Mutual TLS with client certificate
    println!("\n5. Creating a client with mutual TLS (client certificate)");
    let _client_transport = Http2Builder::new()
        .with_custom_tls("localhost.example.crt".to_string(), true)
        .with_client_cert("client.crt".to_string(), "client.key".to_string())
        .with_host("localhost".to_string())
        .with_port(8443)
        .build();

    println!("   Client created with URL: https://localhost:8443");
    println!("   TLS: Enabled with custom root certificate");
    println!("   Certificate: localhost.example.crt");
    println!("   Client Certificate: client.crt");
    println!("   Client Key: client.key");
    println!("   Verify Server: Yes");

    // Example 6: TLS with SNI
    println!("\n6. Creating a client with SNI (Server Name Indication)");
    let _client_transport = Http2Builder::new()
        .with_custom_tls("localhost.example.crt".to_string(), true)
        .with_sni("example.com".to_string())
        .with_host("localhost".to_string())
        .with_port(8443)
        .build();

    println!("   Client created with URL: https://localhost:8443");
    println!("   TLS: Enabled with custom root certificate");
    println!("   Certificate: localhost.example.crt");
    println!("   SNI: example.com");
    println!("   Verify Server: Yes");

    println!("\nImplementation Status:");
    println!("✅ Added hyper-rustls dependency for proper TLS support");
    println!("✅ Implemented client certificate support for mutual TLS");
    println!("✅ Added explicit SNI support for multi-domain servers");
    println!("⏳ Implement connection pooling and timeouts (coming soon)");

    Ok(())
}
