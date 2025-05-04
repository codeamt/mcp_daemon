use mcp_daemon::{error, Result};

#[test]
fn test_error_helper_functions() {
    // Test prompt_not_found
    let error = error::prompt_not_found("test_prompt");
    assert!(format!("{:?}", error).contains("Prompt not found"));

    // Test tool_not_found
    let error = error::tool_not_found("test_tool");
    assert!(format!("{:?}", error).contains("Tool not found"));

    // Test resource_not_found
    let error = error::resource_not_found("test_uri");
    assert!(format!("{:?}", error).contains("Resource not found"));

    // Test resource_template_not_found
    let error = error::resource_template_not_found("test_template");
    assert!(format!("{:?}", error).contains("Resource template not found"));

    // Test invalid_request
    let error = error::invalid_request("test reason");
    assert!(format!("{:?}", error).contains("test reason"));
}

#[test]
fn test_result_type() {
    // Test the Result type
    fn test_function() -> Result<i32> {
        Ok(42)
    }

    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
