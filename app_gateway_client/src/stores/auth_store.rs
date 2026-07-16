use std::collections::HashMap;

use leptos::prelude::*;

use crate::modules::auth::api as auth_api;
use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::AuthorizationPermission;
use crate::modules::identity::api as identity_api;
use crate::modules::identity::models::UserIdentity;

#[derive(Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<UserIdentity>>,
    pub api_client: ApiClient,
    pub initial_check_done: RwSignal<bool>,
}

impl AuthContext {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            user: RwSignal::new(None),
            api_client,
            initial_check_done: RwSignal::new(false),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub fn is_auth_signal(&self) -> Signal<bool> {
        let user = self.user;
        Signal::derive(move || user.get().is_some())
    }

    fn get_perm(user: &UserIdentity, module: &str) -> Option<AuthorizationPermission> {
        user.permissions
            .get(module)
            .or_else(|| user.permissions.get("all"))
            .cloned()
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

    pub fn can_read(&self, module: &str) -> bool {
        self.user.get().as_ref().map_or(false, |u| {
            Self::get_perm(u, module).map_or(false, |p| p.read)
        })
    }

    pub fn can_read_signal(&self, module: &str) -> Signal<bool> {
        let user = self.user;
        let module = module.to_string();
        Signal::derive(move || {
            user.get().as_ref().map_or(false, |u| {
                Self::get_perm(u, &module).map_or(false, |p| p.read)
            })
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
        self.initial_check_done.set(true);
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
