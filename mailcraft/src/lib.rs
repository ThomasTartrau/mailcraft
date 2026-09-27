//! Compile-time email template engine for Rust.
//!
//! Mailcraft provides a type-safe email template system where missing templates
//! fail the build instead of crashing at runtime. It combines a derive macro
//! with Handlebars to give you compile-time verified email templates.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use mailcraft::{EmailTemplate, EmailTemplateKind, TemplateEngine};
//! use serde_json::json;
//!
//! #[derive(EmailTemplate, Copy, Clone, Debug)]
//! enum MyEmail {
//!     Welcome,
//!     PasswordReset,
//!     #[email_template(path = "campaigns/promo", kind = "campaign")]
//!     PromoBlast,
//! }
//!
//! let engine = TemplateEngine::builder()
//!     .register_templates::<MyEmail>()?
//!     .register_partial("base_html", mailcraft::BASE_HTML_TEMPLATE)?
//!     .register_partial("base_text", mailcraft::BASE_TEXT_TEMPLATE)?
//!     .build()?;
//!
//! let rendered = engine.render_event("welcome", &json!({
//!     "name": "Alice",
//!     "app_name": "MyApp",
//! }))?;
//!
//! println!("Subject: {}", rendered.subject);
//! println!("HTML: {}", rendered.html);
//! println!("Text: {}", rendered.text);
//! ```
#![doc(
    html_logo_url = "https://gitlab.com/ThomasTartrau/mailcraft/-/raw/main/assets/logo/mailcraft-symbol.svg",
    html_favicon_url = "https://gitlab.com/ThomasTartrau/mailcraft/-/raw/main/assets/logo/mailcraft-favicon.ico"
)]

mod email;
mod engine;
mod helpers;
mod staging;

pub use email::{Attachment, Email, EmailAddress, EmailBuildError, EmailBuilder};
pub use engine::{
    RegisterTemplates, RenderedEmail, TemplateEngine, TemplateEngineBuilder, TemplateError,
};
pub use handlebars;
pub use mailcraft_derive::EmailTemplate;
pub use staging::redirect_email_for_staging;

use std::future::Future;

pub const BASE_HTML_TEMPLATE: &str = include_str!("../templates/base.html.hbs");
pub const BASE_TEXT_TEMPLATE: &str = include_str!("../templates/base.txt.hbs");

/// Distinguishes transactional event templates from marketing campaign templates.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EmailTemplateKind {
    Event,
    Campaign,
}

/// Trait for sending emails. Implement this for your transport layer
/// (SMTP via lettre, HTTP API via reqwest, etc.).
pub trait EmailSender: Send + Sync {
    type Error: std::error::Error + Send + 'static;

    fn send(&self, email: Email) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
