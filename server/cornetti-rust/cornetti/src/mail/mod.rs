use lettre::message::header::ContentType;

pub struct EmailAttachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: ContentType,
}

pub mod confs;
#[cfg(feature = "mail-gmail")]
pub mod gmail;
pub mod services;
pub mod smtp;
