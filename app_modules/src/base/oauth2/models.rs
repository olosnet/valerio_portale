//! Modello Mongo per la metadata OAuth2 associata a un utente locale.

use bson::oid::ObjectId;
use cornetti::auth_oauth2::models::OAuth2Metadata;
use cornetti::core::traits::BaseModule;
use cornetti::mongo::traits::MongoBaseModel;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Metadata OAuth2 persistita in Mongo (collezione `oauth2`).
///
/// Specchia `cornetti::auth_oauth2::models::OAuth2Metadata` aggiungendo
/// l'`_id` Mongo e la conversione chrono/bson per le date (`created`/
/// `modified` seguono la convenzione di `MongoUserModel`).
///
/// # Nota di sicurezza
///
/// Contiene `access_token`/`refresh_token` in chiaro: la scelta di
/// cifrarli è demandata al consumatore del framework.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoOAuth2MetadataModel {
    #[serde(skip_serializing)]
    pub _id: Option<ObjectId>,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub tenant_id: String,
    #[serde_as(as = "Option<bson::serde_helpers::datetime::FromChrono04DateTime>")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde_as(as = "bson::serde_helpers::datetime::FromChrono04DateTime")]
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl MongoBaseModel for MongoOAuth2MetadataModel {
    fn _id(&self) -> &Option<ObjectId> {
        &self._id
    }

    fn created(&self) -> &Option<chrono::DateTime<chrono::Utc>> {
        &self.created
    }

    fn modified(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.modified
    }

    fn touch(&mut self) {
        self.modified = chrono::Utc::now();
    }

    fn collection_name() -> &'static str {
        super::OAuth2Module::module_name()
    }
}

impl From<MongoOAuth2MetadataModel> for OAuth2Metadata {
    fn from(model: MongoOAuth2MetadataModel) -> Self {
        OAuth2Metadata {
            provider: model.provider,
            provider_user_id: model.provider_user_id,
            access_token: model.access_token,
            refresh_token: model.refresh_token,
            expires_at: model.expires_at,
            scopes: model.scopes,
            user_id: model.user_id,
            tenant_id: model.tenant_id,
            created_at: model.created.unwrap_or(model.modified),
            updated_at: model.modified,
        }
    }
}

impl From<OAuth2Metadata> for MongoOAuth2MetadataModel {
    fn from(metadata: OAuth2Metadata) -> Self {
        MongoOAuth2MetadataModel {
            _id: None,
            provider: metadata.provider,
            provider_user_id: metadata.provider_user_id,
            access_token: metadata.access_token,
            refresh_token: metadata.refresh_token,
            expires_at: metadata.expires_at,
            scopes: metadata.scopes,
            user_id: metadata.user_id,
            tenant_id: metadata.tenant_id,
            created: Some(metadata.created_at),
            modified: metadata.updated_at,
        }
    }
}
