use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone)]
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_from: String,
    smtp_user: Option<String>,
    smtp_password: Option<String>,
    base_url: String,
}

impl EmailService {
    pub fn from_env() -> Self {
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1025u16);
        let smtp_from = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "noreply@soliloquio.local".to_string());
        let smtp_user = std::env::var("SMTP_USER").ok().filter(|s| !s.is_empty());
        let smtp_password = std::env::var("SMTP_PASSWORD").ok().filter(|s| !s.is_empty());
        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        EmailService {
            smtp_host,
            smtp_port,
            smtp_from,
            smtp_user,
            smtp_password,
            base_url,
        }
    }

    pub async fn send_password_reset(&self, to: &str, token: &str) -> Result<(), String> {
        let link = format!("{}/auth/reset_password?token={}", self.base_url, token);
        let body = format!(
            "<p>Click the link below to reset your password. This link expires in 1 hour.</p>\
             <p><a href=\"{link}\">{link}</a></p>\
             <p>If you did not request a password reset, ignore this email.</p>"
        );
        self.send(to, "Reset your password", &body).await
    }

    pub async fn send_email_verification(&self, to: &str, token: &str) -> Result<(), String> {
        let link = format!("{}/auth/verify_email?token={}", self.base_url, token);
        let body = format!(
            "<p>Click the link below to verify your email address. This link expires in 24 hours.</p>\
             <p><a href=\"{link}\">{link}</a></p>"
        );
        self.send(to, "Verify your email address", &body).await
    }

    async fn send(&self, to: &str, subject: &str, html_body: &str) -> Result<(), String> {
        let email = Message::builder()
            .from(
                self.smtp_from
                    .parse()
                    .map_err(|e| format!("invalid from address: {e}"))?,
            )
            .to(to
                .parse()
                .map_err(|e| format!("invalid to address: {e}"))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| format!("failed to build email: {e}"))?;

        let transport = self.build_transport();
        transport
            .send(email)
            .await
            .map_err(|e| format!("failed to send email: {e}"))?;

        Ok(())
    }

    fn build_transport(&self) -> AsyncSmtpTransport<Tokio1Executor> {
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.smtp_host)
            .port(self.smtp_port);
        if let (Some(user), Some(pass)) = (&self.smtp_user, &self.smtp_password) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        builder.build()
    }
}
