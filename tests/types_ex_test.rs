use mcp_daemon::schema::{Base64Bytes, Tag, TagData};
use serde_json::json;

#[test]
fn test_base64_bytes_serialization() {
    // Test serialization
    let bytes = Base64Bytes(vec![1, 2, 3, 4, 5]);
    let json_value = json!(bytes);
    assert_eq!(json_value, json!("AQIDBAU="));
}

#[test]
fn test_base64_bytes_deserialization() {
    // Test deserialization
    let json_value = json!("AQIDBAU=");
    let bytes: Base64Bytes = serde_json::from_value(json_value).unwrap();
    assert_eq!(bytes.0, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_base64_bytes_default() {
    // Test default implementation
    let bytes = Base64Bytes::default();
    assert_eq!(bytes.0, Vec::<u8>::new());
}

#[test]
fn test_base64_bytes_invalid_input() {
    // Test invalid base64 input
    let json_value = json!("invalid-base64");
    let result: Result<Base64Bytes, _> = serde_json::from_value(json_value);
    assert!(result.is_err());
}

#[test]
fn test_empty_serialization() {
    // Test Empty serialization
    let empty = mcp_daemon::schema::Empty::default();
    let json_value = json!(empty);
    assert_eq!(json_value, json!({}));
}

#[test]
fn test_empty_deserialization() {
    // Test Empty deserialization with empty object
    let json_value = json!({});
    let _: mcp_daemon::schema::Empty = serde_json::from_value(json_value).unwrap();

    // Test Empty deserialization with non-empty object
    let json_value = json!({"key": "value"});
    let _: mcp_daemon::schema::Empty = serde_json::from_value(json_value).unwrap();
}

#[test]
fn test_tag_serialization() {
    // Define a test tag
    #[derive(Default)]
    struct TestTag;

    impl TagData for TestTag {
        const TAG: &'static str = "test-tag";
    }

    // Test Tag serialization
    let tag = Tag(TestTag::default());
    let json_value = serde_json::to_value(&tag).unwrap();
    assert_eq!(json_value, json!("test-tag"));
}

#[test]
fn test_tag_deserialization() {
    // Define a test tag
    #[derive(Default)]
    struct TestTag;

    impl TagData for TestTag {
        const TAG: &'static str = "test-tag";
    }

    // Test Tag deserialization with correct tag
    let json_value = json!("test-tag");
    let _: Tag<TestTag> = serde_json::from_value(json_value).unwrap();

    // Test Tag deserialization with incorrect tag
    let json_value = json!("wrong-tag");
    let result: Result<Tag<TestTag>, _> = serde_json::from_value(json_value);
    assert!(result.is_err());
}

#[test]
fn test_protocol_version() {
    // Test protocol version constants
    assert_eq!(mcp_daemon::schema::ProtocolVersion::LATEST.to_string(), "2025-03-26");
    assert_eq!(mcp_daemon::schema::ProtocolVersion::V_2025_03_26.to_string(), "2025-03-26");

    // Test as_str method
    assert_eq!(mcp_daemon::schema::ProtocolVersion::LATEST.as_str(), "2025-03-26");
}
