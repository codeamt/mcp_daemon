//! Protocol-specific conversions between schema types.
//!
//! This module provides a large number of conversion implementations that allow
//! for seamless conversion between different MCP protocol schema types. These
//! conversions are essential for the proper functioning of the MCP protocol and
//! enable convenient usage patterns for both client and server implementations.
//!
//! Most of these conversions are automatically generated and follow a consistent
//! pattern of implementing the `From` trait for various combinations of schema types.
//! This enables ergonomic construction of response types from different input formats.
//!
//! Example conversions include:
//! - Converting embedded resources to various result types
//! - Converting between client and server notification/request types
//! - Converting basic types like strings and vecs to appropriate schema types

use crate::schema::*;
use base64::Engine;

// These implementations are necessary for the protocol conversions to work correctly
impl ::std::convert::From<String> for TextContent {
    fn from(text: String) -> Self {
        TextContent {
            text,
            annotations: None,
            type_: "text".to_string(),
        }
    }
}

impl ::std::convert::From<&str> for TextContent {
    fn from(text: &str) -> Self {
        TextContent {
            text: text.to_string(),
            annotations: None,
            type_: "text".to_string(),
        }
    }
}

impl ::std::convert::From<CallToolResultContentItem> for CallToolResult {
    fn from(content: CallToolResultContentItem) -> Self {
        CallToolResult {
            content: vec![content],
            is_error: None,
            meta: Default::default(),
        }
    }
}

impl ::std::convert::From<()> for CallToolResult {
    fn from(_: ()) -> Self {
        CallToolResult {
            content: Vec::new(),
            is_error: None,
            meta: Default::default(),
        }
    }
}

impl<T: Into<CallToolResultContentItem>> ::std::convert::From<Vec<T>> for CallToolResult {
    fn from(content: Vec<T>) -> Self {
        CallToolResult {
            content: content.into_iter().map(|c| c.into()).collect(),
            is_error: None,
            meta: Default::default(),
        }
    }
}

/// Creates a `TextResourceContents` from a String.
///
/// This conversion provides a simple way to create text resource contents when you only have
/// the text content. It automatically sets `mime_type` to `None` and `uri` to an empty string.
impl ::std::convert::From<String> for TextResourceContents {
    fn from(text: String) -> Self {
        TextResourceContents {
            text,
            mime_type: None,
            uri: String::new(),
        }
    }
}

/// Creates a `TextResourceContents` from a string slice.
///
/// This conversion provides a convenient way to create text resource contents directly
/// from a string literal or slice without having to call `.to_string()` first.
impl ::std::convert::From<&str> for TextResourceContents {
    fn from(text: &str) -> Self {
        text.to_string().into()
    }
}

