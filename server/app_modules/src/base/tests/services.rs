use crate::base::tests::models::TestMailSendBody;
use cornetti::{
    core::models::CornettiGenericResponse, mail::smtp::services::SendSmtpMailService,
    templates::services::TemplatesService,
};
use lettre::message::header::ContentType;

pub struct TestsService {
    mail_service: SendSmtpMailService,
    template_service: TemplatesService,
}

impl TestsService {
    pub fn new(
        mail_conf: &cornetti::mail::smtp::confs::SmtpMailConf,
        templates_conf: &cornetti::templates::confs::TemplatesConf,
    ) -> Self {
        let mail_service = SendSmtpMailService::new(mail_conf.clone());
        let template_service = TemplatesService::new(templates_conf.clone());

        TestsService {
            mail_service,
            template_service,
        }
    }

    pub async fn send_test_email(
        &self,
        data: &TestMailSendBody,
    ) -> Result<CornettiGenericResponse, cornetti::core::models::CornettiError> {
        let mut context = std::collections::HashMap::new();
        context.insert("body".to_string(), minijinja::Value::from(&data.body));

        let rendered_body = self.template_service.render("test-mail.jinja", &context)?;

        self.mail_service
            .send_email(
                data.from.as_deref(),
                &data.to,
                &data.subject,
                None,
                rendered_body,
                vec![],
                ContentType::TEXT_HTML,
            )
            .await?;

        Ok(CornettiGenericResponse::new(
            "Test email sent successfully.".to_string(),
        ))
    }
}
