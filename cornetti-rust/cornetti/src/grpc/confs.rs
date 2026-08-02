use crate::core::{
    confs::resolve_secret_opt,
    models::{CornettiError, CornettiResult},
};
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use tonic::transport::{Endpoint, Server};

#[cfg(feature = "grpc-tls")]
use tonic::transport::{Certificate, ClientTlsConfig, Identity, ServerTlsConfig};

/// TLS settings for the gRPC server (`[grpc.server.tls]` TOML section).
///
/// PEM material can be inlined (`certificate`, `key`, `client_ca_root`) or
/// loaded from files (`certificate_file`, `key_file`, `client_ca_root_file`).
/// TLS requires the `grpc-tls` feature.
#[derive(Clone, Debug, Default)]
pub struct GrpcServerTlsConf {
    /// Whether TLS is enabled.
    pub enable: bool,
    /// PEM-encoded TLS certificate.
    pub certificate: Option<String>,
    /// PEM-encoded TLS private key.
    pub key: Option<String>,
    /// PEM-encoded client CA certificate for mTLS.
    pub client_ca_root: Option<String>,
    /// Whether client authentication is optional.
    pub client_auth_optional: bool,
}

impl<'de> Deserialize<'de> for GrpcServerTlsConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            enable: Option<bool>,
            certificate: Option<String>,
            certificate_file: Option<String>,
            key: Option<String>,
            key_file: Option<String>,
            client_ca_root: Option<String>,
            client_ca_root_file: Option<String>,
            client_auth_optional: Option<bool>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = GrpcServerTlsConf::default();

        Ok(GrpcServerTlsConf {
            enable: raw.enable.unwrap_or(defaults.enable),
            certificate: resolve_secret_opt(raw.certificate, raw.certificate_file)
                .map_err(D::Error::custom)?,
            key: resolve_secret_opt(raw.key, raw.key_file).map_err(D::Error::custom)?,
            client_ca_root: resolve_secret_opt(raw.client_ca_root, raw.client_ca_root_file)
                .map_err(D::Error::custom)?,
            client_auth_optional: raw
                .client_auth_optional
                .unwrap_or(defaults.client_auth_optional),
        })
    }
}

impl<'de> Deserialize<'de> for GrpcServerConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            host: Option<String>,
            port: Option<u16>,
            concurrency_limit_per_connection: Option<usize>,
            timeout_secs: Option<u64>,
            initial_stream_window_size: Option<u32>,
            initial_connection_window_size: Option<u32>,
            max_concurrent_streams: Option<u32>,
            tcp_keepalive_secs: Option<u64>,
            tcp_keepalive_interval_secs: Option<u64>,
            tcp_keepalive_retries: Option<u32>,
            tcp_nodelay: Option<bool>,
            http2_keepalive_interval_secs: Option<u64>,
            http2_keepalive_timeout_secs: Option<u64>,
            http2_adaptive_window: Option<bool>,
            max_frame_size: Option<u32>,
            max_connection_age_secs: Option<u64>,
            max_connection_age_grace_secs: Option<u64>,
            accept_http1: Option<bool>,
            load_shed: Option<bool>,
            tls: Option<GrpcServerTlsConf>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = GrpcServerConf::default();

        Ok(GrpcServerConf {
            host: raw.host.unwrap_or(defaults.host),
            port: raw.port.unwrap_or(defaults.port),
            concurrency_limit_per_connection: raw.concurrency_limit_per_connection,
            timeout_secs: raw.timeout_secs,
            initial_stream_window_size: raw.initial_stream_window_size,
            initial_connection_window_size: raw.initial_connection_window_size,
            max_concurrent_streams: raw.max_concurrent_streams,
            tcp_keepalive_secs: raw.tcp_keepalive_secs,
            tcp_keepalive_interval_secs: raw.tcp_keepalive_interval_secs,
            tcp_keepalive_retries: raw.tcp_keepalive_retries,
            tcp_nodelay: raw.tcp_nodelay.unwrap_or(defaults.tcp_nodelay),
            http2_keepalive_interval_secs: raw.http2_keepalive_interval_secs,
            http2_keepalive_timeout_secs: raw.http2_keepalive_timeout_secs,
            http2_adaptive_window: raw.http2_adaptive_window,
            max_frame_size: raw.max_frame_size,
            max_connection_age_secs: raw.max_connection_age_secs,
            max_connection_age_grace_secs: raw.max_connection_age_grace_secs,
            accept_http1: raw.accept_http1.unwrap_or(defaults.accept_http1),
            load_shed: raw.load_shed.unwrap_or(defaults.load_shed),
            tls: raw.tls.unwrap_or_default(),
        })
    }
}

