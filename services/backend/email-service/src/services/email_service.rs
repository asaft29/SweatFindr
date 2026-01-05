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
            std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Email Service".to_string());

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

    pub async fn send_refund_approved_email(
        &self,
        to_email: &str,
        ticket_code: &str,
        event_name: &str,
    ) -> Result<()> {
        let html_body = html! {
            (DOCTYPE)
            html {
                head {
                    style {
                        r#"
                        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
                        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
                        .header { background-color: #28a745; color: white; padding: 20px; text-align: center; }
                        .content { background-color: #f9f9f9; padding: 20px; }
                        .ticket { font-size: 18px; font-weight: bold; color: #28a745; text-align: center; padding: 15px; background-color: white; border: 2px solid #28a745; margin: 20px 0; }
                        .footer { text-align: center; padding: 20px; color: #777; font-size: 12px; }
                        "#
                    }
                }
                body {
                    div.container {
                        div.header {
                            h1 { "✓ Refund Approved" }
                        }
                        div.content {
                            p { "Good news!" }
                            p { "Your refund request has been approved for the following:" }

                            div.ticket {
                                p { "Event: " (event_name) }
                                p { "Ticket: " (ticket_code) }
                            }

                            p { "The refund will be processed shortly. Thank you for your patience." }
                        }
                        div.footer {
                            p { "This is an automated message, please do not reply." }
                        }
                    }
                }
            }
        };

        self.send_html_email(to_email, "Refund Request Approved", html_body.into_string())
            .await
    }

    pub async fn send_refund_rejected_email(
        &self,
        to_email: &str,
        ticket_code: &str,
        event_name: &str,
        reason: &str,
    ) -> Result<()> {
        let html_body = html! {
            (DOCTYPE)
            html {
                head {
                    style {
                        r#"
                        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
                        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
                        .header { background-color: #dc3545; color: white; padding: 20px; text-align: center; }
                        .content { background-color: #f9f9f9; padding: 20px; }
                        .ticket { font-size: 18px; font-weight: bold; color: #333; text-align: center; padding: 15px; background-color: white; border: 2px solid #dc3545; margin: 20px 0; }
                        .reason { background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 15px; margin: 20px 0; }
                        .footer { text-align: center; padding: 20px; color: #777; font-size: 12px; }
                        "#
                    }
                }
                body {
                    div.container {
                        div.header {
                            h1 { "✗ Refund Request Declined" }
                        }
                        div.content {
                            p { "We're sorry to inform you that your refund request has been declined." }

                            div.ticket {
                                p { "Event: " (event_name) }
                                p { "Ticket: " (ticket_code) }
                            }

                            div.reason {
                                p { strong { "Reason: " } (reason) }
                            }

                            p { "If you have any questions, please contact our support team." }
                        }
                        div.footer {
                            p { "This is an automated message, please do not reply." }
                        }
                    }
                }
            }
        };

        self.send_html_email(to_email, "Refund Request Declined", html_body.into_string())
            .await
    }

    pub async fn send_password_reset_email(&self, to_email: &str, reset_code: &str) -> Result<()> {
        let html_body = html! {
            (DOCTYPE)
            html {
                head {
                    style {
                        r#"
                        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
                        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
                        .header { background-color: #4F46E5; color: white; padding: 20px; text-align: center; }
                        .content { background-color: #f9f9f9; padding: 20px; }
                        .code { font-size: 32px; font-weight: bold; color: #4F46E5; text-align: center; padding: 20px; background-color: white; border: 2px dashed #4F46E5; margin: 20px 0; letter-spacing: 5px; }
                        .footer { text-align: center; padding: 20px; color: #777; font-size: 12px; }
                        .warning { background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 10px; margin: 15px 0; }
                        "#
                    }
                }
                body {
                    div.container {
                        div.header {
                            h1 { "Password Reset" }
                        }
                        div.content {
                            p { "Hello," }
                            p { "We received a request to reset your password. Use the following code to reset your password:" }

                            div.code { (reset_code) }

                            p { "This code will expire in 15 minutes." }

                            div.warning {
                                p { strong { "Security Notice: " } "If you didn't request this password reset, please ignore this email. Your password will remain unchanged." }
                            }
                        }
                        div.footer {
                            p { "This is an automated message, please do not reply." }
                        }
                    }
                }
            }
        };

        self.send_html_email(to_email, "Password Reset Code", html_body.into_string())
            .await
    }

    async fn send_html_email(
        &self,
        to_email: &str,
        subject: &str,
        body_string: String,
    ) -> Result<()> {
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.from_name, self.from_email)
                    .parse()
                    .context("Failed to parse from email")?,
            )
            .to(to_email.parse().context("Failed to parse to email")?)
            .subject(subject)
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
