use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::{SmtpTransport, Transport};

#[derive(Clone)]
pub struct Mail {
    client: SmtpTransport,
    from: String,
    reply_to: String,
    to: String,
}

impl Mail {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        from: impl Into<String>,
        reply_to: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<Mail> {
        let creds = Credentials::new(username.into(), password.into());
        let client = SmtpTransport::relay("smtp.163.com")
            .context("connect smtp server `smtp.163.com` failed")?
            .credentials(creds)
            .build();

        Ok(Self {
            client,
            from: from.into(),
            reply_to: reply_to.into(),
            to: to.into(),
        })
    }

    pub fn send(&self, subject: impl AsRef<str>, body: impl Into<String>) -> Result<()> {
        let subject = subject.as_ref();
        let email = Message::builder()
            .from(
                self.from
                    .parse()
                    .with_context(|| format!("parse mailbox `{}` failed", self.from))?,
            )
            .reply_to(
                self.reply_to
                    .parse()
                    .with_context(|| format!("parse mailbox `{}` failed", self.reply_to))?,
            )
            .to(self
                .to
                .parse()
                .with_context(|| format!("parse mailbox `{}` failed", self.to))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.into())
            .with_context(|| format!("init email `{}` failed", subject))?;

        self.client.send(&email).context("send mail failed")?;

        Ok(())
    }
}