/// gRPC server configuration (`[grpc.server]` TOML section).
///
/// TLS configuration requires the `grpc-tls` feature.
#[derive(Clone, Debug)]
pub struct GrpcServerConf {
    /// Host to bind to (default: `0.0.0.0`).
    pub host: String,
    /// Port to bind to (default: `50051`).
    pub port: u16,
    /// Max concurrent requests per connection.
    pub concurrency_limit_per_connection: Option<usize>,
    /// Per-request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Initial HTTP/2 stream window size.
    pub initial_stream_window_size: Option<u32>,
    /// Initial HTTP/2 connection window size.
    pub initial_connection_window_size: Option<u32>,
    /// Max concurrent streams per connection.
    pub max_concurrent_streams: Option<u32>,
    /// TCP keepalive time in seconds.
    pub tcp_keepalive_secs: Option<u64>,
    /// TCP keepalive interval in seconds.
    pub tcp_keepalive_interval_secs: Option<u64>,
    /// TCP keepalive retry count.
    pub tcp_keepalive_retries: Option<u32>,
    /// Whether TCP_NODELAY is enabled (default: `true`).
    pub tcp_nodelay: bool,
    /// HTTP/2 PING interval in seconds.
    pub http2_keepalive_interval_secs: Option<u64>,
    /// HTTP/2 PING timeout in seconds.
    pub http2_keepalive_timeout_secs: Option<u64>,
    /// Whether HTTP/2 adaptive window is enabled.
    pub http2_adaptive_window: Option<bool>,
    /// Max HTTP/2 frame size.
    pub max_frame_size: Option<u32>,
    /// Max connection age in seconds.
    pub max_connection_age_secs: Option<u64>,
    /// Grace period after max connection age in seconds.
    pub max_connection_age_grace_secs: Option<u64>,
    /// Whether to accept HTTP/1 requests (default: `false`).
    pub accept_http1: bool,
    /// Whether load shedding is enabled (default: `false`).
    pub load_shed: bool,
    /// TLS settings.
    pub tls: GrpcServerTlsConf,
}

impl Default for GrpcServerConf {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            concurrency_limit_per_connection: None,
            timeout_secs: None,
            initial_stream_window_size: None,
            initial_connection_window_size: None,
            max_concurrent_streams: None,
            tcp_keepalive_secs: None,
            tcp_keepalive_interval_secs: None,
            tcp_keepalive_retries: None,
            tcp_nodelay: true,
            http2_keepalive_interval_secs: None,
            http2_keepalive_timeout_secs: None,
            http2_adaptive_window: None,
            max_frame_size: None,
            max_connection_age_secs: None,
            max_connection_age_grace_secs: None,
            accept_http1: false,
            load_shed: false,
            tls: GrpcServerTlsConf::default(),
        }
    }
}

impl GrpcServerConf {
    /// Returns `host:port` as a string.
    pub fn bind_address(&self) -> String {
        host_with_port(&self.host, self.port)
    }

