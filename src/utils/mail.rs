use anyhow::Result;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

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
        let client = SmtpTransport::relay("smtp.163.com")?
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
        let email = Message::builder()
            .from(self.from.parse()?)
            .reply_to(self.reply_to.parse()?)
            .to(self.to.parse()?)
            .subject(subject.as_ref())
            .header(ContentType::TEXT_HTML)
            .body(body.into())?;

        self.client.send(&email)?;

        Ok(())
    }
}
