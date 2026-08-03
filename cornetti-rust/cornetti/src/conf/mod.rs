//! TOML-based configuration loader.
//!
//! All framework configurations live in a single TOML file (default
//! `./Config.toml`, overridable with the `CORNETTI_CONF` environment
//! variable) with one section per module:
//!
//! ```toml
//! [app]                    # base server + tenant
//! [auth.jwt]               # JWT tokens and cookies
//! [auth.jwt.store]         # JWT session store
//! [auth.apikey]            # API key authentication
//! [auth.oauth2]            # OAuth2 (providers: [[auth.oauth2.providers]])
//! [redis]                  # Redis connection
//! [sqlx]                   # SQL pool ([sqlx.pool] for pool settings)
//! [mongo]                  # MongoDB connection
//! [mail]                   # provider dispatch ([mail.smtp], [mail.gmail])
//! [grpc.server]            # tonic server ([grpc.server.tls] for TLS)
//! [grpc.client]            # tonic client ([grpc.client.tls] for TLS)
//! [otp]                    # simple OTP generator
//! [templates]              # template engine
//! [filemanager]            # uploads
//! ```
//!
//! Every configuration struct implements the [`CornettiConf`] trait, so
//! application modules can register their own sections by implementing the
//! same trait for their config type.
//!
//! See `cornetti.example.toml` in the repository root for a fully commented
//! example.

use std::path::Path;

use serde::Deserialize;

use crate::core::confs::BaseConf;
use crate::core::models::CornettiResult;

#[cfg(feature = "auth")]
use crate::auth::confs::{JWTStoreConf, JwtAuthConf};
#[cfg(feature = "auth-apikey")]
use crate::auth_apikey::confs::ApiKeyAuthConf;
#[cfg(feature = "auth-oauth2")]
use crate::auth_oauth2::confs::OAuth2AuthConf;
#[cfg(feature = "filemanager")]
use crate::filemanager::confs::FileManagerConf;
#[cfg(feature = "grpc")]
use crate::grpc::confs::{GrpcClientConf, GrpcServerConf};
#[cfg(feature = "mail")]
use crate::mail::confs::BaseMailConfig;
#[cfg(feature = "mail-gmail")]
use crate::mail::gmail::confs::GmailMailConf;
#[cfg(feature = "mail")]
use crate::mail::smtp::confs::SmtpMailConf;
#[cfg(feature = "mongo")]
use crate::mongo::confs::MongoDBConfig;
#[cfg(feature = "otp")]
use crate::otp::confs::SimpleOtpConf;
#[cfg(feature = "redisdb")]
use crate::redis::confs::RedisDBConfig;
#[cfg(feature = "sqlxdb")]
use crate::sqlx::confs::SqlxDBConfig;
#[cfg(feature = "templates")]
use crate::templates::confs::TemplatesConf;

/// Default configuration file name used by [`CornettiConf::load`] when
/// `CORNETTI_CONF` is unset.
pub const DEFAULT_CONFIG_FILE: &str = "./Config.toml";

/// Environment variable selecting the main configuration file.
pub const MAIN_CONFIG_ENV_VAR: &str = "CORNETTI_CONF";

/// Prefix of the environment variables selecting per-section configuration
/// files (`CORNETTI_CONF_<SECTION>`).
pub const SECTION_CONFIG_ENV_PREFIX: &str = "CORNETTI_CONF_";

/// Trait implemented by every configuration section.
///
/// Framework sections (`BaseConf`, `JwtAuthConf`, `RedisDBConfig`, ...) and
/// application-specific modules implement this trait to register their own
/// TOML section. The trait provides the default reader: each conf loads its
/// section from the default `./Config.toml` (or the file selected by
/// `CORNETTI_CONF`), overridden by the per-section environment variable
/// `CORNETTI_CONF_<SECTION>` when set.
pub trait CornettiConf: Clone + std::fmt::Debug + Default + for<'de> Deserialize<'de> {
    /// TOML section key for this configuration, e.g. `"redis"` for the
    /// `[redis]` table. Dotted paths are allowed for nested sections
    /// (`"auth.jwt"`).
    fn section_name() -> &'static str;