    /// Resolves the host:port to a `SocketAddr`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the address cannot be resolved.
    pub fn socket_address(&self) -> io::Result<SocketAddr> {
        let host = self.host.trim_start_matches('[').trim_end_matches(']');

        (host, self.port).to_socket_addrs()?.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid gRPC server address")
        })
    }

    /// Whether TLS is enabled and both certificate and key are configured.
    pub fn has_tls_identity(&self) -> bool {
        self.tls.enable && self.tls.certificate.is_some() && self.tls.key.is_some()
    }

    /// Whether mTLS client authentication is configured.
    pub fn has_tls_client_auth(&self) -> bool {
        self.tls.enable && self.tls.client_ca_root.is_some()
    }

    /// Builds a `ServerTlsConfig` if TLS is enabled.
    ///
    /// Requires the `grpc-tls` feature. Without it, returns `None` if TLS is
    /// disabled, or an error if TLS is enabled.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if required TLS settings are missing or keys are invalid.
    #[cfg(feature = "grpc-tls")]
    pub fn tls_config(&self) -> CornettiResult<Option<ServerTlsConfig>> {
        if !self.tls.enable {
            return Ok(None);
        }

        let certificate = self.tls.certificate.as_deref().ok_or_else(|| {
            grpc_config_error("Missing tls.certificate (or tls.certificate_file) in [grpc.server]")
        })?;
        let key = self.tls.key.as_deref().ok_or_else(|| {
            grpc_config_error("Missing tls.key (or tls.key_file) in [grpc.server]")
        })?;

        let mut config = ServerTlsConfig::new().identity(Identity::from_pem(certificate, key));

        if let Some(client_ca_root) = &self.tls.client_ca_root {
            config = config
                .client_ca_root(Certificate::from_pem(client_ca_root))
                .client_auth_optional(self.tls.client_auth_optional);
        }

        Ok(Some(config))
    }

    /// Builds a fully configured tonic `Server` from this config.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if TLS is enabled but TLS configuration fails.
    pub fn builder(&self) -> CornettiResult<Server> {
        self.apply_tls_to_server(self.apply_to_server(Server::builder()))
    }

    /// Applies all non-TLS settings from this config to a tonic `Server`.
    pub fn apply_to_server<L>(&self, server: Server<L>) -> Server<L> {
        let mut server = server
            .load_shed(self.load_shed)
            .tcp_nodelay(self.tcp_nodelay)
            .accept_http1(self.accept_http1)
            .initial_stream_window_size(self.initial_stream_window_size)
            .initial_connection_window_size(self.initial_connection_window_size)
            .max_concurrent_streams(self.max_concurrent_streams)
            .tcp_keepalive(self.tcp_keepalive_secs.map(Duration::from_secs))
            .tcp_keepalive_interval(self.tcp_keepalive_interval_secs.map(Duration::from_secs))
            .tcp_keepalive_retries(self.tcp_keepalive_retries)
            .http2_keepalive_interval(self.http2_keepalive_interval_secs.map(Duration::from_secs))
            .http2_keepalive_timeout(self.http2_keepalive_timeout_secs.map(Duration::from_secs))
            .http2_adaptive_window(self.http2_adaptive_window)
            .max_frame_size(self.max_frame_size);

        if let Some(value) = self.concurrency_limit_per_connection {
            server = server.concurrency_limit_per_connection(value);
        }

        if let Some(value) = self.timeout_secs {
            server = server.timeout(Duration::from_secs(value));
        }

        if let Some(value) = self.max_connection_age_secs {
            server = server.max_connection_age(Duration::from_secs(value));
        }

        if let Some(value) = self.max_connection_age_grace_secs {
            server = server.max_connection_age_grace(Duration::from_secs(value));
        }

        server
    }
}

/// TLS settings for the gRPC client (`[grpc.client.tls]` TOML section).
///
/// PEM material can be inlined (`ca_certificate`, `certificate`, `key`) or
/// loaded from files (`ca_certificate_file`, `certificate_file`, `key_file`).
/// TLS requires the `grpc-tls` feature.
#[derive(Clone, Debug, Default)]
pub struct GrpcClientTlsConf {
    /// TLS SNI domain name.
    pub domain_name: Option<String>,
    /// PEM-encoded CA certificate.
    pub ca_certificate: Option<String>,
    /// PEM-encoded client certificate.
    pub certificate: Option<String>,
    /// PEM-encoded client private key.
    pub key: Option<String>,
}