/// Creates a `BlobResourceContents` from `Base64Bytes`.
///
/// This conversion simplifies the creation of blob resource contents from Base64-encoded data.
impl ::std::convert::From<Base64Bytes> for BlobResourceContents {
    fn from(blob: Base64Bytes) -> Self {
        BlobResourceContents {
            blob: base64::prelude::BASE64_STANDARD.encode(&blob.0),
            mime_type: None,
            uri: String::new(),
        }
    }
}
impl ::std::convert::From<&EmbeddedResource> for CallToolResult {
    fn from(value: &EmbeddedResource) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmbeddedResource> for PromptMessageContent {
    fn from(value: &EmbeddedResource) -> Self {
        <PromptMessageContent as ::std::convert::From<EmbeddedResource>>::from(
            <EmbeddedResource as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmbeddedResource> for PromptMessage {
    fn from(value: &EmbeddedResource) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmbeddedResource> for CallToolResultContentItem {
    fn from(value: &EmbeddedResource) -> Self {
        <CallToolResultContentItem as ::std::convert::From<EmbeddedResource>>::from(
            <EmbeddedResource as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmbeddedResource> for ServerResult {
    fn from(value: &EmbeddedResource) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmbeddedResource> for GetPromptResult {
    fn from(value: &EmbeddedResource) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListResourcesResult> for ServerResult {
    fn from(value: &ListResourcesResult) -> Self {
        <ServerResult as ::std::convert::From<ListResourcesResult>>::from(
            <ListResourcesResult as ::std::convert::From<&ListResourcesResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&ReadResourceRequest> for ClientRequest {
    fn from(value: &ReadResourceRequest) -> Self {
        <ClientRequest as ::std::convert::From<ReadResourceRequest>>::from(
            <ReadResourceRequest as ::std::convert::From<&ReadResourceRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&ReadResourceResultContentsItem> for ReadResourceResult {
    fn from(value: &ReadResourceResultContentsItem) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<
                &ReadResourceResultContentsItem,
            >>::from(value),
        )
    }
}
impl ::std::convert::From<&ReadResourceResultContentsItem> for ServerResult {
    fn from(value: &ReadResourceResultContentsItem) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<&ReadResourceResultContentsItem>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&CancelledNotification> for ServerNotification {
    fn from(value: &CancelledNotification) -> Self {
        <ServerNotification as ::std::convert::From<CancelledNotification>>::from(
            <CancelledNotification as ::std::convert::From<&CancelledNotification>>::from(value),
        )
    }
}
impl ::std::convert::From<&CancelledNotification> for ClientNotification {
    fn from(value: &CancelledNotification) -> Self {
        <ClientNotification as ::std::convert::From<CancelledNotification>>::from(
            <CancelledNotification as ::std::convert::From<&CancelledNotification>>::from(value),
        )
    }
}
// Implementation moved to schema_ext.rs to avoid conflicting implementations
// impl ::std::convert::From<TextResourceContents> for ReadResourceResult {...}
impl ::std::convert::From<TextResourceContents> for ServerResult {
    fn from(value: TextResourceContents) -> Self {
        let read_resource_result = ReadResourceResult {
            contents: vec![ReadResourceResultContentsItem::TextResourceContents(value)],
            meta: Default::default(),
        };
        ServerResult::ReadResourceResult(read_resource_result)
    }
}
impl ::std::convert::From<&PromptMessage> for GetPromptResult {
    fn from(value: &PromptMessage) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&PromptMessage>>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptMessage> for ServerResult {
    fn from(value: &PromptMessage) -> Self {
        <ServerResult as ::std::convert::From<GetPromptResult>>::from(
            <GetPromptResult as ::std::convert::From<&PromptMessage>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListResourceTemplatesRequest> for ClientRequest {
    fn from(value: &ListResourceTemplatesRequest) -> Self {
        ClientRequest::ListResourcesRequest(ListResourcesRequest {
            method: value.method.clone(),
            params: value.params.clone().map(|p| ListResourcesRequestParams {
                cursor: p.cursor,
            }),
        })
    }
}

impl ::std::convert::From<ListResourceTemplatesRequest> for ClientRequest {
    fn from(value: ListResourceTemplatesRequest) -> Self {
        ClientRequest::ListResourcesRequest(ListResourcesRequest {
            method: value.method,
            params: value.params.map(|p| ListResourcesRequestParams {
                cursor: p.cursor,
            }),
        })
    }
}
impl ::std::convert::From<PromptMessageContent> for ServerResult {
    fn from(value: PromptMessageContent) -> Self {
        <ServerResult as ::std::convert::From<GetPromptResult>>::from(
            <GetPromptResult as ::std::convert::From<PromptMessageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<PromptMessageContent> for GetPromptResult {
    fn from(value: PromptMessageContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&[&str]> for ServerResult {
    fn from(value: &[&str]) -> Self {
        <ServerResult as ::std::convert::From<CompleteResult>>::from(
            <CompleteResult as ::std::convert::From<&[&str]>>::from(value),
        )
    }
}
impl ::std::convert::From<&[&str]> for CompleteResult {
    fn from(value: &[&str]) -> Self {
        <CompleteResult as ::std::convert::From<CompleteResultCompletion>>::from(
            <CompleteResultCompletion as ::std::convert::From<&[&str]>>::from(value),
        )
    }
}
impl ::std::convert::From<Vec<String>> for CompleteResult {
    fn from(value: Vec<String>) -> Self {
        <CompleteResult as ::std::convert::From<CompleteResultCompletion>>::from(
            <CompleteResultCompletion as ::std::convert::From<Vec<String>>>::from(value),
        )
    }
}
impl ::std::convert::From<Vec<String>> for ServerResult {
    fn from(value: Vec<String>) -> Self {
        <ServerResult as ::std::convert::From<CompleteResult>>::from(
            <CompleteResult as ::std::convert::From<Vec<String>>>::from(value),
        )
    }
}
impl ::std::convert::From<&RootsListChangedNotification> for ClientNotification {
    fn from(value: &RootsListChangedNotification) -> Self {
        < ClientNotification as :: std :: convert :: From < RootsListChangedNotification >> :: from (< RootsListChangedNotification as :: std :: convert :: From < & RootsListChangedNotification >> :: from (value))
    }
}
impl ::std::convert::From<&ToolListChangedNotification> for ServerNotification {
    fn from(value: &ToolListChangedNotification) -> Self {
        < ServerNotification as :: std :: convert :: From < ToolListChangedNotification >> :: from (< ToolListChangedNotification as :: std :: convert :: From < & ToolListChangedNotification >> :: from (value))
    }
}
impl ::std::convert::From<Vec<Root>> for ClientResult {
    fn from(value: Vec<Root>) -> Self {
        <ClientResult as ::std::convert::From<ListRootsResult>>::from(
            <ListRootsResult as ::std::convert::From<Vec<Root>>>::from(value),
        )
    }
}
impl ::std::convert::From<Vec<Resource>> for ServerResult {
    fn from(value: Vec<Resource>) -> Self {
        <ServerResult as ::std::convert::From<ListResourcesResult>>::from(
            <ListResourcesResult as ::std::convert::From<Vec<Resource>>>::from(value),
        )
    }
}
impl ::std::convert::From<&GetPromptRequest> for ClientRequest {
    fn from(value: &GetPromptRequest) -> Self {
        <ClientRequest as ::std::convert::From<GetPromptRequest>>::from(
            <GetPromptRequest as ::std::convert::From<&GetPromptRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for PromptMessage {
    fn from(value: &TextContent) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for CreateMessageResultContent {
    fn from(value: &TextContent) -> Self {
        <CreateMessageResultContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for GetPromptResult {
    fn from(value: &TextContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for CallToolResultContentItem {
    fn from(value: &TextContent) -> Self {
        <CallToolResultContentItem as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for PromptMessageContent {
    fn from(value: &TextContent) -> Self {
        <PromptMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for SamplingMessageContent {
    fn from(value: &TextContent) -> Self {
        <SamplingMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for ServerResult {
    fn from(value: &TextContent) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextContent> for CallToolResult {
    fn from(value: &TextContent) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<&TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&JsonrpcRequest> for JsonrpcMessage {
    fn from(value: &JsonrpcRequest) -> Self {
        Self {
            subtype_0: Some(value.clone()),
            subtype_1: None,
            subtype_2: None,
            subtype_3: None,
            subtype_4: None,
            subtype_5: None,
        }
    }
}

impl ::std::convert::From<JsonrpcRequest> for JsonrpcMessage {
    fn from(value: JsonrpcRequest) -> Self {
        Self {
            subtype_0: Some(value),
            subtype_1: None,
            subtype_2: None,
            subtype_3: None,
            subtype_4: None,
            subtype_5: None,
        }
    }
}
impl ::std::convert::From<&CreateMessageResult> for ClientResult {
    fn from(value: &CreateMessageResult) -> Self {
        <ClientResult as ::std::convert::From<CreateMessageResult>>::from(
            <CreateMessageResult as ::std::convert::From<&CreateMessageResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListRootsRequest> for ServerRequest {
    fn from(value: &ListRootsRequest) -> Self {
        <ServerRequest as ::std::convert::From<ListRootsRequest>>::from(
            <ListRootsRequest as ::std::convert::From<&ListRootsRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for CallToolResultContentItem {
    fn from(value: String) -> Self {
        <CallToolResultContentItem as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for CallToolResult {
    fn from(value: String) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for ReadResourceResultContentsItem {
    fn from(value: String) -> Self {
        <ReadResourceResultContentsItem as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for PromptMessage {
    fn from(value: String) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for SamplingMessageContent {
    fn from(value: String) -> Self {
        <SamplingMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for GetPromptResult {
    fn from(value: String) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for ReadResourceResult {
    fn from(value: String) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for PromptMessageContent {
    fn from(value: String) -> Self {
        <PromptMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for CreateMessageResultContent {
    fn from(value: String) -> Self {
        <CreateMessageResultContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<String> for EmbeddedResourceResource {
    fn from(value: String) -> Self {
        <EmbeddedResourceResource as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<String>>::from(value),
        )
    }
}
impl ::std::convert::From<&PingRequest> for ClientRequest {
    fn from(value: &PingRequest) -> Self {
        <ClientRequest as ::std::convert::From<PingRequest>>::from(
            <PingRequest as ::std::convert::From<&PingRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&PingRequest> for ServerRequest {
    fn from(value: &PingRequest) -> Self {
        <ServerRequest as ::std::convert::From<PingRequest>>::from(
            <PingRequest as ::std::convert::From<&PingRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&UnsubscribeRequest> for ClientRequest {
    fn from(value: &UnsubscribeRequest) -> Self {
        <ClientRequest as ::std::convert::From<UnsubscribeRequest>>::from(
            <UnsubscribeRequest as ::std::convert::From<&UnsubscribeRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&CallToolRequest> for ClientRequest {
    fn from(value: &CallToolRequest) -> Self {
        <ClientRequest as ::std::convert::From<CallToolRequest>>::from(
            <CallToolRequest as ::std::convert::From<&CallToolRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<CallToolResultContentItem> for ServerResult {
    fn from(value: CallToolResultContentItem) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(value),
        )
    }
}
impl ::std::convert::From<&JsonrpcNotification> for JsonrpcMessage {
    fn from(value: &JsonrpcNotification) -> Self {
        Self {
            subtype_0: None,
            subtype_1: Some(value.clone()),
            subtype_2: None,
            subtype_3: None,
            subtype_4: None,
            subtype_5: None,
        }
    }
}

impl ::std::convert::From<JsonrpcNotification> for JsonrpcMessage {
    fn from(value: JsonrpcNotification) -> Self {
        Self {
            subtype_0: None,
            subtype_1: Some(value),
            subtype_2: None,
            subtype_3: None,
            subtype_4: None,
            subtype_5: None,
        }
    }
}
impl ::std::convert::From<&BlobResourceContents> for EmbeddedResourceResource {
    fn from(value: &BlobResourceContents) -> Self {
        <EmbeddedResourceResource as ::std::convert::From<BlobResourceContents>>::from(
            <BlobResourceContents as ::std::convert::From<&BlobResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&BlobResourceContents> for ReadResourceResultContentsItem {
    fn from(value: &BlobResourceContents) -> Self {
        <ReadResourceResultContentsItem as ::std::convert::From<BlobResourceContents>>::from(
            <BlobResourceContents as ::std::convert::From<&BlobResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&BlobResourceContents> for ReadResourceResult {
    fn from(value: &BlobResourceContents) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<&BlobResourceContents>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&BlobResourceContents> for ServerResult {
    fn from(value: &BlobResourceContents) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<&BlobResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&ProgressNotification> for ClientNotification {
    fn from(value: &ProgressNotification) -> Self {
        <ClientNotification as ::std::convert::From<ProgressNotification>>::from(
            <ProgressNotification as ::std::convert::From<&ProgressNotification>>::from(value),
        )
    }
}
impl ::std::convert::From<&ProgressNotification> for ServerNotification {
    fn from(value: &ProgressNotification) -> Self {
        <ServerNotification as ::std::convert::From<ProgressNotification>>::from(
            <ProgressNotification as ::std::convert::From<&ProgressNotification>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for GetPromptResult {
    fn from(value: &str) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for CreateMessageResultContent {
    fn from(value: &str) -> Self {
        <CreateMessageResultContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for SamplingMessageContent {
    fn from(value: &str) -> Self {
        <SamplingMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for CallToolResultContentItem {
    fn from(value: &str) -> Self {
        <CallToolResultContentItem as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for ReadResourceResultContentsItem {
    fn from(value: &str) -> Self {
        <ReadResourceResultContentsItem as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for ReadResourceResult {
    fn from(value: &str) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for PromptMessage {
    fn from(value: &str) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for CallToolResult {
    fn from(value: &str) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for EmbeddedResourceResource {
    fn from(value: &str) -> Self {
        <EmbeddedResourceResource as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<&str> for PromptMessageContent {
    fn from(value: &str) -> Self {
        <PromptMessageContent as ::std::convert::From<TextContent>>::from(
            <TextContent as ::std::convert::From<&str>>::from(value),
        )
    }
}
impl ::std::convert::From<EmbeddedResource> for GetPromptResult {
    fn from(value: EmbeddedResource) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<EmbeddedResource> for CallToolResult {
    fn from(value: EmbeddedResource) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<EmbeddedResource> for PromptMessage {
    fn from(value: EmbeddedResource) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<EmbeddedResource> for ServerResult {
    fn from(value: EmbeddedResource) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<EmbeddedResource>>::from(value),
        )
    }
}
impl ::std::convert::From<&ReadResourceResult> for ServerResult {
    fn from(value: &ReadResourceResult) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<&ReadResourceResult>>::from(value),
        )
    }
}
impl ::std::convert::From<BlobResourceContents> for ServerResult {
    fn from(value: BlobResourceContents) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<BlobResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<BlobResourceContents> for ReadResourceResult {
    fn from(value: BlobResourceContents) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<BlobResourceContents>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&ImageContent> for GetPromptResult {
    fn from(value: &ImageContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for CallToolResultContentItem {
    fn from(value: &ImageContent) -> Self {
        <CallToolResultContentItem as ::std::convert::From<ImageContent>>::from(
            <ImageContent as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for PromptMessage {
    fn from(value: &ImageContent) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for CallToolResult {
    fn from(value: &ImageContent) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for PromptMessageContent {
    fn from(value: &ImageContent) -> Self {
        <PromptMessageContent as ::std::convert::From<ImageContent>>::from(
            <ImageContent as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for SamplingMessageContent {
    fn from(value: &ImageContent) -> Self {
        <SamplingMessageContent as ::std::convert::From<ImageContent>>::from(
            <ImageContent as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for ServerResult {
    fn from(value: &ImageContent) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ImageContent> for CreateMessageResultContent {
    fn from(value: &ImageContent) -> Self {
        <CreateMessageResultContent as ::std::convert::From<ImageContent>>::from(
            <ImageContent as ::std::convert::From<&ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&Result> for EmptyResult {
    fn from(value: &Result) -> Self {
        <EmptyResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            &Result,
        >>::from(value))
    }
}
impl ::std::convert::From<&Result> for ClientResult {
    fn from(value: &Result) -> Self {
        <ClientResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            &Result,
        >>::from(value))
    }
}
impl ::std::convert::From<&Result> for ServerResult {
    fn from(value: &Result) -> Self {
        <ServerResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            &Result,
        >>::from(value))
    }
}
impl ::std::convert::From<&JsonrpcError> for JsonrpcMessage {
    fn from(value: &JsonrpcError) -> Self {
        Self {
            subtype_0: None,
            subtype_1: None,
            subtype_2: None,
            subtype_3: None,
            subtype_4: Some(value.clone()),
            subtype_5: None,
        }
    }
}

impl ::std::convert::From<JsonrpcError> for JsonrpcMessage {
    fn from(value: JsonrpcError) -> Self {
        Self {
            subtype_0: None,
            subtype_1: None,
            subtype_2: None,
            subtype_3: None,
            subtype_4: Some(value),
            subtype_5: None,
        }
    }
}
impl ::std::convert::From<&ListResourceTemplatesResult> for ServerResult {
    fn from(value: &ListResourceTemplatesResult) -> Self {
        Self::ListResourcesResult(ListResourcesResult {
            meta: value.meta.clone(),
            next_cursor: value.next_cursor.clone(),
            resources: value.resource_templates.iter().map(|rt| Resource {
                uri: rt.uri_template.clone(),
                name: rt.name.clone(),
                description: rt.description.clone(),
                mime_type: rt.mime_type.clone(),
                annotations: rt.annotations.clone(),
            }).collect(),
        })
    }
}

impl ::std::convert::From<ListResourceTemplatesResult> for ServerResult {
    fn from(value: ListResourceTemplatesResult) -> Self {
        Self::ListResourcesResult(ListResourcesResult {
            meta: value.meta,
            next_cursor: value.next_cursor,
            resources: value.resource_templates.into_iter().map(|rt| Resource {
                uri: rt.uri_template,
                name: rt.name,
                description: rt.description,
                mime_type: rt.mime_type,
                annotations: rt.annotations,
            }).collect(),
        })
    }
}
impl ::std::convert::From<&ResourceListChangedNotification> for ServerNotification {
    fn from(value: &ResourceListChangedNotification) -> Self {
        <ServerNotification as ::std::convert::From<ResourceListChangedNotification>>::from(
            <ResourceListChangedNotification as ::std::convert::From<
                &ResourceListChangedNotification,
            >>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptMessageContent> for ServerResult {
    fn from(value: &PromptMessageContent) -> Self {
        <ServerResult as ::std::convert::From<GetPromptResult>>::from(
            <GetPromptResult as ::std::convert::From<&PromptMessageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptMessageContent> for GetPromptResult {
    fn from(value: &PromptMessageContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<&PromptMessageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptMessageContent> for PromptMessage {
    fn from(value: &PromptMessageContent) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<&PromptMessageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&CompleteResult> for ServerResult {
    fn from(value: &CompleteResult) -> Self {
        <ServerResult as ::std::convert::From<CompleteResult>>::from(
            <CompleteResult as ::std::convert::From<&CompleteResult>>::from(value),
        )
    }
}
impl ::std::convert::From<PromptMessage> for ServerResult {
    fn from(value: PromptMessage) -> Self {
        <ServerResult as ::std::convert::From<GetPromptResult>>::from(
            <GetPromptResult as ::std::convert::From<PromptMessage>>::from(value),
        )
    }
}
impl ::std::convert::From<&InitializeResult> for ServerResult {
    fn from(value: &InitializeResult) -> Self {
        <ServerResult as ::std::convert::From<InitializeResult>>::from(
            <InitializeResult as ::std::convert::From<&InitializeResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptListChangedNotification> for ServerNotification {
    fn from(value: &PromptListChangedNotification) -> Self {
        <ServerNotification as ::std::convert::From<PromptListChangedNotification>>::from(
            <PromptListChangedNotification as ::std::convert::From<
                &PromptListChangedNotification,
            >>::from(value),
        )
    }
}
impl ::std::convert::From<&CallToolResultContentItem> for CallToolResult {
    fn from(value: &CallToolResultContentItem) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<&CallToolResultContentItem>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&CallToolResultContentItem> for ServerResult {
    fn from(value: &CallToolResultContentItem) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<&CallToolResultContentItem>>::from(value),
        )
    }
}
impl ::std::convert::From<&CompleteRequest> for ClientRequest {
    fn from(value: &CompleteRequest) -> Self {
        <ClientRequest as ::std::convert::From<CompleteRequest>>::from(
            <CompleteRequest as ::std::convert::From<&CompleteRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&EmptyResult> for Result {
    fn from(value: &EmptyResult) -> Self {
        <Result as ::std::convert::From<EmptyResult>>::from(<EmptyResult as ::std::convert::From<
            &EmptyResult,
        >>::from(value))
    }
}
impl ::std::convert::From<&EmptyResult> for ClientResult {
    fn from(value: &EmptyResult) -> Self {
        <ClientResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            &EmptyResult,
        >>::from(value))
    }
}
impl ::std::convert::From<&EmptyResult> for ServerResult {
    fn from(value: &EmptyResult) -> Self {
        <ServerResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            &EmptyResult,
        >>::from(value))
    }
}
impl ::std::convert::From<&ListToolsResult> for ServerResult {
    fn from(value: &ListToolsResult) -> Self {
        <ServerResult as ::std::convert::From<ListToolsResult>>::from(
            <ListToolsResult as ::std::convert::From<&ListToolsResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&SetLevelRequest> for ClientRequest {
    fn from(value: &SetLevelRequest) -> Self {
        <ClientRequest as ::std::convert::From<SetLevelRequest>>::from(
            <SetLevelRequest as ::std::convert::From<&SetLevelRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&CompleteResultCompletion> for ServerResult {
    fn from(value: &CompleteResultCompletion) -> Self {
        <ServerResult as ::std::convert::From<CompleteResult>>::from(
            <CompleteResult as ::std::convert::From<&CompleteResultCompletion>>::from(value),
        )
    }
}
impl ::std::convert::From<&CompleteResultCompletion> for CompleteResult {
    fn from(value: &CompleteResultCompletion) -> Self {
        <CompleteResult as ::std::convert::From<CompleteResultCompletion>>::from(
            <CompleteResultCompletion as ::std::convert::From<&CompleteResultCompletion>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&CreateMessageRequest> for ServerRequest {
    fn from(value: &CreateMessageRequest) -> Self {
        <ServerRequest as ::std::convert::From<CreateMessageRequest>>::from(
            <CreateMessageRequest as ::std::convert::From<&CreateMessageRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&ResourceReference> for CompleteRequestParamsRef {
    fn from(value: &ResourceReference) -> Self {
        <CompleteRequestParamsRef as ::std::convert::From<ResourceReference>>::from(
            <ResourceReference as ::std::convert::From<&ResourceReference>>::from(value),
        )
    }
}
impl ::std::convert::From<Base64Bytes> for EmbeddedResourceResource {
    fn from(value: Base64Bytes) -> Self {
        <EmbeddedResourceResource as ::std::convert::From<BlobResourceContents>>::from(
            <BlobResourceContents as ::std::convert::From<Base64Bytes>>::from(value),
        )
    }
}
impl ::std::convert::From<Base64Bytes> for ReadResourceResultContentsItem {
    fn from(value: Base64Bytes) -> Self {
        <ReadResourceResultContentsItem as ::std::convert::From<BlobResourceContents>>::from(
            <BlobResourceContents as ::std::convert::From<Base64Bytes>>::from(value),
        )
    }
}
impl ::std::convert::From<Base64Bytes> for ServerResult {
    fn from(value: Base64Bytes) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<Base64Bytes>>::from(value),
        )
    }
}
impl ::std::convert::From<Base64Bytes> for ReadResourceResult {
    fn from(value: Base64Bytes) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<Base64Bytes>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListResourcesRequest> for ClientRequest {
    fn from(value: &ListResourcesRequest) -> Self {
        <ClientRequest as ::std::convert::From<ListResourcesRequest>>::from(
            <ListResourcesRequest as ::std::convert::From<&ListResourcesRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<ImageContent> for ServerResult {
    fn from(value: ImageContent) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<ImageContent> for GetPromptResult {
    fn from(value: ImageContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<ImageContent> for PromptMessage {
    fn from(value: ImageContent) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<ImageContent> for CallToolResult {
    fn from(value: ImageContent) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<ImageContent>>::from(value),
        )
    }
}
impl ::std::convert::From<EmptyResult> for ClientResult {
    fn from(value: EmptyResult) -> Self {
        <ClientResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            EmptyResult,
        >>::from(value))
    }
}
impl ::std::convert::From<EmptyResult> for ServerResult {
    fn from(value: EmptyResult) -> Self {
        <ServerResult as ::std::convert::From<Result>>::from(<Result as ::std::convert::From<
            EmptyResult,
        >>::from(value))
    }
}
impl ::std::convert::From<Vec<ReadResourceResultContentsItem>> for ServerResult {
    fn from(value: Vec<ReadResourceResultContentsItem>) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<Vec<ReadResourceResultContentsItem>>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&LoggingMessageNotification> for ServerNotification {
    fn from(value: &LoggingMessageNotification) -> Self {
        <ServerNotification as ::std::convert::From<LoggingMessageNotification>>::from(
            <LoggingMessageNotification as ::std::convert::From<&LoggingMessageNotification>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<Vec<Tool>> for ServerResult {
    fn from(value: Vec<Tool>) -> Self {
        <ServerResult as ::std::convert::From<ListToolsResult>>::from(
            <ListToolsResult as ::std::convert::From<Vec<Tool>>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListToolsRequest> for ClientRequest {
    fn from(value: &ListToolsRequest) -> Self {
        <ClientRequest as ::std::convert::From<ListToolsRequest>>::from(
            <ListToolsRequest as ::std::convert::From<&ListToolsRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&InitializeRequest> for ClientRequest {
    fn from(value: &InitializeRequest) -> Self {
        <ClientRequest as ::std::convert::From<InitializeRequest>>::from(
            <InitializeRequest as ::std::convert::From<&InitializeRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<TextContent> for CallToolResult {
    fn from(value: TextContent) -> Self {
        <CallToolResult as ::std::convert::From<CallToolResultContentItem>>::from(
            <CallToolResultContentItem as ::std::convert::From<TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<TextContent> for GetPromptResult {
    fn from(value: TextContent) -> Self {
        <GetPromptResult as ::std::convert::From<PromptMessage>>::from(
            <PromptMessage as ::std::convert::From<TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<TextContent> for ServerResult {
    fn from(value: TextContent) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<TextContent> for PromptMessage {
    fn from(value: TextContent) -> Self {
        <PromptMessage as ::std::convert::From<PromptMessageContent>>::from(
            <PromptMessageContent as ::std::convert::From<TextContent>>::from(value),
        )
    }
}
impl ::std::convert::From<&ResourceUpdatedNotification> for ServerNotification {
    fn from(value: &ResourceUpdatedNotification) -> Self {
        < ServerNotification as :: std :: convert :: From < ResourceUpdatedNotification >> :: from (< ResourceUpdatedNotification as :: std :: convert :: From < & ResourceUpdatedNotification >> :: from (value))
    }
}
impl ::std::convert::From<&ListRootsResult> for ClientResult {
    fn from(value: &ListRootsResult) -> Self {
        <ClientResult as ::std::convert::From<ListRootsResult>>::from(
            <ListRootsResult as ::std::convert::From<&ListRootsResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&ListPromptsResult> for ServerResult {
    fn from(value: &ListPromptsResult) -> Self {
        <ServerResult as ::std::convert::From<ListPromptsResult>>::from(
            <ListPromptsResult as ::std::convert::From<&ListPromptsResult>>::from(value),
        )
    }
}
impl ::std::convert::From<CompleteResultCompletion> for ServerResult {
    fn from(value: CompleteResultCompletion) -> Self {
        <ServerResult as ::std::convert::From<CompleteResult>>::from(
            <CompleteResult as ::std::convert::From<CompleteResultCompletion>>::from(value),
        )
    }
}
impl ::std::convert::From<ReadResourceResultContentsItem> for ServerResult {
    fn from(value: ReadResourceResultContentsItem) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<()> for ServerResult {
    fn from(_: ()) -> Self {
        let call_tool_result = CallToolResult {
            content: Vec::new(),
            is_error: None,
            meta: Default::default(),
        };
        ServerResult::CallToolResult(call_tool_result)
    }
}
impl ::std::convert::From<&ListPromptsRequest> for ClientRequest {
    fn from(value: &ListPromptsRequest) -> Self {
        <ClientRequest as ::std::convert::From<ListPromptsRequest>>::from(
            <ListPromptsRequest as ::std::convert::From<&ListPromptsRequest>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextResourceContents> for EmbeddedResourceResource {
    fn from(value: &TextResourceContents) -> Self {
        <EmbeddedResourceResource as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<&TextResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextResourceContents> for ReadResourceResultContentsItem {
    fn from(value: &TextResourceContents) -> Self {
        <ReadResourceResultContentsItem as ::std::convert::From<TextResourceContents>>::from(
            <TextResourceContents as ::std::convert::From<&TextResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&TextResourceContents> for ReadResourceResult {
    fn from(value: &TextResourceContents) -> Self {
        <ReadResourceResult as ::std::convert::From<ReadResourceResultContentsItem>>::from(
            <ReadResourceResultContentsItem as ::std::convert::From<&TextResourceContents>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&TextResourceContents> for ServerResult {
    fn from(value: &TextResourceContents) -> Self {
        <ServerResult as ::std::convert::From<ReadResourceResult>>::from(
            <ReadResourceResult as ::std::convert::From<&TextResourceContents>>::from(value),
        )
    }
}
impl ::std::convert::From<&GetPromptResult> for ServerResult {
    fn from(value: &GetPromptResult) -> Self {
        <ServerResult as ::std::convert::From<GetPromptResult>>::from(
            <GetPromptResult as ::std::convert::From<&GetPromptResult>>::from(value),
        )
    }
}
impl ::std::convert::From<Vec<ResourceTemplate>> for ServerResult {
    fn from(value: Vec<ResourceTemplate>) -> Self {
        Self::ListResourcesResult(ListResourcesResult {
            meta: ::serde_json::Map::new(),
            next_cursor: None,
            resources: value.into_iter().map(|rt| Resource {
                uri: rt.uri_template,
                name: rt.name,
                description: rt.description,
                mime_type: rt.mime_type,
                annotations: rt.annotations,
            }).collect(),
        })
    }
}
impl ::std::convert::From<Vec<Prompt>> for ServerResult {
    fn from(value: Vec<Prompt>) -> Self {
        <ServerResult as ::std::convert::From<ListPromptsResult>>::from(
            <ListPromptsResult as ::std::convert::From<Vec<Prompt>>>::from(value),
        )
    }
}
impl ::std::convert::From<&CallToolResult> for ServerResult {
    fn from(value: &CallToolResult) -> Self {
        <ServerResult as ::std::convert::From<CallToolResult>>::from(
            <CallToolResult as ::std::convert::From<&CallToolResult>>::from(value),
        )
    }
}
impl ::std::convert::From<&PromptReference> for CompleteRequestParamsRef {
    fn from(value: &PromptReference) -> Self {
        <CompleteRequestParamsRef as ::std::convert::From<PromptReference>>::from(
            <PromptReference as ::std::convert::From<&PromptReference>>::from(value),
        )
    }
}
impl ::std::convert::From<&InitializedNotification> for ClientNotification {
    fn from(value: &InitializedNotification) -> Self {
        <ClientNotification as ::std::convert::From<InitializedNotification>>::from(
            <InitializedNotification as ::std::convert::From<&InitializedNotification>>::from(
                value,
            ),
        )
    }
}
impl ::std::convert::From<&JsonrpcResponse> for JsonrpcMessage {
    fn from(value: &JsonrpcResponse) -> Self {
        Self {
            subtype_0: None,
            subtype_1: None,
            subtype_2: None,
            subtype_3: Some(value.clone()),
            subtype_4: None,
            subtype_5: None,
        }
    }
}

impl ::std::convert::From<JsonrpcResponse> for JsonrpcMessage {
    fn from(value: JsonrpcResponse) -> Self {
        Self {
            subtype_0: None,
            subtype_1: None,
            subtype_2: None,
            subtype_3: Some(value),
            subtype_4: None,
            subtype_5: None,
        }
    }
}
impl ::std::convert::From<&SubscribeRequest> for ClientRequest {
    fn from(value: &SubscribeRequest) -> Self {
        <ClientRequest as ::std::convert::From<SubscribeRequest>>::from(
            <SubscribeRequest as ::std::convert::From<&SubscribeRequest>>::from(value),
        )
    }
}