    /// Environment variable selecting a per-section TOML file override,
    /// derived from [`Self::section_name`] as `CORNETTI_CONF_<SECTION>`
    /// (uppercase). Returns `None` for dotted (nested) section names.
    fn env_var() -> Option<String> {
        let name = Self::section_name();
        if name.contains('.') {
            None
        } else {
            Some(format!(
                "{SECTION_CONFIG_ENV_PREFIX}{}",
                name.to_uppercase()
            ))
        }
    }

    /// Per-section validation hook, called at the end of [`Self::load`],
    /// [`Self::from_toml_str`] and [`Self::from_toml_file`].
    ///
    /// Framework sections override this to enforce required values (e.g.
    /// `app_id` for [`BaseConf`]) or cross-field rules (e.g. OAuth2 provider
    /// checks).
    fn validate(&self) -> CornettiResult<()> {
        Ok(())
    }

    /// Default reader: loads this section from the main configuration file
    /// (`CORNETTI_CONF` or `./Config.toml`; a missing file is tolerated),
    /// then merges the per-section override file selected by
    /// [`Self::env_var`] when set.
    ///
    /// # Errors
    ///
    /// Returns a configuration error if a configured file is unreadable,
    /// invalid, or fails validation.
    fn load() -> CornettiResult<Self>
    where
        Self: Sized,
    {
        let mut section = match main_file_content()? {
            Some(content) => {
                let root: toml::Value = toml::from_str(&content)?;
                get_dotted(&root, Self::section_name())
                    .cloned()
                    .unwrap_or_else(empty_table)
            }
            None => empty_table(),
        };

        if let Some(env_var) = Self::env_var()
            && let Ok(path) = std::env::var(&env_var)
        {
            let content = std::fs::read_to_string(&path).map_err(|err| {
                crate::errors::conf::conf_missing_file().with_internal_detail(format!(
                    "Failed to read {env_var} file '{path}': {err}"
                ))
            })?;

            let overlay: toml::Value = toml::from_str(&content).map_err(|err| {
                crate::errors::conf::conf_parse_error().with_internal_detail(format!(
                    "{env_var} file '{path}' is not valid TOML: {err}"
                ))
            })?;

            match (&mut section, overlay) {
                (toml::Value::Table(base), toml::Value::Table(overlay)) => {
                    merge_tables(base, overlay);
                }
                (_, overlay) => {
                    section = overlay;
                }
            }
        }

        let conf: Self = section.try_into()?;
        conf.validate()?;
        Ok(conf)
    }

    /// Deserializes this section from a TOML string containing the section
    /// content directly (no `[section]` header).
    ///
    /// # Errors
    ///
    /// Returns a configuration error if the TOML is invalid or fails
    /// validation.
    fn from_toml_str(toml: &str) -> CornettiResult<Self>
    where
        Self: Sized,
    {
        let conf: Self = toml::from_str(toml)?;
        conf.validate()?;
        Ok(conf)
    }

    /// Deserializes this section from a TOML file containing the section
    /// content directly (no `[section]` header).
    ///
    /// # Errors
    ///
    /// Returns a configuration error if the file is missing, unreadable, or
    /// invalid.
    fn from_toml_file(path: impl AsRef<Path>) -> CornettiResult<Self>
    where
        Self: Sized,
    {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|err| {
            crate::errors::conf::conf_missing_file().with_internal_detail(format!(
                "Failed to read configuration file '{}': {err}",
                path.display()
            ))
        })?;
        Self::from_toml_str(&content)
    }
}

fn empty_table() -> toml::Value {
    toml::Value::Table(toml::map::Map::new())
}

/// Looks up a section by key, resolving dotted paths (`"auth.jwt"`) through
/// nested tables. `toml::Value::get` only supports single keys.
fn get_dotted<'v>(value: &'v toml::Value, path: &str) -> Option<&'v toml::Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

/// Reads the main configuration file content: the path from `CORNETTI_CONF`
/// or [`DEFAULT_CONFIG_FILE`]. Returns `Ok(None)` when the file does not
/// exist.
fn main_file_content() -> CornettiResult<Option<String>> {
    let path = std::env::var(MAIN_CONFIG_ENV_VAR)
        .unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());

    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(crate::errors::conf::conf_missing_file().with_internal_detail(
            format!("Failed to read configuration file '{path}': {err}"),
        )),
    }
}

