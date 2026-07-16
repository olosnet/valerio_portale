use crate::{
    core::{
        helpers::{
            common::base_generate_random_string,
            sec::{self, verify_password},
        },
        models::CornettiResult,
    },
    otp::confs::SimpleOtpConf,
    redis::services::RedisDBService,
};
use redis::AsyncTypedCommands;
use std::sync::Arc;

/// Generates and verifies one-time passwords against a Redis-backed store.
///
/// The OTP is hashed with Argon2 before storage. The plaintext OTP is returned
/// to the caller after generation and never persisted.
///
/// Currently, the constructor and the `generate_otp`/`verify_otp` methods are
/// crate-private; use [`SimpleOtpStore`] directly for external integration.
pub struct SimpleOtpGenerator {
    /// OTP generation configuration.
    pub conf: SimpleOtpConf,
    /// Logical domain this generator is scoped to (used in the Redis key).
    pub ref_domain: String,
    store: SimpleOtpStore,
}

impl SimpleOtpGenerator {
    #[allow(dead_code)]
    fn new(
        conf: SimpleOtpConf,
        tenant_id: String,
        app_id: String,
        redis_conn: Arc<RedisDBService>,
        ref_domain: String,
    ) -> Self {
        Self {
            conf,
            store: SimpleOtpStore {
                redis_conn,
                tenant_id,
                app_id,
            },
            ref_domain,
        }
    }

    #[allow(dead_code)]
    async fn generate_otp(&self) -> CornettiResult<String> {
        let otp = base_generate_random_string(
            self.conf.otp_length,
            self.conf.otp_length,
            self.conf.otp_chars.clone(),
        );

        let otp_hash = sec::hash_password(&otp);
        let expires = chrono::Utc::now() + chrono::Duration::seconds(self.conf.otp_expires_minutes);

        self.store
            .set_otp(self.ref_domain.as_str(), otp_hash.as_str(), expires)
            .await?;

        Ok(otp)
    }

    #[allow(dead_code)]
    async fn verify_otp(&self, otp: &str) -> CornettiResult<bool> {
        let saved_otp_hash = self.store.get_otp(self.ref_domain.as_str()).await?;

        if saved_otp_hash.is_none() {
            return Ok(false);
        }

        Ok(verify_password(saved_otp_hash.unwrap().as_str(), otp))
    }
}

/// Redis-backed OTP storage with scoped keys.
///
/// Key format: `{tenant_id}:{app_id}:otp:{ref_domain}`
///
/// OTP hashes are stored as plain Redis string values with TTL set to the
/// configured expiry plus a 30-second grace period.
pub struct SimpleOtpStore {
    /// Redis connection pool.
    pub redis_conn: Arc<RedisDBService>,
    /// Tenant identifier used in the key prefix.
    pub tenant_id: String,
    /// Application identifier used in the key prefix.
    pub app_id: String,
}

impl SimpleOtpStore {
    /// Creates a new OTP store bound to the given tenant and application.
    pub fn new(redis_conn: Arc<RedisDBService>, tenant_id: String, app_id: String) -> Self {
        Self {
            redis_conn,
            tenant_id,
            app_id,
        }
    }

    #[allow(dead_code)]
    fn otp_key(&self, ref_domain: &str) -> String {
        format!("{}:{}:otp:{}", self.tenant_id, self.app_id, ref_domain)
    }

    #[allow(dead_code)]
    async fn set_otp(
        &self,
        ref_domain: &str,
        otp_hash: &str,
        expires: chrono::DateTime<chrono::Utc>,
    ) -> CornettiResult<()> {
        let key = self.otp_key(ref_domain);

        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        connection.set(&key, otp_hash).await?;

        let expires = expires + chrono::Duration::seconds(30);
        connection.expire(&key, expires.timestamp()).await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn get_otp(&self, ref_domain: &str) -> CornettiResult<Option<String>> {
        let mut connection = self
            .redis_conn
            .client()
            .get_multiplexed_async_connection()
            .await?;

        let key = self.otp_key(ref_domain);
        let otp = connection.get(&key).await?;
        Ok(otp)
    }
}
