//! Handler OAuth2 del dominio: implementa `OAuth2UserHandler<User>` di
//! cornetti collegando provider OAuth2 e utenti locali.

use super::repos::OAuth2MetadataRepository;
use crate::base::users::{
    models::{User, UserCreate},
    repos::UsersRepository,
};
use cornetti::auth_oauth2::models::{OAuth2Metadata, OAuth2UserTransportData};
use cornetti::auth_oauth2::traits::{OAuth2Identity, OAuth2UserHandler};
use cornetti::core::models::CornettiResult;
use cornetti::errors;
use cornetti::mongo::services::MongoDBService;
use std::sync::Arc;

/// Handler OAuth2: lookup/creazione utenti locali + persistenza metadata.
pub struct OAuth2UserHandlerImpl {
    users_repo: UsersRepository,
    oauth2_repo: OAuth2MetadataRepository,
}

impl OAuth2UserHandlerImpl {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        OAuth2UserHandlerImpl {
            users_repo: UsersRepository::new(mongo.clone()),
            oauth2_repo: OAuth2MetadataRepository::new(mongo),
        }
    }
}

impl OAuth2UserHandler<User> for OAuth2UserHandlerImpl {
    /// Trova un utente locale già collegato a un account OAuth2.
    async fn find_by_oauth2(
        &self,
        tenant_id: &str,
        provider: &str,
        provider_user_id: &str,
    ) -> CornettiResult<Option<(User, OAuth2Metadata)>> {
        match self
            .oauth2_repo
            .find_by_provider_user(tenant_id, provider, provider_user_id)
            .await?
        {
            Some(metadata) => {
                let user = self.users_repo.get(&metadata.user_id).await?;
                Ok(Some((user, metadata)))
            }
            None => Ok(None),
        }
    }

    /// Crea un utente locale a partire dai dati del provider e persiste la
    /// metadata. Senza email dal provider l'utente locale non può essere
    /// creato (l'email è il riferimento del dominio).
    ///
    /// Nota: la metadata viene creata senza token — il framework aggiorna la
    /// metadata con access/refresh token al primo callback successivo
    /// (percorso `find_by_oauth2` + `update_oauth2_metadata`).
    async fn create_from_oauth2(
        &self,
        tenant_id: &str,
        user_data: &OAuth2UserTransportData,
    ) -> CornettiResult<(User, OAuth2Metadata)> {
        let email = user_data.email.clone().ok_or_else(|| {
            errors::bad_request::invalid_email().with_internal_detail(
                "OAuth2 provider did not return an email: cannot create a local user",
            )
        })?;

        let email_prefix = email.split('@').next().unwrap_or("oauth2").to_string();
        let name = user_data.name.clone().unwrap_or(email_prefix);

        let user = self
            .users_repo
            .create(UserCreate {
                name,
                surname: String::new(),
                email,
                enabled: true,
                groups_ids: Vec::new(),
            })
            .await?;

        let user_id = user
            .id
            .clone()
            .ok_or_else(errors::not_found::item_not_found)?;

        let now = chrono::Utc::now();
        let metadata = OAuth2Metadata {
            provider: user_data.provider.clone(),
            provider_user_id: user_data.provider_user_id.clone(),
            access_token: String::new(),
            refresh_token: None,
            expires_at: None,
            scopes: Vec::new(),
            user_id,
            tenant_id: tenant_id.to_string(),
            created_at: now,
            updated_at: now,
        };

        self.oauth2_repo.upsert(&metadata).await?;

        Ok((user, metadata))
    }

    /// Aggiorna la metadata OAuth2 (es. dopo un refresh dei token).
    async fn update_oauth2_metadata(
        &self,
        _tenant_id: &str,
        metadata: &OAuth2Metadata,
    ) -> CornettiResult<()> {
        self.oauth2_repo.upsert(metadata).await
    }
}

impl OAuth2Identity for User {
    /// Soggetto JWT: l'email (riferimento del dominio, usata da identity,
    /// refresh e logout), con fallback sull'id.
    fn subject(&self) -> String {
        self.email
            .clone()
            .or_else(|| self.id.clone())
            .unwrap_or_default()
    }
}
