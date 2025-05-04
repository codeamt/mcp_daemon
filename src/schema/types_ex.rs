use std::borrow::Cow;

use base64::Engine;
use parse_display::Display;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

/// Type for handling byte sequences as Base64-encoded strings
///
/// This type is used when you want to handle a byte sequence as a Base64-encoded string in JSON serialization,
/// and then convert it back to a byte sequence when deserializing.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::schema::Base64Bytes;
/// use serde_json::json;
///
/// let bytes = Base64Bytes(vec![1, 2, 3, 4, 5]);
/// let json = json!(bytes);
/// assert_eq!(json, json!("AQIDBAU="));
///
/// let bytes: Base64Bytes = serde_json::from_value(json).unwrap();
/// assert_eq!(bytes.0, vec![1, 2, 3, 4, 5]);
/// ```
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Base64Bytes(pub Vec<u8>);

impl Serialize for Base64Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = base64::prelude::BASE64_STANDARD.encode(&self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Base64Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        base64::prelude::BASE64_STANDARD
            .decode(&*s)
            .map_err(serde::de::Error::custom)
            .map(Base64Bytes)
    }
}

/// Type representing an empty JSON object
///
/// This type is used when you want to output an empty JSON object `{}` in JSON serialization,
/// and accept any JSON object when deserializing, but its content is ignored.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::schema::Empty;
/// use serde_json::json;
///
/// let empty = Empty::default();
/// let json = json!(empty);
/// assert_eq!(json, json!({}));
///
/// let empty: Empty = serde_json::from_value(json!({ "key": "value" })).unwrap();
/// let json = json!(empty);
/// assert_eq!(json, json!({}));
/// ```
#[derive(Serialize, Default)]
#[serde(transparent)]
pub struct Empty(#[allow(unused)] Map<String, Value>);

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _: Map<String, Value> = Deserialize::deserialize(deserializer)?;
        Ok(Empty::default())
    }
}

/// Type representing a tag string associated with a type
///
/// This type is used when you want to output a tag string associated with a type in JSON serialization,
/// and check if the tag string matches when deserializing.
///
/// The tag string is specified by the `TAG` constant of the `TagData` trait.
///
/// # Example
///
/// ```rust,ignore
/// use mcp_daemon::schema::{Tag, TagData};
/// use serde_json::json;
///
/// #[derive(Default)]
/// struct MyTag;
///
/// impl TagData for MyTag {
///     const TAG: &'static str = "my-tag";
/// }
///
/// let tag = Tag(MyTag::default());
/// let json = serde_json::to_value(&tag).unwrap();
/// assert_eq!(json, json!("my-tag"));
///
/// let tag: Tag<MyTag> = serde_json::from_value(json).unwrap();
/// ```
pub struct Tag<T>(pub T);

/// Trait for specifying a tag string associated with a type
///
/// This trait is used to specify the tag string for use with the `Tag` type.
pub trait TagData: Default {
    /// Tag string
    const TAG: &'static str;
}

impl<T: TagData> Serialize for Tag<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(T::TAG)
    }
}

impl<'de, T: TagData> Deserialize<'de> for Tag<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        if s != T::TAG {
            return Err(serde::de::Error::custom(format!("expected tag {}", T::TAG)));
        }
        Ok(Tag(T::default()))
    }
}

/// Represents a version of the MCP protocol.
///
/// This type encapsulates a protocol version string and provides constants for
/// known protocol versions. It's used during the initialization phase to ensure
/// compatibility between clients and servers.
///
/// # Examples
///
/// ```rust,ignore
/// use mcp_daemon::schema::ProtocolVersion;
///
/// // Get the latest protocol version
/// let version = ProtocolVersion::LATEST;
///
/// // Use a specific protocol version
/// let version = ProtocolVersion::V_2025_03_26;
///
/// // Get the version as a string
/// let version_str = version.as_str(); // "2025-03-26"
/// ```
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Clone, Copy,
)]
pub struct ProtocolVersion(&'static str);

impl ProtocolVersion {
    /// The latest supported protocol version.
    ///
    /// This constant can be used to always use the most recent protocol version
    /// supported by this library.
    pub const LATEST: Self = Self::V_2025_03_26;
    
    /// The March 26, 2025 version of the MCP protocol.
    ///
    /// This version corresponds to the protocol as specified in the 2025-03-26 version
    /// of the MCP specification.
    pub const V_2025_03_26: Self = Self("2025-03-26");

    /// Returns the protocol version as a string.
    ///
    /// # Returns
    ///
    /// A string representation of the protocol version, like "2025-03-26".
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use mcp_daemon::schema::ProtocolVersion;
    ///
    /// let version = ProtocolVersion::LATEST;
    /// assert_eq!(version.as_str(), "2025-03-26");
    /// ```
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_base64_bytes() {
        let bytes = Base64Bytes(vec![1, 2, 3, 4, 5]);
        let json = json!(bytes);
        assert_eq!(json, json!("AQIDBAU="));

        let bytes: Base64Bytes = serde_json::from_value(json).unwrap();
        assert_eq!(bytes.0, vec![1, 2, 3, 4, 5]);
    }
}
