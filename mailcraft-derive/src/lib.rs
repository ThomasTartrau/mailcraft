#![doc(
    html_logo_url = "https://gitlab.com/ThomasTartrau/mailcraft/-/raw/main/assets/logo/mailcraft-symbol.svg",
    html_favicon_url = "https://gitlab.com/ThomasTartrau/mailcraft/-/raw/main/assets/logo/mailcraft-favicon.ico"
)]

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, ExprArray, ExprLit, Fields, Lit, LitStr, parse_macro_input};

/// Derive macro for email template enums.
///
/// For each unit variant:
/// - the slug defaults to the variant's name in `snake_case`
/// - the `kind` defaults to `EmailTemplateKind::Event`
///
/// Both can be overridden with `#[email_template(path = "...", kind = "campaign")]`.
///
/// Generates an inherent `impl` exposing:
/// - `slug(self) -> &'static str`
/// - `kind(self) -> mailcraft::EmailTemplateKind`
/// - `all() -> &'static [Self]`
/// - `register_all(handlebars: &mut mailcraft::handlebars::Handlebars<'_>) -> Result<(), mailcraft::handlebars::TemplateError>`
///   which inlines `include_str!` calls for every `templates/<slug>/{subject,body.html,body.txt}.hbs`,
///   so a missing file fails the build.
///
/// # Localization
///
/// Add `#[email_template(locales = ["fr", "en"], default_locale = "fr")]` on the enum
/// to enable i18n. Templates are then expected at `templates/<slug>/<locale>/` and
/// the macro generates `locales()` and `default_locale()` methods.
#[proc_macro_derive(EmailTemplate, attributes(email_template))]
pub fn derive_email_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(name, "EmailTemplate can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let mut locales: Option<Vec<String>> = None;
    let mut default_locale: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("email_template") {
            continue;
        }
        let parse_result = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("locales") {
                let value = meta.value()?;
                let array: ExprArray = value.parse()?;
                let mut parsed_locales = Vec::new();
                for elem in &array.elems {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = elem
                    {
                        parsed_locales.push(s.value());
                    } else {
                        return Err(meta.error("locales must be string literals"));
                    }
                }
                locales = Some(parsed_locales);
            } else if meta.path.is_ident("default_locale") {
                let lit: LitStr = meta.value()?.parse()?;
                default_locale = Some(lit.value());
            } else {
                return Err(
                    meta.error("unknown enum attribute, expected `locales` or `default_locale`")
                );
            }
            Ok(())
        });
        if let Err(err) = parse_result {
            return err.to_compile_error().into();
        }
    }

    if locales.is_some() && default_locale.is_none() {
        return syn::Error::new_spanned(
            name,
            "`default_locale` is required when `locales` is specified",
        )
        .to_compile_error()
        .into();
    }

    if let (Some(locs), Some(def)) = (&locales, &default_locale)
        && !locs.contains(def)
    {
        return syn::Error::new_spanned(
            name,
            format!("`default_locale` \"{}\" is not in the `locales` list", def),
        )
        .to_compile_error()
        .into();
    }

    let mut slug_arms = Vec::new();
    let mut kind_arms = Vec::new();
    let mut all_idents = Vec::new();
    let mut register_stmts = Vec::new();

    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                &variant.ident,
                "EmailTemplate variants must be unit variants (no data)",
            )
            .to_compile_error()
            .into();
        }

        let variant_ident = &variant.ident;
        let mut slug_override: Option<String> = None;
        let mut is_campaign = false;

        for attr in &variant.attrs {
            if !attr.path().is_ident("email_template") {
                continue;
            }
            let parse_result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    let lit: LitStr = meta.value()?.parse()?;
                    slug_override = Some(lit.value());
                } else if meta.path.is_ident("kind") {
                    let lit: LitStr = meta.value()?.parse()?;
                    match lit.value().as_str() {
                        "campaign" => is_campaign = true,
                        "event" => is_campaign = false,
                        other => {
                            return Err(meta.error(format!(
                                "unknown kind `{other}`, expected `event` or `campaign`"
                            )));
                        }
                    }
                } else {
                    return Err(meta.error("unknown attribute, expected `path` or `kind`"));
                }
                Ok(())
            });
            if let Err(err) = parse_result {
                return err.to_compile_error().into();
            }
        }

        let slug = slug_override.unwrap_or_else(|| variant_ident.to_string().to_snake_case());

        let emit_register = |key_prefix: &str, path_prefix: &str| {
            let subject_key = format!("{key_prefix}/subject");
            let html_key = format!("{key_prefix}/body.html");
            let txt_key = format!("{key_prefix}/body.txt");
            let subject_path = format!("{path_prefix}/subject.hbs");
            let html_path = format!("{path_prefix}/body.html.hbs");
            let txt_path = format!("{path_prefix}/body.txt.hbs");
            quote! {
                handlebars.register_template_string(#subject_key, include_str!(#subject_path))?;
                handlebars.register_template_string(#html_key, include_str!(#html_path))?;
                handlebars.register_template_string(#txt_key, include_str!(#txt_path))?;
            }
        };

        if let Some(ref locs) = locales {
            let default_loc = default_locale.as_ref().unwrap();
            for locale in locs {
                let tpl_dir = format!("templates/{slug}/{locale}");
                register_stmts.push(emit_register(&format!("{slug}/{locale}"), &tpl_dir));
                if locale == default_loc {
                    register_stmts.push(emit_register(&slug, &tpl_dir));
                }
            }
        } else {
            register_stmts.push(emit_register(&slug, &format!("templates/{slug}")));
        }

        slug_arms.push(quote! { Self::#variant_ident => #slug });
        if is_campaign {
            kind_arms
                .push(quote! { Self::#variant_ident => mailcraft::EmailTemplateKind::Campaign });
        }
        all_idents.push(quote! { Self::#variant_ident });
    }

    let kind_match_body = if kind_arms.is_empty() {
        quote! { mailcraft::EmailTemplateKind::Event }
    } else {
        quote! {
            match self {
                #(#kind_arms,)*
                _ => mailcraft::EmailTemplateKind::Event,
            }
        }
    };

    let locale_methods = if let Some(ref locs) = locales {
        let default_loc = default_locale.as_ref().unwrap();
        let locale_strs: Vec<&str> = locs.iter().map(|s| s.as_str()).collect();
        quote! {
            pub const fn locales() -> &'static [&'static str] {
                &[ #(#locale_strs),* ]
            }

            pub const fn default_locale() -> &'static str {
                #default_loc
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl #name {
            pub const fn slug(self) -> &'static str {
                match self {
                    #(#slug_arms,)*
                }
            }

            pub const fn kind(self) -> mailcraft::EmailTemplateKind {
                #kind_match_body
            }

            pub const fn all() -> &'static [Self] {
                &[ #(#all_idents),* ]
            }

            #locale_methods

            pub fn register_all(
                handlebars: &mut mailcraft::handlebars::Handlebars<'_>,
            ) -> Result<(), mailcraft::handlebars::TemplateError> {
                #(#register_stmts)*
                Ok(())
            }
        }

        impl mailcraft::RegisterTemplates for #name {
            fn register(
                handlebars: &mut mailcraft::handlebars::Handlebars<'_>,
            ) -> Result<(), mailcraft::handlebars::TemplateError> {
                Self::register_all(handlebars)
            }
        }
    };

    TokenStream::from(expanded)
}
