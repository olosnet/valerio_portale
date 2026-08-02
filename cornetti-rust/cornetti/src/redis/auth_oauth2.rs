use crate::auth_oauth2::traits::OAuth2SessionStore;
use crate::core::models::CornettiResult;
use crate::redis::services::RedisDBService;
use redis::AsyncCommands;
use std::sync::Arc;

/// Redis-backed OAuth2 session store implementing [`OAuth2SessionStore`].
///
/// Maps CSRF state → flow payload (PKCE verifier) as plain keys with a
/// per-entry TTL.
///
/// `take_oauth2_state` uses `GETDEL` (atomic read-and-delete), the one-shot
/// semantics required by the trait: a consumed or expired state cannot be
/// replayed.
///
/// # Key format
///
/// `{tenant_id}:{app_id}:oauth2:{state_key}` where `state_key` already
/// includes the provider (`{provider}:{state}`).
pub struct RedisOAuth2SessionStore {
    pub redis_conn: Arc<RedisDBService>,
    app_id: String,
}

impl RedisOAuth2SessionStore {
    /// Creates a new Redis OAuth2 session store.
    pub fn new(redis_conn: Arc<RedisDBService>, app_id: &str) -> Self {
        RedisOAuth2SessionStore {
            redis_conn,
            app_id: app_id.to_string(),
        }
    }

    fn state_key(&self, tenant_id: &str, state_key: &str) -> String {
        format!("{}:{}:oauth2:{}", tenant_id, self.app_id, state_key)
    }
}

impl OAuth2SessionStore for RedisOAuth2SessionStore {
    async fn set_oauth2_state(
        &self,
        tenant_id: &str,
        state_key: &str,
        pkce_verifier: String,
        ttl_secs: u64,
    ) -> CornettiResult<()> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let _: () = connection
            .set_ex(
                self.state_key(tenant_id, state_key),
                pkce_verifier,
                ttl_secs,
            )
            .await?;

        Ok(())
    }

    async fn take_oauth2_state(
        &self,
        tenant_id: &str,
        state_key: &str,
    ) -> CornettiResult<Option<String>> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let value: Option<String> = connection
            .get_del(self.state_key(tenant_id, state_key))
            .await?;

        Ok(value)
    }
}
