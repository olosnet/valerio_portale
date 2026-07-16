use cornetti::{
    core::{errors, models::CornettiResult},
    mongo::{helpers::modules_collection_name, services::MongoDBService},
};
use futures::TryStreamExt;

pub struct PermissionsRepository<'a> {
    pub mongo: &'a MongoDBService,
}

impl<'a> PermissionsRepository<'a> {
    pub fn new(mongo: &'a MongoDBService) -> Self {
        PermissionsRepository { mongo }
    }

    pub async fn list(&self) -> CornettiResult<Vec<String>> {
        let collection = self
            .mongo
            .db()
            .collection::<mongodb::bson::Document>(modules_collection_name());

        let permissions_pipeline = vec![
            bson::doc! {
                "$group": {"_id": mongodb::bson::Bson::Null, "permissions": {"$push": "$permissions"}},
            },
            bson::doc! {
                    "$project": {
                        "_id": 0,
                        "permissions": {
                            "$reduce": {
                                "input": "$permissions",
                                "initialValue": [],
                                "in": {"$concatArrays": ["$$value", "$$this"]},
                            }
                        },
                    }

            },
            bson::doc! {
                "$addFields": {"permissions": {"$setUnion": ["$permissions", []]}}
            },
        ];

        match collection.aggregate(permissions_pipeline).await {
            Ok(mut cursor) => {
                let permissions: Vec<String> = match cursor.try_next().await {
                    Ok(Some(doc)) => {
                        let array = doc.get_array("permissions").map_err(|e| {
                            errors::internal_server_error::generic_error(e.to_string())
                        })?;
                        Ok(array
                            .iter()
                            .filter_map(|value| value.as_str().map(String::from))
                            .collect())
                    }
                    Ok(None) => Ok(vec![]),
                    Err(e) => Err(errors::internal_server_error::generic_error(e.to_string())),
                }?;

                Ok(permissions)
            }
            Err(e) => Err(errors::internal_server_error::db_error(e.to_string())),
        }
    }
}
