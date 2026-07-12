use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]

pub struct TestMailSendBody {
    pub from: Option<String>,
    pub to: String,
    pub subject: String,
    pub body: String,
}