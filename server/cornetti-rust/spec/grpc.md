# Module: grpc (src/grpc/)

## Purpose

Provides gRPC integration via tonic: server and client configuration builders,
TLS support (optional `grpc-tls` feature), and bidirectional error conversion
between `CornettiError` and `tonic::Status`.

Requires the `grpc` feature.

## ADDED Requirements

### Requirement: gRPC server configuration

`GrpcServerConf` SHALL read all settings from environment variables (`GRPC_SERVER_*`),
apply them to a tonic `Server` via `builder()`, and support optional TLS (mTLS)
configuration when the `grpc-tls` feature is enabled. Without `grpc-tls`, enabling
TLS SHALL return a 500 error.

See `GrpcServerConf` in `src/grpc/confs.rs`.

#### Scenario: Server builder applies all settings
- WHEN `builder()` is called
- THEN a tonic `Server` SHALL be returned with load shedding, TCP nodelay, HTTP/2
  settings, connection limits, and timeouts applied from configuration

#### Scenario: TLS without grpc-tls feature
- WHEN `tls_enable` is true but `grpc-tls` feature is not active
- THEN `builder()` SHALL return a 500 error

### Requirement: gRPC client configuration

`GrpcClientConf` SHALL read all settings from environment variables (`GRPC_CLIENT_*`),
build a tonic `Endpoint` via `builder()`, and support optional TLS (including mTLS
and custom CA certificates) when the `grpc-tls` feature is enabled. TLS SHALL be
detected from both `use_tls` flag and `https://` endpoint detection.

See `GrpcClientConf` in `src/grpc/confs.rs`.

#### Scenario: TLS enabled via flag
- WHEN `use_tls` is true
- THEN `tls_enabled()` SHALL return true

#### Scenario: TLS enabled via https endpoint
- WHEN `endpoint` starts with `"https://"`
- THEN `tls_enabled()` SHALL return true even if `use_tls` is false

#### Scenario: mTLS requires both cert and key
- WHEN only one of `tls_certificate` or `tls_key` is configured
- THEN `tls_config()` SHALL return a 500 error

### Requirement: gRPC ↔ HTTP status mapping

The system SHALL provide bidirectional mapping between gRPC status codes and HTTP
status codes. The mapping SHALL cover all standard tonic `Code` variants.
`From<CornettiError> for tonic::Status` SHALL convert HTTP status codes to their
closest gRPC equivalents. `From<tonic::Status> for CornettiError` SHALL convert
gRPC status codes to HTTP codes.

See `src/grpc/errors.rs`.

#### Scenario: gRPC NotFound → HTTP 404
- WHEN a `tonic::Status` with code `NotFound` is converted
- THEN the resulting `CornettiError` SHALL have status 404

#### Scenario: HTTP 503 → gRPC Unavailable
- WHEN a `CornettiError` with status 503 is converted
- THEN the resulting `tonic::Status` SHALL have code `Unavailable`

#### Scenario: Tonic transport error → HTTP 500
- WHEN `From<tonic::transport::Error>` is invoked
- THEN a 500 `CornettiError` SHALL be produced with the error chain in detail