impl<'de> Deserialize<'de> for GrpcClientTlsConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            domain_name: Option<String>,
            ca_certificate: Option<String>,
            ca_certificate_file: Option<String>,
            certificate: Option<String>,
            certificate_file: Option<String>,
            key: Option<String>,
            key_file: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        Ok(GrpcClientTlsConf {
            domain_name: raw.domain_name,
            ca_certificate: resolve_secret_opt(raw.ca_certificate, raw.ca_certificate_file)
                .map_err(D::Error::custom)?,
            certificate: resolve_secret_opt(raw.certificate, raw.certificate_file)
                .map_err(D::Error::custom)?,
            key: resolve_secret_opt(raw.key, raw.key_file).map_err(D::Error::custom)?,
        })
    }
}

impl<'de> Deserialize<'de> for GrpcClientConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            endpoint: Option<String>,
            host: Option<String>,
            port: Option<u16>,
            use_tls: Option<bool>,
            user_agent: Option<String>,
            timeout_secs: Option<u64>,
            connect_timeout_secs: Option<u64>,
            concurrency_limit: Option<usize>,
            rate_limit_requests: Option<u64>,
            rate_limit_duration_secs: Option<u64>,
            initial_stream_window_size: Option<u32>,
            initial_connection_window_size: Option<u32>,
            buffer_size: Option<usize>,
            tcp_keepalive_secs: Option<u64>,
            tcp_keepalive_interval_secs: Option<u64>,
            tcp_keepalive_retries: Option<u32>,
            tcp_nodelay: Option<bool>,
            http2_keepalive_interval_secs: Option<u64>,
            keep_alive_timeout_secs: Option<u64>,
            keep_alive_while_idle: Option<bool>,
            http2_adaptive_window: Option<bool>,
            max_frame_size: Option<u32>,
            tls: Option<GrpcClientTlsConf>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = GrpcClientConf::default();

        Ok(GrpcClientConf {
            endpoint: non_empty(raw.endpoint),
            host: raw.host.unwrap_or(defaults.host),
            port: raw.port.unwrap_or(defaults.port),
            use_tls: raw.use_tls.unwrap_or(defaults.use_tls),
            user_agent: non_empty(raw.user_agent),
            timeout_secs: raw.timeout_secs,
            connect_timeout_secs: raw.connect_timeout_secs,
            concurrency_limit: raw.concurrency_limit,
            rate_limit_requests: raw.rate_limit_requests,
            rate_limit_duration_secs: raw.rate_limit_duration_secs,
            initial_stream_window_size: raw.initial_stream_window_size,
            initial_connection_window_size: raw.initial_connection_window_size,
            buffer_size: raw.buffer_size,
            tcp_keepalive_secs: raw.tcp_keepalive_secs,
            tcp_keepalive_interval_secs: raw.tcp_keepalive_interval_secs,
            tcp_keepalive_retries: raw.tcp_keepalive_retries,
            tcp_nodelay: raw.tcp_nodelay.unwrap_or(defaults.tcp_nodelay),
            http2_keepalive_interval_secs: raw.http2_keepalive_interval_secs,
            keep_alive_timeout_secs: raw.keep_alive_timeout_secs,
            keep_alive_while_idle: raw.keep_alive_while_idle,
            http2_adaptive_window: raw.http2_adaptive_window,
            max_frame_size: raw.max_frame_size,
            tls: raw.tls.unwrap_or_default(),
        })
    }
}

