use mailcraft::{EmailTemplate, EmailTemplateKind, TemplateEngine};
use serde::Serialize;

#[derive(EmailTemplate, Copy, Clone, Debug)]
#[email_template(locales = ["fr", "en"], default_locale = "fr")]
enum LocalizedEmail {
    LocalizedWelcome,
}

#[derive(Serialize)]
struct LocalizedContext {
    name: String,
}

#[derive(EmailTemplate, Copy, Clone, Debug)]
enum TestEmail {
    TestWelcome,
    #[email_template(path = "promo_blast", kind = "campaign")]
    TestPromo,
}

#[derive(Serialize)]
struct WelcomeContext {
    name: String,
}

#[derive(Serialize)]
struct PromoContext {
    offer: String,
}

#[test]
fn derive_slug_and_kind() {
    assert_eq!(TestEmail::TestWelcome.slug(), "test_welcome");
    assert_eq!(TestEmail::TestWelcome.kind(), EmailTemplateKind::Event);
    assert_eq!(TestEmail::TestPromo.slug(), "promo_blast");
    assert_eq!(TestEmail::TestPromo.kind(), EmailTemplateKind::Campaign);
}

#[test]
fn derive_all_returns_all_variants() {
    assert_eq!(TestEmail::all().len(), 2);
}

#[test]
fn derive_register_and_render() {
    let engine = TemplateEngine::builder()
        .register_templates::<TestEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event(
            "test_welcome",
            &WelcomeContext {
                name: "Alice".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered.subject, "Welcome Alice");
    assert!(rendered.html.contains("Hello Alice"));
    assert!(rendered.text.contains("Hello Alice"));
}

#[test]
fn derive_html_escapes_text_does_not() {
    let engine = TemplateEngine::builder()
        .register_templates::<TestEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event(
            "test_welcome",
            &WelcomeContext {
                name: "<script>xss</script>".into(),
            },
        )
        .unwrap();

    assert!(rendered.html.contains("&lt;script&gt;"));
    assert!(!rendered.html.contains("<script>xss</script>"));

    assert!(rendered.text.contains("<script>xss</script>"));

    assert!(rendered.subject.contains("<script>xss</script>"));
}

#[test]
fn derive_campaign_variant_renders() {
    let engine = TemplateEngine::builder()
        .register_templates::<TestEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event(
            "promo_blast",
            &PromoContext {
                offer: "50% off".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered.subject, "Special offer: 50% off");
    assert!(rendered.html.contains("50% off"));
    assert!(rendered.text.contains("50% off"));
}

#[test]
fn localized_render_uses_locale_templates() {
    let engine = TemplateEngine::builder()
        .register_templates::<LocalizedEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event_localized(
            "localized_welcome",
            "en",
            &LocalizedContext {
                name: "Alice".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered.subject, "Welcome Alice");
    assert!(rendered.html.contains("Hello Alice"));
    assert!(rendered.text.contains("Hello Alice"));

    let rendered_fr = engine
        .render_event_localized(
            "localized_welcome",
            "fr",
            &LocalizedContext {
                name: "Alice".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered_fr.subject, "Bienvenue Alice");
    assert!(rendered_fr.html.contains("Bonjour Alice"));
    assert!(rendered_fr.text.contains("Bonjour Alice"));
}

#[test]
fn localized_render_falls_back_to_default() {
    let engine = TemplateEngine::builder()
        .register_templates::<LocalizedEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event_localized(
            "localized_welcome",
            "de",
            &LocalizedContext {
                name: "Alice".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered.subject, "Bienvenue Alice");
    assert!(rendered.html.contains("Bonjour Alice"));
    assert!(rendered.text.contains("Bonjour Alice"));
}

#[test]
fn localized_enum_render_event_uses_default_locale() {
    let engine = TemplateEngine::builder()
        .register_templates::<LocalizedEmail>()
        .unwrap()
        .build()
        .unwrap();

    let rendered = engine
        .render_event(
            "localized_welcome",
            &LocalizedContext {
                name: "Alice".into(),
            },
        )
        .unwrap();
    assert_eq!(rendered.subject, "Bienvenue Alice");
    assert!(rendered.html.contains("Bonjour Alice"));
}

#[test]
fn localized_enum_metadata() {
    assert_eq!(LocalizedEmail::locales(), &["fr", "en"]);
    assert_eq!(LocalizedEmail::default_locale(), "fr");
    assert_eq!(LocalizedEmail::LocalizedWelcome.slug(), "localized_welcome");
}