/// Recursively merges `overlay` into `base`: scalar and array values replace
/// the base value, tables are merged key-by-key.
fn merge_tables(
    base: &mut toml::map::Map<String, toml::Value>,
    overlay: toml::map::Map<String, toml::Value>,
) {
    for (key, value) in overlay {
        match (base.get_mut(&key), value) {
            (Some(toml::Value::Table(base_table)), toml::Value::Table(overlay_table)) => {
                merge_tables(base_table, overlay_table);
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

impl CornettiConf for BaseConf {
    fn section_name() -> &'static str {
        "app"
    }

    fn validate(&self) -> CornettiResult<()> {
        if self.app_id.is_empty() {
            return Err(crate::errors::conf::conf_parse_error()
                .with_internal_detail("app_id is required in the [app] section"));
        }
        Ok(())
    }
}

#[cfg(feature = "auth")]
impl CornettiConf for JwtAuthConf {
    fn section_name() -> &'static str {
        "auth.jwt"
    }
}

#[cfg(feature = "auth")]
impl CornettiConf for JWTStoreConf {
    fn section_name() -> &'static str {
        "auth.jwt.store"
    }
}

#[cfg(feature = "auth-apikey")]
impl CornettiConf for ApiKeyAuthConf {
    fn section_name() -> &'static str {
        "auth.apikey"
    }
}

#[cfg(feature = "auth-oauth2")]
impl CornettiConf for OAuth2AuthConf {
    fn section_name() -> &'static str {
        "auth.oauth2"
    }

    fn validate(&self) -> CornettiResult<()> {
        OAuth2AuthConf::validate(self)
    }
}

#[cfg(feature = "redisdb")]
impl CornettiConf for RedisDBConfig {
    fn section_name() -> &'static str {
        "redis"
    }
}

#[cfg(feature = "sqlxdb")]
impl CornettiConf for SqlxDBConfig {
    fn section_name() -> &'static str {
        "sqlx"
    }
}

#[cfg(feature = "mongo")]
impl CornettiConf for MongoDBConfig {
    fn section_name() -> &'static str {
        "mongo"
    }
}

#[cfg(feature = "mail")]
impl CornettiConf for BaseMailConfig {
    fn section_name() -> &'static str {
        "mail"
    }
}

#[cfg(feature = "mail")]
impl CornettiConf for SmtpMailConf {
    fn section_name() -> &'static str {
        "mail.smtp"
    }
}

#[cfg(feature = "mail-gmail")]
impl CornettiConf for GmailMailConf {
    fn section_name() -> &'static str {
        "mail.gmail"
    }
}

#[cfg(feature = "grpc")]
impl CornettiConf for GrpcServerConf {
    fn section_name() -> &'static str {
        "grpc.server"
    }
}

#[cfg(feature = "grpc")]
impl CornettiConf for GrpcClientConf {
    fn section_name() -> &'static str {
        "grpc.client"
    }
}

#[cfg(feature = "otp")]
impl CornettiConf for SimpleOtpConf {
    fn section_name() -> &'static str {
        "otp"
    }
}

#[cfg(feature = "templates")]
impl CornettiConf for TemplatesConf {
    fn section_name() -> &'static str {
        "templates"
    }
}

#[cfg(feature = "filemanager")]
impl CornettiConf for FileManagerConf {
    fn section_name() -> &'static str {
        "filemanager"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes tests that mutate environment variables: the loaders scan
    /// every `CORNETTI_CONF_*` variable, so parallel env tests would
    /// contaminate each other.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Application-defined section used to exercise the `CornettiConf` trait
    /// extension points.
    #[derive(Clone, Debug, Default, Deserialize)]
    struct TestAppConf {
        #[serde(default)]
        api_key: String,
        #[serde(default)]
        retries: usize,
    }