/// gRPC client configuration (`[grpc.client]` TOML section).
///
/// Supports connecting via endpoint URI or host:port. TLS configuration
/// requires the `grpc-tls` feature.
#[derive(Clone, Debug)]
pub struct GrpcClientConf {
    /// Full endpoint URI (overrides host/port if set).
    pub endpoint: Option<String>,
    /// Host address (default: `"localhost"`).
    pub host: String,
    /// Port number (default: `50051`).
    pub port: u16,
    /// Whether TLS is used (default: `false`).
    pub use_tls: bool,
    /// User-Agent string.
    pub user_agent: Option<String>,
    /// Per-request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Connection timeout in seconds.
    pub connect_timeout_secs: Option<u64>,
    /// Max concurrent requests.
    pub concurrency_limit: Option<usize>,
    /// Rate limit: number of requests.
    pub rate_limit_requests: Option<u64>,
    /// Rate limit: duration in seconds.
    pub rate_limit_duration_secs: Option<u64>,
    /// Initial HTTP/2 stream window size.
    pub initial_stream_window_size: Option<u32>,
    /// Initial HTTP/2 connection window size.
    pub initial_connection_window_size: Option<u32>,
    /// Buffer size hint.
    pub buffer_size: Option<usize>,
    /// TCP keepalive time in seconds.
    pub tcp_keepalive_secs: Option<u64>,
    /// TCP keepalive interval in seconds.
    pub tcp_keepalive_interval_secs: Option<u64>,
    /// TCP keepalive retry count.
    pub tcp_keepalive_retries: Option<u32>,
    /// Whether TCP_NODELAY is enabled (default: `true`).
    pub tcp_nodelay: bool,
    /// HTTP/2 PING interval in seconds.
    pub http2_keepalive_interval_secs: Option<u64>,
    /// Keepalive timeout in seconds.
    pub keep_alive_timeout_secs: Option<u64>,
    /// Whether to send keepalive PINGs while idle.
    pub keep_alive_while_idle: Option<bool>,
    /// Whether HTTP/2 adaptive window is enabled.
    pub http2_adaptive_window: Option<bool>,
    /// Max HTTP/2 frame size.
    pub max_frame_size: Option<u32>,
    /// TLS settings.
    pub tls: GrpcClientTlsConf,
}

impl Default for GrpcClientConf {
    fn default() -> Self {
        Self {
            endpoint: None,
            host: "localhost".to_string(),
            port: 50051,
            use_tls: false,
            user_agent: None,
            timeout_secs: None,
            connect_timeout_secs: None,
            concurrency_limit: None,
            rate_limit_requests: None,
            rate_limit_duration_secs: None,
            initial_stream_window_size: None,
            initial_connection_window_size: None,
            buffer_size: None,
            tcp_keepalive_secs: None,
            tcp_keepalive_interval_secs: None,
            tcp_keepalive_retries: None,
            tcp_nodelay: true,
            http2_keepalive_interval_secs: None,
            keep_alive_timeout_secs: None,
            keep_alive_while_idle: None,
            http2_adaptive_window: None,
            max_frame_size: None,
            tls: GrpcClientTlsConf::default(),
        }
    }
}

