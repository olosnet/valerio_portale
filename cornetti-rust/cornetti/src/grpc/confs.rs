use crate::core::{
    helpers::common::env_or_envfile,
    models::{CornettiError, CornettiResult},
};
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};
use tonic::transport::{Endpoint, Server};

#[cfg(feature = "grpc-tls")]
use tonic::transport::{Certificate, ClientTlsConfig, Identity, ServerTlsConfig};

/// gRPC server configuration.
///
/// All fields are read from environment variables with sensible defaults.
/// TLS configuration requires the `grpc-tls` feature.
#[derive(Clone)]
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
    /// Whether TCP_NODELAY is enabled.
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
    /// Whether to accept HTTP/1 requests.
    pub accept_http1: bool,
    /// Whether load shedding is enabled.
    pub load_shed: bool,
    /// Whether TLS is enabled.
    pub tls_enable: bool,
    /// PEM-encoded TLS certificate.
    pub tls_certificate: Option<String>,
    /// PEM-encoded TLS private key.
    pub tls_key: Option<String>,
    /// PEM-encoded client CA certificate for mTLS.
    pub tls_client_ca_root: Option<String>,
    /// Whether client authentication is optional.
    pub tls_client_auth_optional: bool,
}

impl GrpcServerConf {
    /// Reads configuration from environment variables.
    ///
    /// See the source for the full list of `GRPC_SERVER_*` variables.
    pub fn from_env() -> Self {
        let host = std::env::var("GRPC_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("GRPC_SERVER_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .unwrap_or(50051);

        let concurrency_limit_per_connection =
            env_parse::<usize>("GRPC_SERVER_CONCURRENCY_LIMIT_PER_CONNECTION");
        let timeout_secs = env_parse::<u64>("GRPC_SERVER_TIMEOUT_SECS");
        let initial_stream_window_size = env_parse::<u32>("GRPC_SERVER_INITIAL_STREAM_WINDOW_SIZE");
        let initial_connection_window_size =
            env_parse::<u32>("GRPC_SERVER_INITIAL_CONNECTION_WINDOW_SIZE");
        let max_concurrent_streams = env_parse::<u32>("GRPC_SERVER_MAX_CONCURRENT_STREAMS");
        let tcp_keepalive_secs = env_parse::<u64>("GRPC_SERVER_TCP_KEEPALIVE_SECS");
        let tcp_keepalive_interval_secs =
            env_parse::<u64>("GRPC_SERVER_TCP_KEEPALIVE_INTERVAL_SECS");
        let tcp_keepalive_retries = env_parse::<u32>("GRPC_SERVER_TCP_KEEPALIVE_RETRIES");
        let tcp_nodelay = std::env::var("GRPC_SERVER_TCP_NODELAY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let http2_keepalive_interval_secs =
            env_parse::<u64>("GRPC_SERVER_HTTP2_KEEPALIVE_INTERVAL_SECS");
        let http2_keepalive_timeout_secs =
            env_parse::<u64>("GRPC_SERVER_HTTP2_KEEPALIVE_TIMEOUT_SECS");
        let http2_adaptive_window = env_parse::<bool>("GRPC_SERVER_HTTP2_ADAPTIVE_WINDOW");
        let max_frame_size = env_parse::<u32>("GRPC_SERVER_MAX_FRAME_SIZE");
        let max_connection_age_secs = env_parse::<u64>("GRPC_SERVER_MAX_CONNECTION_AGE_SECS");
        let max_connection_age_grace_secs =
            env_parse::<u64>("GRPC_SERVER_MAX_CONNECTION_AGE_GRACE_SECS");
        let accept_http1 = std::env::var("GRPC_SERVER_ACCEPT_HTTP1")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let load_shed = std::env::var("GRPC_SERVER_LOAD_SHED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let tls_enable = std::env::var("GRPC_SERVER_TLS_ENABLE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let tls_certificate =
            env_or_envfile_non_empty("GRPC_SERVER_TLS_CERT", "GRPC_SERVER_TLS_CERT_FILE");
        let tls_key = env_or_envfile_non_empty("GRPC_SERVER_TLS_KEY", "GRPC_SERVER_TLS_KEY_FILE");
        let tls_client_ca_root = env_or_envfile_non_empty(
            "GRPC_SERVER_TLS_CLIENT_CA_ROOT",
            "GRPC_SERVER_TLS_CLIENT_CA_ROOT_FILE",
        );
        let tls_client_auth_optional = std::env::var("GRPC_SERVER_TLS_CLIENT_AUTH_OPTIONAL")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Self {
            host,
            port,
            concurrency_limit_per_connection,
            timeout_secs,
            initial_stream_window_size,
            initial_connection_window_size,
            max_concurrent_streams,
            tcp_keepalive_secs,
            tcp_keepalive_interval_secs,
            tcp_keepalive_retries,
            tcp_nodelay,
            http2_keepalive_interval_secs,
            http2_keepalive_timeout_secs,
            http2_adaptive_window,
            max_frame_size,
            max_connection_age_secs,
            max_connection_age_grace_secs,
            accept_http1,
            load_shed,
            tls_enable,
            tls_certificate,
            tls_key,
            tls_client_ca_root,
            tls_client_auth_optional,
        }
    }

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
        self.tls_enable && self.tls_certificate.is_some() && self.tls_key.is_some()
    }

    /// Whether mTLS client authentication is configured.
    pub fn has_tls_client_auth(&self) -> bool {
        self.tls_enable && self.tls_client_ca_root.is_some()
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
        if !self.tls_enable {
            return Ok(None);
        }

        let certificate = self.tls_certificate.as_deref().ok_or_else(|| {
            grpc_config_error("Missing GRPC_SERVER_TLS_CERT or GRPC_SERVER_TLS_CERT_FILE")
        })?;
        let key = self.tls_key.as_deref().ok_or_else(|| {
            grpc_config_error("Missing GRPC_SERVER_TLS_KEY or GRPC_SERVER_TLS_KEY_FILE")
        })?;

        let mut config = ServerTlsConfig::new().identity(Identity::from_pem(certificate, key));

        if let Some(client_ca_root) = &self.tls_client_ca_root {
            config = config
                .client_ca_root(Certificate::from_pem(client_ca_root))
                .client_auth_optional(self.tls_client_auth_optional);
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

/// gRPC client configuration.
///
/// Supports connecting via endpoint URI or host:port. TLS configuration
/// requires the `grpc-tls` feature.
#[derive(Clone)]
pub struct GrpcClientConf {
    /// Full endpoint URI (overrides host/port if set).
    pub endpoint: Option<String>,
    /// Host address.
    pub host: String,
    /// Port number.
    pub port: u16,
    /// Whether TLS is used.
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
    /// Whether TCP_NODELAY is enabled.
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
    /// TLS SNI domain name.
    pub tls_domain_name: Option<String>,
    /// PEM-encoded CA certificate.
    pub tls_ca_certificate: Option<String>,
    /// PEM-encoded client certificate.
    pub tls_certificate: Option<String>,
    /// PEM-encoded client private key.
    pub tls_key: Option<String>,
}

impl GrpcClientConf {
    /// Reads configuration from environment variables.
    ///
    /// See the source for the full list of `GRPC_CLIENT_*` variables.
    pub fn from_env() -> Self {
        let endpoint = non_empty(std::env::var("GRPC_CLIENT_ENDPOINT").ok());
        let host = std::env::var("GRPC_CLIENT_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("GRPC_CLIENT_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .unwrap_or(50051);
        let use_tls = std::env::var("GRPC_CLIENT_USE_TLS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let user_agent = non_empty(std::env::var("GRPC_CLIENT_USER_AGENT").ok());
        let timeout_secs = env_parse::<u64>("GRPC_CLIENT_TIMEOUT_SECS");
        let connect_timeout_secs = env_parse::<u64>("GRPC_CLIENT_CONNECT_TIMEOUT_SECS");
        let concurrency_limit = env_parse::<usize>("GRPC_CLIENT_CONCURRENCY_LIMIT");
        let rate_limit_requests = env_parse::<u64>("GRPC_CLIENT_RATE_LIMIT_REQUESTS");
        let rate_limit_duration_secs = env_parse::<u64>("GRPC_CLIENT_RATE_LIMIT_DURATION_SECS");
        let initial_stream_window_size = env_parse::<u32>("GRPC_CLIENT_INITIAL_STREAM_WINDOW_SIZE");
        let initial_connection_window_size =
            env_parse::<u32>("GRPC_CLIENT_INITIAL_CONNECTION_WINDOW_SIZE");
        let buffer_size = env_parse::<usize>("GRPC_CLIENT_BUFFER_SIZE");
        let tcp_keepalive_secs = env_parse::<u64>("GRPC_CLIENT_TCP_KEEPALIVE_SECS");
        let tcp_keepalive_interval_secs =
            env_parse::<u64>("GRPC_CLIENT_TCP_KEEPALIVE_INTERVAL_SECS");
        let tcp_keepalive_retries = env_parse::<u32>("GRPC_CLIENT_TCP_KEEPALIVE_RETRIES");
        let tcp_nodelay = std::env::var("GRPC_CLIENT_TCP_NODELAY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let http2_keepalive_interval_secs =
            env_parse::<u64>("GRPC_CLIENT_HTTP2_KEEPALIVE_INTERVAL_SECS");
        let keep_alive_timeout_secs = env_parse::<u64>("GRPC_CLIENT_KEEP_ALIVE_TIMEOUT_SECS");
        let keep_alive_while_idle = env_parse::<bool>("GRPC_CLIENT_KEEP_ALIVE_WHILE_IDLE");
        let http2_adaptive_window = env_parse::<bool>("GRPC_CLIENT_HTTP2_ADAPTIVE_WINDOW");
        let max_frame_size = env_parse::<u32>("GRPC_CLIENT_MAX_FRAME_SIZE");

        let tls_domain_name = non_empty(std::env::var("GRPC_CLIENT_TLS_DOMAIN_NAME").ok());
        let tls_ca_certificate =
            env_or_envfile_non_empty("GRPC_CLIENT_TLS_CA_CERT", "GRPC_CLIENT_TLS_CA_CERT_FILE");
        let tls_certificate =
            env_or_envfile_non_empty("GRPC_CLIENT_TLS_CERT", "GRPC_CLIENT_TLS_CERT_FILE");
        let tls_key = env_or_envfile_non_empty("GRPC_CLIENT_TLS_KEY", "GRPC_CLIENT_TLS_KEY_FILE");

        Self {
            endpoint,
            host,
            port,
            use_tls,
            user_agent,
            timeout_secs,
            connect_timeout_secs,
            concurrency_limit,
            rate_limit_requests,
            rate_limit_duration_secs,
            initial_stream_window_size,
            initial_connection_window_size,
            buffer_size,
            tcp_keepalive_secs,
            tcp_keepalive_interval_secs,
            tcp_keepalive_retries,
            tcp_nodelay,
            http2_keepalive_interval_secs,
            keep_alive_timeout_secs,
            keep_alive_while_idle,
            http2_adaptive_window,
            max_frame_size,
            tls_domain_name,
            tls_ca_certificate,
            tls_certificate,
            tls_key,
        }
    }

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
        self.tls_enabled() && self.tls_certificate.is_some() && self.tls_key.is_some()
    }

    /// Whether TLS is enabled and a CA certificate is configured.
    pub fn has_tls_ca_certificate(&self) -> bool {
        self.tls_enabled() && self.tls_ca_certificate.is_some()
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

        if let Some(domain_name) = &self.tls_domain_name {
            config = config.domain_name(domain_name.clone());
        }

        if let Some(ca_certificate) = &self.tls_ca_certificate {
            config = config.ca_certificate(Certificate::from_pem(ca_certificate));
        }

        match (&self.tls_certificate, &self.tls_key) {
            (Some(certificate), Some(key)) => {
                config = config.identity(Identity::from_pem(certificate, key));
            }
            (None, None) => {}
            _ => {
                return Err(grpc_config_error(
                    "GRPC_CLIENT_TLS_CERT and GRPC_CLIENT_TLS_KEY must both be configured",
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
        if self.tls_enable {
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

fn env_parse<T>(key: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    std::env::var(key).ok()?.parse().ok()
}

fn env_or_envfile_non_empty(env: &str, env_file: &str) -> Option<String> {
    non_empty(env_or_envfile(env, env_file))
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
