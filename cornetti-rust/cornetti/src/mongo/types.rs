use mongodb::bson;
use serde::{self, Deserializer, Serializer};

/// Wrapper around MongoDB's `ObjectId` providing human-readable serialization.
///
/// In human-readable formats (JSON), serializes as the hex string.
/// In binary formats (BSON), serializes as the native `ObjectId`.
#[derive(Clone, Debug)]
pub struct CornettiObjectId(bson::oid::ObjectId);

impl Default for CornettiObjectId {
    fn default() -> Self {
        CornettiObjectId(bson::oid::ObjectId::new())
    }
}

impl CornettiObjectId {
    /// Parses an `ObjectId` from a hex string.
    ///
    /// # Errors
    ///
    /// Returns a `bson::error::Error` if the string is not a valid 24-byte hex string.
    pub fn parse_str(s: &str) -> Result<Self, bson::error::Error> {
        bson::oid::ObjectId::parse_str(s).map(CornettiObjectId)
    }

    /// Returns a reference to the inner BSON `ObjectId`.
    pub fn to_bson_oid(&self) -> &bson::oid::ObjectId {
        &self.0
    }
}

impl From<CornettiObjectId> for bson::oid::ObjectId {
    fn from(oid: CornettiObjectId) -> Self {
        oid.0
    }
}

impl From<bson::oid::ObjectId> for CornettiObjectId {
    fn from(oid: bson::oid::ObjectId) -> Self {
        CornettiObjectId(oid)
    }
}

impl From<CornettiObjectId> for std::string::String {
    fn from(oid: CornettiObjectId) -> Self {
        oid.0.to_string()
    }
}

impl From<&CornettiObjectId> for std::string::String {
    fn from(oid: &CornettiObjectId) -> Self {
        oid.0.to_string()
    }
}

impl From<&std::string::String> for CornettiObjectId {
    /// Converts a `&String` to `CornettiObjectId`, defaulting to a new `ObjectId` on parse failure.
    fn from(oid: &String) -> Self {
        CornettiObjectId(bson::oid::ObjectId::parse_str(oid).unwrap_or_default())
    }
}

impl From<&str> for CornettiObjectId {
    /// Converts a `&str` to `CornettiObjectId`, defaulting to a new `ObjectId` on parse failure.
    fn from(oid: &str) -> Self {
        CornettiObjectId(bson::oid::ObjectId::parse_str(oid).unwrap_or_default())
    }
}

impl std::fmt::Display for CornettiObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for CornettiObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for CornettiObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let oid = String::deserialize(deserializer)?;
            bson::oid::ObjectId::parse_str(&oid)
                .map_err(serde::de::Error::custom).map(CornettiObjectId)
        } else {
            let oid = bson::oid::ObjectId::deserialize(deserializer)?;
            Ok(CornettiObjectId(oid))
        }
    }
}
