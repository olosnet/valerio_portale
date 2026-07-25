#[cfg(feature = "grpc")]
grpc(500, log_level: Error): {
    *grpc_transport_error(500, log_level: Error) => "gRPC transport error",
    grpc_status_error(500, log_level: Error)     => "gRPC status error",
    *grpc_config_error(500, log_level: Error)    => "gRPC configuration error",
},
