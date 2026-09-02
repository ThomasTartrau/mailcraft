use thiserror::Error;

/// Errors that can occur when building an email.
#[derive(Debug, Error)]
pub enum EmailBuildError {
    #[error("missing required field: to (at least one recipient required)")]
    MissingTo,

    #[error("missing required field: from")]
    MissingFrom,

    #[error("missing required field: subject")]
    MissingSubject,

    #[error("missing required field: html_body")]
    MissingHtmlBody,

    #[error("missing required field: text_body")]
    MissingTextBody,
}

/// Email address with optional display name.
#[derive(Debug, Clone)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub email: String,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            name: None,
            email: email.into(),
        }
    }

    pub fn with_name(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: email.into(),
        }
    }
}

/// File attachment for an email.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl Attachment {
    pub fn pdf(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            content_type: "application/pdf".to_string(),
            data,
        }
    }

    pub fn new(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            data,
        }
    }
}

/// Email ready to be sent.
#[derive(Debug, Clone)]
pub struct Email {
    pub to: Vec<String>,
    pub from: EmailAddress,
    pub reply_to: Option<EmailAddress>,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub attachments: Vec<Attachment>,
}

impl Email {
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

/// Builder for constructing emails.
#[derive(Debug, Default)]
pub struct EmailBuilder {
    to: Vec<String>,
    from: Option<EmailAddress>,
    reply_to: Option<EmailAddress>,
    subject: Option<String>,
    html_body: Option<String>,
    text_body: Option<String>,
    attachments: Vec<Attachment>,
}

impl EmailBuilder {
    pub fn to(mut self, email: impl Into<String>) -> Self {
        self.to.push(email.into());
        self
    }

    pub fn from(mut self, address: EmailAddress) -> Self {
        self.from = Some(address);
        self
    }

    pub fn reply_to(mut self, address: EmailAddress) -> Self {
        self.reply_to = Some(address);
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }

    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn build(self) -> Result<Email, EmailBuildError> {
        if self.to.is_empty() {
            return Err(EmailBuildError::MissingTo);
        }

        Ok(Email {
            to: self.to,
            from: self.from.ok_or(EmailBuildError::MissingFrom)?,
            reply_to: self.reply_to,
            subject: self.subject.ok_or(EmailBuildError::MissingSubject)?,
            html_body: self.html_body.ok_or(EmailBuildError::MissingHtmlBody)?,
            text_body: self.text_body.ok_or(EmailBuildError::MissingTextBody)?,
            attachments: self.attachments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_simple_email() {
        let email = Email::builder()
            .to("recipient@example.com")
            .from(EmailAddress::with_name("Sender", "sender@example.com"))
            .subject("Test Subject")
            .html_body("<p>Hello</p>")
            .text_body("Hello")
            .build()
            .unwrap();

        assert_eq!(email.to, vec!["recipient@example.com"]);
        assert_eq!(email.subject, "Test Subject");
    }

    #[test]
    fn build_email_missing_to() {
        let result = Email::builder()
            .from(EmailAddress::new("sender@example.com"))
            .subject("Test")
            .html_body("<p>Hello</p>")
            .text_body("Hello")
            .build();

        assert!(matches!(result, Err(EmailBuildError::MissingTo)));
    }

    #[test]
    fn build_email_with_attachment() {
        let email = Email::builder()
            .to("recipient@example.com")
            .from(EmailAddress::new("sender@example.com"))
            .subject("Invoice")
            .html_body("<p>See attached</p>")
            .text_body("See attached")
            .attachment(Attachment::pdf("invoice.pdf", vec![1, 2, 3]))
            .build()
            .unwrap();

        assert_eq!(email.attachments.len(), 1);
        assert_eq!(email.attachments[0].filename, "invoice.pdf");
    }
}
