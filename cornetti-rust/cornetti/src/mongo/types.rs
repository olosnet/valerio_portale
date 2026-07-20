use mongodb::bson;

/// Type alias for backward compatibility — prefer `bson::oid::ObjectId` directly.
pub type CornettiObjectId = bson::oid::ObjectId;

/// Parses an `ObjectId` from a hex string.
///
/// Returns a `bson::error::Error` if the string is not a valid 24-byte hex string.
pub fn parse_object_id(s: &str) -> Result<bson::oid::ObjectId, bson::error::Error> {
    bson::oid::ObjectId::parse_str(s)
}
