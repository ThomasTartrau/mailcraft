<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/logo/mailcraft-logo-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/logo/mailcraft-logo-light.svg">
  <img alt="mailcraft" src="assets/logo/mailcraft-banner.png" width="560">
</picture>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

**Compile-time email template engine for Rust. Missing template? Build error, not a runtime crash.**

*Derive macro &bull; Handlebars &bull; HTML + text &bull; Dark mode &bull; Outlook-compatible &bull; Transport-agnostic*

[Quick Start](#quick-start) &bull;
[Architecture](#architecture) &bull;
[Features](#features) &bull;
[Starter Templates](#starter-templates) &bull;
[Transport](#transport)

</div>

---

## What is mailcraft?

Most email libraries handle *sending*. Mailcraft handles what comes before: defining, organizing and rendering email templates with compile-time safety.

A `#[derive(EmailTemplate)]` macro turns an enum into a template registry. Each variant maps to Handlebars files on disk (`subject.hbs`, `body.html.hbs`, `body.txt.hbs`). The derive generates `include_str!` calls at compile time -- a missing template file is a build error, not a 3 AM production crash.

Mailcraft is **transport-agnostic**. It renders emails; you send them with whatever you want ([lettre](https://crates.io/crates/lettre), [missive](https://crates.io/crates/missive), an HTTP API, a test double).

---

## Architecture

| Crate | Role |
|---|---|
| `mailcraft` | Template engine, email builder, helpers, staging redirect, `EmailSender` trait |
| `mailcraft-derive` | `#[derive(EmailTemplate)]` proc-macro (re-exported by `mailcraft`) |

How it works at build time:

```text
#[derive(EmailTemplate)]
        |
        v
  For each variant, generate:
    include_str!("templates/<slug>/subject.hbs")     --+
    include_str!("templates/<slug>/body.html.hbs")     |--> missing file = compile error
    include_str!("templates/<slug>/body.txt.hbs")    --+
        |
        v
  register_all() loads them into Handlebars at startup
        |
        v
  render_event("slug", &data) --> RenderedEmail { subject, html, text }
```

---

## Quick Start

```bash
cargo add mailcraft serde serde_json
```

### 1. Define your templates

```rust
use mailcraft::EmailTemplate;

#[derive(EmailTemplate, Copy, Clone, Debug)]
enum MyEmail {
    Welcome,
    PasswordReset,
    #[email_template(path = "campaigns/weekly", kind = "campaign")]
    WeeklyDigest,
}
```

### 2. Create the template files

Each variant maps to a directory under `templates/`. The slug is the variant name in `snake_case`, overridable with `#[email_template(path = "...")]`.

```text
src/
  templates/
    welcome/
      subject.hbs            "Welcome to {{app_name}}, {{name}}!"
      body.html.hbs          {{#> base_html}} ... {{/base_html}}
      body.txt.hbs           {{#> base_text}} ... {{/base_text}}
    password_reset/
      subject.hbs
      body.html.hbs
      body.txt.hbs
    campaigns/weekly/
      subject.hbs
      body.html.hbs
      body.txt.hbs
```

### 3. Render

```rust
use mailcraft::{TemplateEngine, Email, EmailAddress};
use serde_json::json;

let engine = TemplateEngine::builder()
    .register_templates::<MyEmail>()?
    .register_partial("base_html", mailcraft::BASE_HTML_TEMPLATE)?
    .register_partial("base_text", mailcraft::BASE_TEXT_TEMPLATE)?
    .build()?;

let rendered = engine.render_event("welcome", &json!({
    "name": "Alice",
    "app_name": "MyApp",
}))?;

let email = Email::builder()
    .to("alice@example.com")
    .from(EmailAddress::with_name("MyApp", "noreply@myapp.com"))
    .subject(rendered.subject)
    .html_body(rendered.html)
    .text_body(rendered.text)
    .build()?;

// Send with your preferred transport
your_sender.send(email).await?;
```

Run the full example:

```bash
cargo run --example simple -p mailcraft
```

---

## Features

### Compile-time safety

The derive macro generates `include_str!` for every template file. If you add a variant to your enum but forget to create the `.hbs` files, the build fails immediately.

```text
error: couldn't read src/templates/welcome/subject.hbs
  --> src/main.rs:4:10
   |
4  | #[derive(EmailTemplate)]
   |          ^^^^^^^^^^^^^
```

Handlebars strict mode is on by default -- a template referencing an undefined variable fails at render time, not silently.

### Built-in helpers

Handlebars provides comparison and logic helpers out of the box:

| Helper | Usage | Description |
|--------|-------|-------------|
| `eq` | `{{#if (eq status "active")}}` | Equality comparison |
| `ne` | `{{#if (ne role "admin")}}` | Inequality comparison |
| `and` | `{{#if (and a b c)}}` | Logical AND (variadic) |
| `or` | `{{#if (or a b c)}}` | Logical OR (variadic) |
| `not` | `{{#if (not done)}}` | Logical NOT |
| `gt`/`gte`/`lt`/`lte` | `{{#if (gt age 18)}}` | Numeric comparisons |
| `len` | `{{len items}}` | Collection length |

Mailcraft adds:

| Helper | Usage | Description |
|--------|-------|-------------|
| `default` | `{{default name "there"}}` | Fallback for falsy values (null, false, empty string, 0) |

Register your own via `TemplateEngineBuilder::register_helper()`.

### Event vs Campaign

Templates are either `Event` (transactional: password reset, invoice, etc.) or `Campaign` (marketing: newsletter, promo). The default is `Event`.

```rust
#[derive(EmailTemplate, Copy, Clone, Debug)]
enum MyEmail {
    PasswordReset,                                              // Event (default)
    #[email_template(kind = "campaign")]
    WeeklyDigest,                                               // Campaign
    #[email_template(path = "campaigns/promo", kind = "campaign")]
    PromoBlast,                                                 // Campaign with custom path
}
```

### Template metadata

The derive generates introspection methods on your enum:

```rust
for tmpl in MyEmail::all() {
    println!("{:?} -> slug: {}, kind: {:?}", tmpl, tmpl.slug(), tmpl.kind());
}
// Welcome -> slug: welcome, kind: Event
// PasswordReset -> slug: password_reset, kind: Event
// WeeklyDigest -> slug: campaigns/weekly, kind: Campaign
```

### Staging redirect

In non-production environments, redirect all emails to a staging inbox without losing traceability:

```rust
use mailcraft::redirect_email_for_staging;

let redirected = redirect_email_for_staging("alice@client.com", "staging@myapp.com");
// staging+a1b2c3d4e5f6...@myapp.com
```

Each original recipient maps to a **unique, deterministic** address via SHA-256 hashing in the `+tag`. The original recipient is never contacted; the staging inbox receives everything, filterable by tag.

---

## Starter Templates

Mailcraft ships generic base templates as constants, ready to use as Handlebars partials:

| Template | Features |
|----------|----------|
| `base.html.hbs` | Table-based layout, Outlook-compatible, dark mode CSS, responsive, logo/footer slots |
| `base.txt.hbs` | Plain text layout with header/footer |

Register them via `mailcraft::BASE_HTML_TEMPLATE` and `mailcraft::BASE_TEXT_TEMPLATE`, then wrap your content:

```handlebars
{{!-- body.html.hbs --}}
{{#> base_html}}
<h2 class="mc-title">Hello {{name}},</h2>
<p class="mc-text">Welcome aboard.</p>
{{/base_html}}
```

```handlebars
{{!-- body.txt.hbs --}}
{{#> base_text}}
Hello {{name}},

Welcome aboard.
{{/base_text}}
```

The starter templates use `mc-` prefixed CSS classes for dark mode. Override them, replace them, or write your own from scratch.

---

## Transport

Mailcraft does not send emails. Implement the `EmailSender` trait to plug in your transport:

```rust
use mailcraft::{Email, EmailSender};

struct LettreSmtp { /* ... */ }

impl EmailSender for LettreSmtp {
    type Error = lettre::transport::smtp::Error;

    async fn send(&self, email: Email) -> Result<(), Self::Error> {
        // Convert to lettre::Message and send
        todo!()
    }
}
```

Works with anything: [lettre](https://crates.io/crates/lettre) (SMTP), [missive](https://crates.io/crates/missive) (multi-provider), [reqwest](https://crates.io/crates/reqwest) (HTTP APIs like SendGrid, Resend), or a `Vec<Email>` for testing.

---

## API Reference

### TemplateEngine

| Method | Description |
|--------|-------------|
| `TemplateEngine::builder()` | Start building a configured engine |
| `.register_templates::<T>()?` | Register all templates from a `#[derive(EmailTemplate)]` enum |
| `.register_partial(name, src)` | Register a Handlebars partial (layouts, components) |
| `.register_helper(name, helper)` | Register a custom Handlebars helper |
| `.lenient()` | Disable strict mode (missing variables render as empty) |
| `.build()` | Build the engine |
| `engine.render_event(slug, &data)` | Render subject + HTML + text |
| `engine.render(name, &data)` | Render a single named template |

### Email

| Method | Description |
|--------|-------------|
| `Email::builder()` | Start building an email |
| `.to(email)` | Add a recipient (chainable) |
| `.from(address)` | Set the sender |
| `.reply_to(address)` | Set reply-to |
| `.subject(text)` | Set the subject |
| `.html_body(html)` | Set the HTML body |
| `.text_body(text)` | Set the text body |
| `.attachment(att)` | Add a file attachment |
| `.build()` | Build the email (returns `Result`) |

### Derive attributes

| Attribute | Effect |
|-----------|--------|
| `#[email_template(path = "...")]` | Override the template directory (default: `snake_case` of variant name) |
| `#[email_template(kind = "campaign")]` | Mark as campaign template (default: `event`) |

---

## Development

```bash
cargo check --workspace     # Type-check
cargo test --workspace      # Run all tests
cargo run --example simple  # Run the example
```

---

## License

MIT -- see [LICENSE](LICENSE).

---

<div align="center">

**[GitLab](https://gitlab.com/thomastartrau/mailcraft)**

[mailcraft](https://crates.io/crates/mailcraft) &bull;
[mailcraft-derive](https://crates.io/crates/mailcraft-derive)

</div>
