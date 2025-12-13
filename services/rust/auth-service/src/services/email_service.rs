use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncTransport;
use lettre::{AsyncSmtpTransport, Message, Tokio1Executor};
use maud::{html, DOCTYPE};
use rand::Rng;

pub struct EmailService {
    from_email: String,
    from_name: String,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn new() -> Result<Self> {
        let smtp_username =
            std::env::var("SMTP_USERNAME").context("SMTP_USERNAME not set in environment")?;
        let smtp_password =
            std::env::var("SMTP_PASSWORD").context("SMTP_PASSWORD not set in environment")?;
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let from_email = std::env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());
        let from_name =
            std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Auth Service".to_string());

        let creds = Credentials::new(smtp_username, smtp_password);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
            .context("Failed to build SMTP relay")?
            .credentials(creds)
            .build();

        Ok(Self {
            from_email,
            from_name,
            mailer,
        })
    }

    pub fn generate_verification_code() -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1000000))
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_code: &str,
    ) -> Result<()> {
        let html_body = html! {
            (DOCTYPE)
            html {
                head {
                    style {
                        r#"
                        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
                        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
                        .header { background-color: #FFA500; color: white; padding: 20px; text-align: center; }
                        .content { background-color: #f9f9f9; padding: 20px; }
                        .code { font-size: 32px; font-weight: bold; color: #FFA500; text-align: center; padding: 20px; background-color: white; border: 2px dashed #FFA500; margin: 20px 0; letter-spacing: 5px; }
                        .footer { text-align: center; padding: 20px; color: #777; font-size: 12px; }
                        "#
                    }
                }
                body {
                    div.container {
                        div.header {
                            h1 { "Email Verification" }
                        }
                        div.content {
                            p { "Hello," }
                            p { "Thank you for registering! Please use the following verification code to complete your registration:" }

                            div.code { (verification_code) }

                            p { "This code will expire in 15 minutes." }
                            p { "If you didn't request this verification code, please ignore this email." }
                        }
                        div.footer {
                            p { "This is an automated message, please do not reply." }
                        }
                    }
                }
            }
        };

        let body_string = html_body.into_string();

        let email = Message::builder()
            .from(
                format!("{} <{}>", self.from_name, self.from_email)
                    .parse()
                    .context("Failed to parse from email")?,
            )
            .to(to_email.parse().context("Failed to parse to email")?)
            .subject("Email Verification Code")
            .header(ContentType::TEXT_HTML)
            .body(body_string)
            .context("Failed to build email message")?;

        self.mailer
            .send(email)
            .await
            .context("Failed to send email")?;

        Ok(())
    }
}