impl GrpcClientConf {
    /// Returns `"https"` if TLS is enabled, `"http"` otherwise.
    pub fn scheme(&self) -> &'static str {
        if self.tls_enabled() { "https" } else { "http" }
    }

    /// Whether TLS is enabled (explicit or implied by `https://` endpoint).
    pub fn tls_enabled(&self) -> bool {
        self.use_tls
            || self
                .endpoint
                .as_deref()
                .map(|endpoint| endpoint.trim().to_ascii_lowercase().starts_with("https://"))
                .unwrap_or(false)
    }

    /// Returns `host:port` as a string.
    pub fn authority(&self) -> String {
        host_with_port(&self.host, self.port)
    }

    /// Returns the endpoint URI (from config or built from host/port/scheme).
    pub fn endpoint_uri(&self) -> String {
        self.endpoint
            .clone()
            .unwrap_or_else(|| format!("{}://{}", self.scheme(), self.authority()))
    }

    /// Whether TLS is enabled and both client cert and key are configured.
    pub fn has_tls_identity(&self) -> bool {
        self.tls_enabled() && self.tls.certificate.is_some() && self.tls.key.is_some()
    }

    /// Whether TLS is enabled and a CA certificate is configured.
    pub fn has_tls_ca_certificate(&self) -> bool {
        self.tls_enabled() && self.tls.ca_certificate.is_some()
    }

    /// Builds a `ClientTlsConfig` if TLS is enabled.
    ///
    /// Requires the `grpc-tls` feature.
    ///
    /// # Errors
    ///
    /// Returns a 500 error if both cert and key are not provided together.
    #[cfg(feature = "grpc-tls")]
    pub fn tls_config(&self) -> CornettiResult<Option<ClientTlsConfig>> {
        if !self.tls_enabled() {
            return Ok(None);
        }

        let mut config = ClientTlsConfig::new().with_enabled_roots();

        if let Some(domain_name) = &self.tls.domain_name {
            config = config.domain_name(domain_name.clone());
        }

        if let Some(ca_certificate) = &self.tls.ca_certificate {
            config = config.ca_certificate(Certificate::from_pem(ca_certificate));
        }

        match (&self.tls.certificate, &self.tls.key) {
            (Some(certificate), Some(key)) => {
                config = config.identity(Identity::from_pem(certificate, key));
            }
            (None, None) => {}
            _ => {
                return Err(grpc_config_error(
                    "tls.certificate and tls.key must both be configured in [grpc.client]",
                ));
            }
        }

        Ok(Some(config))
    }

    /// Builds a fully configured tonic `Endpoint` from this config.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if the endpoint URI is invalid or TLS
    /// configuration fails.
    pub fn builder(&self) -> CornettiResult<Endpoint> {
        let endpoint = self.apply_to_endpoint(Endpoint::from_shared(self.endpoint_uri())?)?;
        self.apply_tls_to_endpoint(endpoint)
    }

    /// Applies all non-TLS settings from this config to a tonic `Endpoint`.
    pub fn apply_to_endpoint(
        &self,
        endpoint: Endpoint,
    ) -> Result<Endpoint, tonic::transport::Error> {
        let mut endpoint = endpoint
            .initial_stream_window_size(self.initial_stream_window_size)
            .initial_connection_window_size(self.initial_connection_window_size)
            .buffer_size(self.buffer_size)
            .tcp_keepalive(self.tcp_keepalive_secs.map(Duration::from_secs))
            .tcp_keepalive_interval(self.tcp_keepalive_interval_secs.map(Duration::from_secs))
            .tcp_keepalive_retries(self.tcp_keepalive_retries)
            .tcp_nodelay(self.tcp_nodelay)
            .max_frame_size(self.max_frame_size);

        if let Some(user_agent) = &self.user_agent {
            endpoint = endpoint.user_agent(user_agent.clone())?;
        }

        if let Some(value) = self.timeout_secs {
            endpoint = endpoint.timeout(Duration::from_secs(value));
        }

        if let Some(value) = self.connect_timeout_secs {
            endpoint = endpoint.connect_timeout(Duration::from_secs(value));
        }

        if let Some(value) = self.concurrency_limit {
            endpoint = endpoint.concurrency_limit(value);
        }

        if let (Some(limit), Some(duration_secs)) =
            (self.rate_limit_requests, self.rate_limit_duration_secs)
        {
            endpoint = endpoint.rate_limit(limit, Duration::from_secs(duration_secs));
        }

        if let Some(value) = self.http2_keepalive_interval_secs {
            endpoint = endpoint.http2_keep_alive_interval(Duration::from_secs(value));
        }

        if let Some(value) = self.keep_alive_timeout_secs {
            endpoint = endpoint.keep_alive_timeout(Duration::from_secs(value));
        }

        if let Some(value) = self.keep_alive_while_idle {
            endpoint = endpoint.keep_alive_while_idle(value);
        }

        if let Some(value) = self.http2_adaptive_window {
            endpoint = endpoint.http2_adaptive_window(value);
        }

        Ok(endpoint)
    }

    /// Builds a tonic `Endpoint` from this configuration.
    ///
    /// Alias for `builder()`.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if the endpoint URI is invalid.
    pub fn tonic_endpoint(&self) -> CornettiResult<Endpoint> {
        self.builder()
    }
}

fn grpc_config_error(detail: impl Into<String>) -> CornettiError {
    crate::errors::grpc::grpc_config_error()
        .with_internal_detail(detail.into())
}

#[cfg(feature = "grpc-tls")]
impl GrpcServerConf {
    fn apply_tls_to_server(&self, mut server: Server) -> CornettiResult<Server> {
        if let Some(tls_config) = self.tls_config()? {
            server = server.tls_config(tls_config)?;
        }

        Ok(server)
    }
}

#[cfg(not(feature = "grpc-tls"))]
impl GrpcServerConf {
    fn apply_tls_to_server(&self, server: Server) -> CornettiResult<Server> {
        if self.tls.enable {
            return Err(grpc_tls_feature_error());
        }

        Ok(server)
    }
}

