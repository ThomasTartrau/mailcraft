use mailcraft::{Email, EmailAddress, EmailTemplate, TemplateEngine};
use serde::Serialize;

#[derive(EmailTemplate, Copy, Clone, Debug)]
enum AppEmail {
    Welcome,
    PasswordReset,
}

#[derive(Serialize)]
struct WelcomeContext {
    name: String,
    app_name: String,
    profile_url: String,
    footer_text: String,
}

#[derive(Serialize)]
struct PasswordResetContext {
    name: String,
    app_name: String,
    reset_url: String,
    expiry_minutes: u32,
    footer_text: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TemplateEngine::builder()
        .register_templates::<AppEmail>()?
        .register_partial("base_html", mailcraft::BASE_HTML_TEMPLATE)?
        .register_partial("base_text", mailcraft::BASE_TEXT_TEMPLATE)?
        .build()?;

    let welcome = engine.render_event(
        "welcome",
        &WelcomeContext {
            name: "Alice".to_string(),
            app_name: "MyApp".to_string(),
            profile_url: "https://myapp.com/profile".to_string(),
            footer_text: "MyApp Inc. - 123 Main St".to_string(),
        },
    )?;

    println!("=== Welcome Email ===");
    println!("Subject: {}", welcome.subject);
    println!();
    println!("--- HTML ---");
    println!("{}", &welcome.html[..200.min(welcome.html.len())]);
    println!("...");
    println!();
    println!("--- Text ---");
    println!("{}", welcome.text);

    let reset = engine.render_event(
        "password_reset",
        &PasswordResetContext {
            name: "Alice".to_string(),
            app_name: "MyApp".to_string(),
            reset_url: "https://myapp.com/reset?token=abc123".to_string(),
            expiry_minutes: 30,
            footer_text: "MyApp Inc. - 123 Main St".to_string(),
        },
    )?;

    println!();
    println!("=== Password Reset Email ===");
    println!("Subject: {}", reset.subject);
    println!();
    println!("--- Text ---");
    println!("{}", reset.text);

    let email = Email::builder()
        .to("alice@example.com")
        .from(EmailAddress::with_name("MyApp", "noreply@myapp.com"))
        .reply_to(EmailAddress::new("support@myapp.com"))
        .subject(welcome.subject)
        .html_body(welcome.html)
        .text_body(welcome.text)
        .build()?;

    println!();
    println!("=== Built Email ===");
    println!("To: {:?}", email.to);
    println!(
        "From: {} <{}>",
        email.from.name.as_deref().unwrap_or(""),
        email.from.email
    );
    println!("Subject: {}", email.subject);

    println!();
    println!("=== Template Registry ===");
    for tmpl in AppEmail::all() {
        println!(
            "  {:?} -> slug: {:?}, kind: {:?}",
            tmpl,
            tmpl.slug(),
            tmpl.kind()
        );
    }

    let redirected = mailcraft::redirect_email_for_staging("alice@client.com", "staging@myapp.com");
    println!();
    println!("=== Staging Redirect ===");
    println!("alice@client.com -> {redirected}");

    Ok(())
}
