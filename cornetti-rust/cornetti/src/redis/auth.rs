use crate::{
    auth::{confs::JWTStoreConf, models::SessionStoreData, traits::SessionStore},
    core::models::CornettiResult,
    redis::services::RedisDBService,
};
use chrono::Utc;
use redis::{AsyncTypedCommands, HashFieldExpirationOptions};
use std::{collections::HashSet, sync::Arc};

/// Redis-backed session store implementing [`SessionStore`].
///
/// Uses field-level TTL via `HSETEX` (requires Redis >= 7.0).
/// Each token is stored as a dedicated key with its own expiry.
/// Sessions are tracked via hash sets and sets for user→session lookups.
pub struct RedisSessionStore {
    store_conf: JWTStoreConf,
    pub redis_conn: Arc<RedisDBService>,
    app_id: String,
}

impl RedisSessionStore {
    /// Creates a new Redis session store.
    pub fn new(store_conf: JWTStoreConf, redis_conn: Arc<RedisDBService>, app_id: &str) -> Self {
        RedisSessionStore {
            store_conf,
            redis_conn,
            app_id: app_id.to_string(),
        }
    }

    fn app_segment(&self) -> String {
        format!("{}:", self.app_id)
    }

    fn key_from_claim(&self, tenant_id: &str, claim: &SessionStoreData) -> String {
        if claim.refresh {
            self.refresh_key(tenant_id, &claim.jti)
        } else {
            self.auth_key(tenant_id, &claim.jti)
        }
    }

    fn auth_key(&self, tenant_id: &str, jti: &str) -> String {
        format!("{}:{}{}:auth:{}", self.store_conf.store_name, tenant_id, self.app_segment(), jti)
    }

    fn refresh_key(&self, tenant_id: &str, jti: &str) -> String {
        format!("{}:{}{}:refresh:{}", self.store_conf.store_name, tenant_id, self.app_segment(), jti)
    }

    fn session_key(&self, tenant_id: &str, session_id: &str) -> String {
        format!("{}:{}{}:sessions:{}", self.store_conf.store_name, tenant_id, self.app_segment(), session_id)
    }

    fn users_sessions_key(&self, tenant_id: &str, subject: &str) -> String {
        format!("{}:{}{}:users:{}:sessions", self.store_conf.store_name, tenant_id, self.app_segment(), subject)
    }
}

impl SessionStore for RedisSessionStore {
    async fn add_token(&self, tenant_id: &str, claim: &SessionStoreData) -> CornettiResult<()> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let token_serialized: String = serde_json::to_string(claim)?;
        let token_key = self.key_from_claim(tenant_id, claim);
        connection.set(&token_key, token_serialized).await?;
        connection.expire_at(&token_key, claim.exp as i64).await?;

        let field_name: &'static str = if claim.refresh { "refresh" } else { "auth" };
        let session_expire_at: usize =
            chrono::Utc::now().timestamp() as usize + self.store_conf.session_expire_mins * 60;
        let session_key = self.session_key(tenant_id, &claim.session_id);

        let hset_options: HashFieldExpirationOptions = HashFieldExpirationOptions::default()
            .set_expiration(redis::SetExpiry::EXAT(claim.exp as u64));
        connection
            .hset_ex(&session_key, &hset_options, &[(field_name, &claim.jti)])
            .await?;
        connection
            .expire_at(&session_key, session_expire_at as i64)
            .await?;

        if !claim.refresh {
            let sessions_key: String = self.users_sessions_key(tenant_id, &claim.sub);
            connection.sadd(&sessions_key, &claim.session_id).await?;
            connection
                .expire_at(&sessions_key, session_expire_at as i64)
                .await?;
        }

        Ok(())
    }

    async fn remove_auth_token(&self, tenant_id: &str, jti: &str) -> CornettiResult<usize> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;
        Ok(connection.del(self.auth_key(tenant_id, jti)).await?)
    }

    async fn remove_refresh_token(&self, tenant_id: &str, jti: &str) -> CornettiResult<usize> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;
        Ok(connection.del(self.refresh_key(tenant_id, jti)).await?)
    }

    async fn get_auth_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> CornettiResult<Option<SessionStoreData>> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;
        let session_data: Option<String> = connection.get(self.auth_key(tenant_id, jti)).await?;

        match session_data {
            Some(data) => {
                let store_data: SessionStoreData = serde_json::from_str(&data)?;
                Ok(Some(store_data))
            }
            None => Ok(None),
        }
    }

    async fn get_refresh_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> CornettiResult<Option<SessionStoreData>> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let session_data: Option<String> = connection.get(self.refresh_key(tenant_id, jti)).await?;

        match session_data {
            Some(data) => {
                let store_data: SessionStoreData = serde_json::from_str(&data)?;
                Ok(Some(store_data))
            }
            None => Ok(None),
        }
    }

    async fn remove_session(
        &self,
        tenant_id: &str,
        sub: &str,
        session_id: &str,
    ) -> CornettiResult<usize> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let session_key = self.session_key(tenant_id, session_id);

        let auth_tkn: Option<String> = connection.hget(&session_key, "auth").await?;
        let refresh_tkn: Option<String> = connection.hget(&session_key, "refresh").await?;

        let mut removed: usize = 0;

        if let Some(tkn) = auth_tkn {
            removed += connection.del(self.auth_key(tenant_id, &tkn)).await?;
        }

        if let Some(tkn) = refresh_tkn {
            removed += connection.del(self.refresh_key(tenant_id, &tkn)).await?;
        }

        removed += connection.del(self.session_key(tenant_id, session_id)).await?;
        removed += connection
            .srem(self.users_sessions_key(tenant_id, sub), session_id)
            .await?;

        Ok(removed)
    }

    async fn subject_sessions(&self, tenant_id: &str, sub: &str) -> CornettiResult<Vec<SessionStoreData>> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let sessions_key = self.users_sessions_key(tenant_id, sub);
        let user_sessions: HashSet<String> = connection.smembers(&sessions_key).await?;

        let mut sessions = Vec::new();

        for s in user_sessions {
            if let Some(auth_tkn) = connection.hget(self.session_key(tenant_id, &s), "auth").await?
                && let Some(session) = self.get_auth_token(tenant_id, &auth_tkn).await?
                    && session.exp > Utc::now().timestamp() as usize {
                        sessions.push(session);
                    }
        }

        Ok(sessions)
    }

    async fn clear_subject_sessions(&self, tenant_id: &str, sub: &str) -> CornettiResult<usize> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let sessions_key = self.users_sessions_key(tenant_id, sub);
        let user_sessions: HashSet<String> = connection.smembers(&sessions_key).await?;

        let mut removed_session: usize = 0;

        for s in user_sessions {
            let res = self.remove_session(tenant_id, sub, &s).await?;
            if res > 0 {
                removed_session += 1;
            }
        }

        Ok(removed_session)
    }
}
