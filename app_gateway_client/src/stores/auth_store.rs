use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::modules::auth::api as auth_api;
use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::AuthorizationPermission;
use crate::modules::identity::api as identity_api;
use crate::modules::identity::models::UserIdentity;

#[derive(Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<UserIdentity>>,
    pub api_client: ApiClient,
}

impl AuthContext {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            user: RwSignal::new(None),
            api_client,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub fn is_auth_signal(&self) -> Signal<bool> {
        let user = self.user;
        Signal::derive(move || user.get().is_some())
    }

    pub fn perms(&self) -> HashMap<String, AuthorizationPermission> {
        self.user
            .get()
            .as_ref()
            .map(|u| u.permissions.clone())
            .unwrap_or_default()
    }

    pub fn perms_signal(&self) -> Signal<HashMap<String, AuthorizationPermission>> {
        let user = self.user;
        Signal::derive(move || {
            user.get()
                .as_ref()
                .map(|u| u.permissions.clone())
                .unwrap_or_default()
        })
    }

    pub fn can_read<'a>(&'a self, module: &str) -> bool {
        self.user
            .get()
            .as_ref()
            .and_then(|u| u.permissions.get(module))
            .map(|p| p.read)
            .unwrap_or(false)
    }

    pub fn can_read_signal<'a>(&'a self, module: &str) -> Signal<bool> {
        let user = self.user;
        let module = module.to_string();
        Signal::derive(move || {
            user.get()
                .as_ref()
                .and_then(|u| u.permissions.get(&module))
                .map(|p| p.read)
                .unwrap_or(false)
        })
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(), String> {
        auth_api::login(&self.api_client, username, password)
            .await
            .map_err(|e| e.to_string())?;

        match identity_api::get_identity(&self.api_client).await {
            Ok(identity) => {
                self.user.set(Some(identity));
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn logout(&self) {
        let _ = auth_api::logout(&self.api_client).await;
        self.user.set(None);
    }

    pub async fn check_session(&self) {
        if let Ok(identity) = identity_api::get_identity(&self.api_client).await {
            self.user.set(Some(identity));
        }
    }
}

pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

pub fn provide_auth(api_client: ApiClient) -> AuthContext {
    let ctx = AuthContext::new(api_client);
    provide_context(ctx.clone());
    ctx
}
