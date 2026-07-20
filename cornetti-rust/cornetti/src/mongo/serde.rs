/// Serde helpers for serializing optional ObjectIds as hex strings in human-readable formats.
pub mod optional_objectid_as_human_readable {
    use serde::{Deserialize, Deserializer, Serializer};

    use bson::oid::ObjectId;

    /// Serializes `Option<ObjectId>` as a hex string, or `None` as JSON null.
    pub fn serialize<S>(value: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(id) => serializer.serialize_str(&id.to_string()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional ObjectId from a hex string. Returns an error on invalid input.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        match Option::<String>::deserialize(deserializer)? {
            Some(s) => ObjectId::parse_str(&s)
                .map(Some)
                .map_err(|e| D::Error::custom(format!("Invalid ObjectId: {}", e))),
            None => Ok(None),
        }
    }
}

/// Serde helpers for serializing `Vec<ObjectId>` as hex strings in human-readable formats.
pub mod vec_objectid_as_human_readable {
    use bson::oid::ObjectId;
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    /// Serializes a vector of ObjectIds as an array of hex strings.
    pub fn serialize<S>(value: &Vec<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for id in value {
            seq.serialize_element(&id.to_string())?;
        }
        seq.end()
    }

    /// Deserializes an array of hex strings into `Vec<ObjectId>`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<ObjectId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|id| ObjectId::parse_str(&id).map_err(D::Error::custom))
            .collect()
    }
}
