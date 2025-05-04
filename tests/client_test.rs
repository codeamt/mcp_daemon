use mcp_daemon::{
    client::ClientBuilder,
    schema::Root,
};

#[test]
fn test_client_builder() {
    // Test default client builder
    let builder = ClientBuilder::new();
    let (_, _, params) = builder.build_raw();
    
    // Verify default capabilities
    assert!(params.capabilities.roots.is_none());
    assert!(params.capabilities.sampling.is_empty());
    
    // Test with roots
    let roots = vec![
        Root {
            name: Some("test_root".to_string()),
            uri: "file:///test/path".to_string(),
        }
    ];
    
    let builder = ClientBuilder::new().with_roots(roots);
    let (_, _, params) = builder.build_raw();
    
    // Verify roots capability
    assert!(params.capabilities.roots.is_some());
    let roots_capability = params.capabilities.roots.unwrap();
    assert_eq!(roots_capability.list_changed, Some(true));
    
    // Test with expose_internals
    let builder = ClientBuilder::new().with_expose_internals(true);
    let (_, options, _) = builder.build_raw();
    
    // Verify expose_internals option
    assert_eq!(options.expose_internals, Some(true));
}