    impl CornettiConf for TestAppConf {
        fn section_name() -> &'static str {
            "test_app"
        }
    }

    #[test]
    fn empty_toml_errors_without_app_id() {
        let err = BaseConf::from_toml_str("").unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_PARSE_ERROR");
    }

    #[test]
    fn minimal_toml_is_valid() {
        let conf = BaseConf::from_toml_str("app_id = \"test\"").unwrap();
        assert_eq!(conf.app_id, "test");
        assert_eq!(conf.port, 8080);
    }

    #[test]
    fn invalid_toml_errors() {
        let err = BaseConf::from_toml_str("host = [").unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_PARSE_ERROR");
        assert!(!err.internal_detail.is_empty());
    }

    #[test]
    fn defaults_apply_without_section() {
        let conf = BaseConf::default();
        assert_eq!(conf.host, "localhost");
    }

    #[test]
    fn section_env_var_derived_from_section_name() {
        assert_eq!(BaseConf::env_var(), Some("CORNETTI_CONF_APP".to_string()));
        assert_eq!(
            TestAppConf::env_var(),
            Some("CORNETTI_CONF_TEST_APP".to_string())
        );
        #[cfg(feature = "redisdb")]
        assert_eq!(RedisDBConfig::env_var(), Some("CORNETTI_CONF_REDIS".to_string()));
        // Dotted (nested) section names have no per-section env var.
        #[cfg(feature = "auth")]
        assert_eq!(JwtAuthConf::env_var(), None);
    }

    #[test]
    fn custom_section_extraction() {
        let custom = TestAppConf::from_toml_str("api_key = \"k\"\nretries = 3\n").unwrap();
        assert_eq!(custom.api_key, "k");
        assert_eq!(custom.retries, 3);
    }

    #[test]
    fn custom_section_missing_returns_default() {
        let custom = TestAppConf::from_toml_str("").unwrap();
        assert_eq!(custom.api_key, "");
        assert_eq!(custom.retries, 0);
    }

    #[test]
    fn merge_tables_recursive() {
        let mut base: toml::map::Map<String, toml::Value> =
            toml::from_str(
                r#"
                scalar = "base"
                array = [1, 2]
                [nested]
                keep = "base"
                replace = "base"
                [nested.deeper]
                x = 1
            "#,
            )
            .unwrap();
        let overlay: toml::map::Map<String, toml::Value> = toml::from_str(
            r#"
            scalar = "overlay"
            array = [3]
            [nested]
            replace = "overlay"
            new = "added"
            [nested.deeper]
            y = 2
        "#,
        )
        .unwrap();

        merge_tables(&mut base, overlay);

        assert_eq!(base["scalar"].as_str(), Some("overlay"));
        assert_eq!(base["array"].as_array().unwrap().len(), 1);
        assert_eq!(base["nested"]["keep"].as_str(), Some("base"));
        assert_eq!(base["nested"]["replace"].as_str(), Some("overlay"));
        assert_eq!(base["nested"]["new"].as_str(), Some("added"));
        assert_eq!(base["nested"]["deeper"]["x"].as_integer(), Some(1));
        assert_eq!(base["nested"]["deeper"]["y"].as_integer(), Some(2));
    }

    #[test]
    fn load_tolerates_missing_main_and_applies_overrides() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir();
        let main = dir.join("cornetti-test-main-missing.toml");
        let app = dir.join("cornetti-test-app.toml");
        let redis = dir.join("cornetti-test-redis.toml");

        std::fs::write(&app, "app_id = \"env-app\"\nport = 9090\n").unwrap();
        std::fs::write(&redis, "db_host = \"override-host\"\n").unwrap();

        // SAFETY: test-only env mutation; tests below use distinct variables.
        unsafe {
            std::env::set_var("CORNETTI_CONF", &main);
            std::env::set_var("CORNETTI_CONF_APP", &app);
            std::env::set_var("CORNETTI_CONF_REDIS", &redis);
        }

        let base_conf = BaseConf::load().unwrap();
        assert_eq!(base_conf.app_id, "env-app");
        assert_eq!(base_conf.port, 9090);
        #[cfg(feature = "redisdb")]
        {
            let redis_conf = RedisDBConfig::load().unwrap();
            assert_eq!(redis_conf.db_host, "override-host");
            assert_eq!(redis_conf.db_port, "6379");
        }

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF");
            std::env::remove_var("CORNETTI_CONF_APP");
            std::env::remove_var("CORNETTI_CONF_REDIS");
        }
        let _ = std::fs::remove_file(&app);
        let _ = std::fs::remove_file(&redis);
    }

    #[test]
    fn load_missing_main_without_app_id_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let missing = std::env::temp_dir().join("cornetti-test-never-exists.toml");
        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF", &missing);
        }
        let err = BaseConf::load().unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_PARSE_ERROR");
        assert!(err.internal_detail.contains("app_id"));
        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF");
        }
    }

    #[test]
    fn custom_section_env_override_merges() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir();
        let custom = dir.join("cornetti-test-testapp.toml");
        std::fs::write(&custom, "retries = 7\n").unwrap();

        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF_TEST_APP", &custom);
        }

        let custom_conf = TestAppConf::load().unwrap();
        assert_eq!(custom_conf.api_key, "");
        assert_eq!(custom_conf.retries, 7);

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF_TEST_APP");
        }
        let _ = std::fs::remove_file(&custom);
    }

    #[cfg(feature = "redisdb")]
    #[test]
    fn section_env_file_invalid_toml_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir();
        let redis = dir.join("cornetti-test-redis-bad.toml");
        std::fs::write(&redis, "db_host = [not valid").unwrap();

        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF_REDIS", &redis);
        }

        let err = RedisDBConfig::load().unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_PARSE_ERROR");
        assert!(err.internal_detail.contains("CORNETTI_CONF_REDIS"));

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF_REDIS");
        }
        let _ = std::fs::remove_file(&redis);
    }

    #[cfg(feature = "redisdb")]
    #[test]
    fn section_env_file_missing_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let missing = std::env::temp_dir().join("cornetti-test-redis-missing.toml");
        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF_REDIS", &missing);
        }

        let err = RedisDBConfig::load().unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_MISSING_FILE");

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF_REDIS");
        }
    }

    #[cfg(feature = "redisdb")]
    #[test]
    fn section_struct_from_toml_str_and_file() {
        let conf = RedisDBConfig::from_toml_str("db_host = \"x\"").unwrap();
        assert_eq!(conf.db_host, "x");
        assert_eq!(conf.db_port, "6379");

        let dir = std::env::temp_dir();
        let file = dir.join("cornetti-test-redis-section.toml");
        std::fs::write(&file, "db_host = \"y\"\ndb_number = \"2\"\n").unwrap();
        let conf = RedisDBConfig::from_toml_file(&file).unwrap();
        assert_eq!(conf.db_host, "y");
        assert_eq!(conf.db_number, "2");
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn section_struct_missing_file_errors() {
        let missing = std::env::temp_dir().join("cornetti-test-section-missing.toml");
        let err = BaseConf::from_toml_file(&missing).unwrap_err();
        assert_eq!(err.corr_id, "BE_CONF_MISSING_FILE");
    }

    #[cfg(feature = "redisdb")]
    #[test]
    fn section_struct_default_reader_loads_from_main_and_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir();
        let main = dir.join("cornetti-test-section-main.toml");
        let redis = dir.join("cornetti-test-section-redis.toml");

        std::fs::write(
            &main,
            "[app]\napp_id = \"t\"\n[redis]\ndb_host = \"main-host\"\ndb_port = \"7777\"\n",
        )
        .unwrap();
        std::fs::write(&redis, "db_host = \"env-host\"\n").unwrap();

        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF", &main);
            std::env::set_var("CORNETTI_CONF_REDIS", &redis);
        }

        let conf = RedisDBConfig::load().unwrap();
        assert_eq!(conf.db_host, "env-host");
        assert_eq!(conf.db_port, "7777");

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF");
            std::env::remove_var("CORNETTI_CONF_REDIS");
        }
        let _ = std::fs::remove_file(&main);
        let _ = std::fs::remove_file(&redis);
    }

    #[cfg(feature = "auth")]
    #[test]
    fn nested_section_default_reader_uses_dotted_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir();
        let main = dir.join("cornetti-test-jwt-main.toml");
        std::fs::write(
            &main,
            "[app]\napp_id = \"t\"\n[auth.jwt]\nexpire_minutes = 45\n",
        )
        .unwrap();

        // SAFETY: test-only env mutation.
        unsafe {
            std::env::set_var("CORNETTI_CONF", &main);
        }

        let conf = JwtAuthConf::load().unwrap();
        assert_eq!(conf.jwt_expire_minutes, 45);

        // SAFETY: restore clean env.
        unsafe {
            std::env::remove_var("CORNETTI_CONF");
        }
        let _ = std::fs::remove_file(&main);
    }
}
