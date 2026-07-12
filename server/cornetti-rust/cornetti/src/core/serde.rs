/// Serde helpers for conditionally skipping fields in human-readable formats.
pub mod skip_if_human_readable {
    use serde::{Deserializer, Serializer};

    /// Serializes the value normally for binary formats, skips it for human-readable
    /// formats (e.g., JSON).
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: serde::Serialize,
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_none()
        } else {
            value.serialize(serializer)
        }
    }

    /// Deserializes the value for binary formats, fails for human-readable formats.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: serde::Deserialize<'de>,
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Err(serde::de::Error::custom(
                "Field skipped in human-readable format",
            ))
        } else {
            T::deserialize(deserializer)
        }
    }
}
