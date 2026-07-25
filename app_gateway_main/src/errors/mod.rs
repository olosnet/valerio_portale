pub mod gateway_errors;

cornetti_macros::export_errors_json! {
    include!("../cornetti-rust/cornetti/src/errors/body.rs")
    include!("../cornetti-rust/cornetti/src/errors/mongo.rs")
    include!("../cornetti-rust/cornetti/src/errors/redis.rs")
    include!("../cornetti-rust/cornetti/src/errors/mail.rs")
    include!("../cornetti-rust/cornetti/src/errors/grpc.rs")
    include!("../cornetti-rust/cornetti/src/errors/sqlx.rs")
    include!("../cornetti-rust/cornetti/src/errors/auth.rs")
    include!("../cornetti-rust/cornetti/src/errors/auth_apikey.rs")
    include!("../cornetti-rust/cornetti/src/errors/filemanager.rs")
    include!("../cornetti-rust/cornetti/src/errors/gmail.rs")
    include!("../cornetti-rust/cornetti/src/errors/templates.rs")
    include!("src/errors/gateway_dsl.rs")
}
