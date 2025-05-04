//! Session handling utilities for MCP protocol
//!
//! This module provides utilities for handling session-related aspects of the MCP protocol,
//! such as request cancellation. These utilities are designed to be used with the jsoncall
//! library's session management system.

use jsoncall::{Hook, RequestId, SessionContext};

use crate::schema::CancelledNotificationParams;

/// Hook for handling request cancellation in the MCP protocol.
///
/// The `CancellationHook` implements the jsoncall `Hook` trait to provide a way
/// to notify clients when a request has been cancelled. When a request is cancelled,
/// this hook sends a `notifications/cancelled` notification to the client with the
/// ID of the cancelled request.
///
/// This is an important part of the MCP protocol as it allows clients to clean up
/// resources associated with cancelled requests and avoid showing results for
/// operations that are no longer needed.
///
/// # Examples
///
/// ```ignore
/// // Example of how CancellationHook would be used with a server framework
/// // This is conceptual and not meant to be run as a doctest
/// use some_server_framework::Builder;
/// use mcp_daemon::request::session::CancellationHook;
///
/// let server = Builder::new()
///     .hook(CancellationHook)
///     .build();
/// // Now the server will automatically send cancellation notifications
/// ```
pub struct CancellationHook;

impl Hook for CancellationHook {
    /// Handles the cancellation of an outgoing request by sending a notification.
    ///
    /// This method is called by the jsoncall framework when a request is cancelled.
    /// It sends a `notifications/cancelled` notification to the client, including
    /// the ID of the cancelled request.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the request that was cancelled
    /// * `session` - The session context used to send the notification
    fn cancel_outgoing_request(&self, id: RequestId, session: &SessionContext) {
        session
            .notification(
                "notifications/cancelled",
                Some(&CancelledNotificationParams {
                    request_id: id,
                    reason: None,
                }),
            )
            .unwrap()
    }
}