#[cfg(feature = "grpc-tls")]
impl GrpcClientConf {
    fn apply_tls_to_endpoint(&self, mut endpoint: Endpoint) -> CornettiResult<Endpoint> {
        if let Some(tls_config) = self.tls_config()? {
            endpoint = endpoint.tls_config(tls_config)?;
        }

        Ok(endpoint)
    }
}

#[cfg(not(feature = "grpc-tls"))]
impl GrpcClientConf {
    fn apply_tls_to_endpoint(&self, endpoint: Endpoint) -> CornettiResult<Endpoint> {
        if self.tls_enabled() {
            return Err(grpc_tls_feature_error());
        }

        Ok(endpoint)
    }
}

#[cfg(not(feature = "grpc-tls"))]
fn grpc_tls_feature_error() -> CornettiError {
    grpc_config_error("Enable `grpc-tls` feature to use gRPC TLS helpers")
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn host_with_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{}]:{}", host, port)
    } else {
        format!("{}:{}", host, port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_server_conf_from_toml_defaults() {
        let conf: GrpcServerConf = toml::from_str("").unwrap();
        assert_eq!(conf.host, "0.0.0.0");
        assert_eq!(conf.port, 50051);
        assert!(conf.tcp_nodelay);
        assert!(!conf.accept_http1);
        assert!(!conf.load_shed);
        assert!(!conf.tls.enable);
        assert!(conf.tls.certificate.is_none());
    }

    #[test]
    fn grpc_server_conf_from_toml_with_tls() {
        let toml = r#"
            host = "127.0.0.1"
            port = 50052
            load_shed = true

            [tls]
            enable = true
            certificate = "CERT"
            key = "KEY"
            client_ca_root = "CA"
            client_auth_optional = true
        "#;
        let conf: GrpcServerConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.host, "127.0.0.1");
        assert_eq!(conf.port, 50052);
        assert!(conf.load_shed);
        assert!(conf.tls.enable);
        assert_eq!(conf.tls.certificate.as_deref(), Some("CERT"));
        assert_eq!(conf.tls.key.as_deref(), Some("KEY"));
        assert_eq!(conf.tls.client_ca_root.as_deref(), Some("CA"));
        assert!(conf.tls.client_auth_optional);
        assert!(conf.has_tls_identity());
        assert!(conf.has_tls_client_auth());
    }

    #[test]
    fn grpc_server_conf_tls_both_forms_errors() {
        let result = toml::from_str::<GrpcServerConf>("[tls]\ncertificate = \"A\"\ncertificate_file = \"/x\"");
        assert!(result.is_err());
    }

    #[test]
    fn grpc_client_conf_from_toml_defaults() {
        let conf: GrpcClientConf = toml::from_str("").unwrap();
        assert_eq!(conf.host, "localhost");
        assert_eq!(conf.port, 50051);
        assert!(!conf.use_tls);
        assert!(conf.tcp_nodelay);
        assert_eq!(conf.endpoint_uri(), "http://localhost:50051");
    }

    #[test]
    fn grpc_client_conf_from_toml() {
        let toml = r#"
            endpoint = "http://grpc.example.com:8080"
            use_tls = true
            user_agent = "cornetti-client"
            timeout_secs = 10
            connect_timeout_secs = 5

            [tls]
            domain_name = "grpc.example.com"
            ca_certificate = "CA"
        "#;
        let conf: GrpcClientConf = toml::from_str(toml).unwrap();
        assert_eq!(conf.endpoint_uri(), "http://grpc.example.com:8080");
        assert_eq!(conf.user_agent.as_deref(), Some("cornetti-client"));
        assert_eq!(conf.timeout_secs, Some(10));
        assert_eq!(conf.connect_timeout_secs, Some(5));
        assert_eq!(conf.tls.domain_name.as_deref(), Some("grpc.example.com"));
        assert_eq!(conf.tls.ca_certificate.as_deref(), Some("CA"));
        assert!(conf.has_tls_ca_certificate());
    }

    #[test]
    fn grpc_client_endpoint_implies_tls() {
        let conf: GrpcClientConf = toml::from_str("endpoint = \"https://grpc.example.com\"").unwrap();
        assert!(conf.tls_enabled());
        assert_eq!(conf.scheme(), "https");
    }
}
