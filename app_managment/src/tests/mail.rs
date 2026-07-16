use clap::{Arg, Command};
use cornetti::{
    core::models::{CornettiError, CornettiGenericResponse},
    mail::smtp::{confs::SmtpMailConf, services::SendSmtpMailService},
    templates::{confs::TemplatesConf, services::TemplatesService},
};
use lettre::message::header::ContentType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TestMailSendBody {
    pub from: Option<String>,
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct TestsMailService {
    mail_service: SendSmtpMailService,
    template_service: TemplatesService,
}

impl TestsMailService {
    pub fn new(mail_conf: &SmtpMailConf, templates_conf: &TemplatesConf) -> Self {
        TestsMailService {
            mail_service: SendSmtpMailService::new(mail_conf.clone()),
            template_service: TemplatesService::new(templates_conf.clone()),
        }
    }

    pub async fn send_test_email(
        &self,
        data: &TestMailSendBody,
    ) -> Result<CornettiGenericResponse, CornettiError> {
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

async fn cmd_send_mail(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let from = args.get_one::<String>("from").cloned();
    let to = args.get_one::<String>("to").expect("required").clone();
    let subject = args.get_one::<String>("subject").expect("required").clone();
    let body = args.get_one::<String>("body").expect("required").clone();

    let mail_conf = SmtpMailConf::from_env();
    let templates_conf = TemplatesConf::from_env();
    let service = TestsMailService::new(&mail_conf, &templates_conf);

    let data = TestMailSendBody {
        from,
        to,
        subject,
        body,
    };

    match service.send_test_email(&data).await {
        Ok(response) => {
            println!("{}", response.message);
            Ok(())
        }
        Err(e) => Err(format!("Errore invio email: {}", e.detail).into()),
    }
}

fn send_mail_cmd() -> Command {
    Command::new("send-mail")
        .about("Invia una email di test per verificare la configurazione SMTP")
        .arg(
            Arg::new("from")
                .long("from")
                .value_name("FROM")
                .help("Mittente (default: conf SMTP)"),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .value_name("TO")
                .required(true)
                .help("Destinatario"),
        )
        .arg(
            Arg::new("subject")
                .long("subject")
                .value_name("SUBJECT")
                .required(true)
                .help("Oggetto dell'email"),
        )
        .arg(
            Arg::new("body")
                .long("body")
                .value_name("BODY")
                .required(true)
                .help("Corpo dell'email (testo semplice)"),
        )
}

pub fn tests_cmd() -> Command {
    Command::new("test")
        .about("Operazioni di test (email, etc.)")
        .subcommand_required(true)
        .subcommand(send_mail_cmd())
}

pub async fn dispatch(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand() {
        Some(("send-mail", a)) => cmd_send_mail(a).await,
        _ => Ok(()),
    }
}
