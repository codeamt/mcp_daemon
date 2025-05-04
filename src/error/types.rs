use jsoncall::{Error, ErrorCode, ErrorObject};
use serde_json::json;

/// Creates an error for when a requested prompt is not found.
///
/// This function generates a standardized JSON-RPC error with the METHOD_NOT_FOUND error code
/// and a message indicating that the prompt was not found. This should be used when a client
/// requests a prompt that doesn't exist or is not available.
///
/// # Parameters
///
/// * `name` - The name of the prompt that was not found
///
/// # Returns
///
/// A JSON-RPC Error object with error code METHOD_NOT_FOUND
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::error::prompt_not_found;
/// 
/// let error = prompt_not_found("non_existent_prompt");
/// // Return this error in a response to the client
/// ```
pub fn prompt_not_found(_name: &str) -> Error {
    Error::new(ErrorCode::METHOD_NOT_FOUND).with_message("Prompt not found", true)
}

/// Creates an error for when a requested tool is not found.
///
/// This function generates a standardized JSON-RPC error with the METHOD_NOT_FOUND error code
/// and a message indicating that the tool was not found. This should be used when a client
/// attempts to use a tool that doesn't exist or is not available.
///
/// # Parameters
///
/// * `name` - The name of the tool that was not found
///
/// # Returns
///
/// A JSON-RPC Error object with error code METHOD_NOT_FOUND
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::error::tool_not_found;
/// 
/// let error = tool_not_found("non_existent_tool");
/// // Return this error in a response to the client
/// ```
pub fn tool_not_found(_name: &str) -> Error {
    Error::new(ErrorCode::METHOD_NOT_FOUND).with_message("Tool not found", true)
}

/// Creates an error for when a requested resource is not found.
///
/// This function generates a standardized JSON-RPC error with the INVALID_PARAMS error code
/// and a message indicating that the resource was not found. It also includes the URI of the
/// requested resource in the error data. This should be used when a client attempts to access
/// a resource that doesn't exist or is not available.
///
/// # Parameters
///
/// * `uri` - The URI of the resource that was not found
///
/// # Returns
///
/// A JSON-RPC Error object with error code INVALID_PARAMS and data containing the resource URI
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::error::resource_not_found;
/// 
/// let error = resource_not_found("my_app://resources/missing");
/// // Return this error in a response to the client
/// ```
pub fn resource_not_found(uri: &str) -> Error {
    ErrorObject {
        code: ErrorCode::INVALID_PARAMS,
        message: "Resource not found".to_string(),
        data: Some(json!({ "uri": uri })),
    }
    .into()
}

/// Creates an error for when a requested resource template is not found.
///
/// This function generates a standardized JSON-RPC error with the INVALID_PARAMS error code
/// and a message indicating that the resource template was not found. It also includes the
/// template name in the error data. This should be used when a client attempts to use a
/// resource template that doesn't exist or is not available.
///
/// # Parameters
///
/// * `template` - The name of the resource template that was not found
///
/// # Returns
///
/// A JSON-RPC Error object with error code INVALID_PARAMS and data containing the template name
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::error::resource_template_not_found;
/// 
/// let error = resource_template_not_found("missing_template");
/// // Return this error in a response to the client
/// ```
pub fn resource_template_not_found(template: &str) -> Error {
    ErrorObject {
        code: ErrorCode::INVALID_PARAMS,
        message: "Resource template not found".to_string(),
        data: Some(json!({ "template": template })),
    }
    .into()
}

/// Creates an error for when a request is invalid for a specified reason.
///
/// This function generates a standardized JSON-RPC error with the INVALID_PARAMS error code
/// and a message indicating why the request is invalid. This is a general-purpose error
/// function that should be used when a client sends a request that cannot be processed due
/// to issues with the request parameters.
///
/// # Parameters
///
/// * `reason` - The reason why the request is invalid
///
/// # Returns
///
/// A JSON-RPC Error object with error code INVALID_PARAMS and the specified reason as the message
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::error::invalid_request;
/// 
/// let error = invalid_request("Missing required parameter 'id'");
/// // Return this error in a response to the client
/// ```
pub fn invalid_request(reason: &str) -> Error {
    Error::new(ErrorCode::INVALID_PARAMS).with_message(reason, true)
}
