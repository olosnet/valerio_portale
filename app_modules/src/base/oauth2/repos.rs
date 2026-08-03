//! Repository della metadata OAuth2 su MongoDB.

use super::models::MongoOAuth2MetadataModel;
use bson::doc;
use cornetti::auth_oauth2::models::OAuth2Metadata;
use cornetti::core::models::CornettiResult;
use cornetti::mongo::services::MongoDBService;
use cornetti::mongo::traits::MongoBaseModel;
use mongodb::Collection;
use std::sync::Arc;

pub struct OAuth2MetadataRepository {
    pub mongo: Arc<MongoDBService>,
}

impl OAuth2MetadataRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        OAuth2MetadataRepository { mongo }
    }

    fn collection(&self) -> Collection<MongoOAuth2MetadataModel> {
        self.mongo
            .db()
            .collection(MongoOAuth2MetadataModel::collection_name())
    }

    /// Trova la metadata per (tenant, provider, provider_user_id).
    pub async fn find_by_provider_user(
        &self,
        tenant_id: &str,
        provider: &str,
        provider_user_id: &str,
    ) -> CornettiResult<Option<OAuth2Metadata>> {
        let model = self
            .collection()
            .find_one(doc! {
                "tenant_id": tenant_id,
                "provider": provider,
                "provider_user_id": provider_user_id,
            })
            .await?;

        Ok(model.map(|m| m.into()))
    }

    /// Trova la metadata per (tenant, user_id locale).
    pub async fn find_by_user_id(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> CornettiResult<Option<OAuth2Metadata>> {
        let model = self
            .collection()
            .find_one(doc! {
                "tenant_id": tenant_id,
                "user_id": user_id,
            })
            .await?;

        Ok(model.map(|m| m.into()))
    }

    /// Inserisce o aggiorna la metadata (upsert su tenant+provider+provider_user_id).
    pub async fn upsert(&self, metadata: &OAuth2Metadata) -> CornettiResult<()> {
        let model: MongoOAuth2MetadataModel = metadata.clone().into();
        let document: bson::Document = model.to_bson().as_document().unwrap().clone();

        let filter = doc! {
            "tenant_id": &metadata.tenant_id,
            "provider": &metadata.provider,
            "provider_user_id": &metadata.provider_user_id,
        };

        self.collection()
            .update_one(filter, doc! { "$set": document })
            .upsert(true)
            .await?;

        Ok(())
    }
}
