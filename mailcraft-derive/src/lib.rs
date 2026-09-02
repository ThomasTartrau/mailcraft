use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

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
#[proc_macro_derive(EmailTemplate, attributes(email_template))]
pub fn derive_email_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(name, "EmailTemplate can only be derived for enums")
            .to_compile_error()
            .into();
    };

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
        let subject_key = format!("{slug}/subject");
        let html_key = format!("{slug}/body.html");
        let txt_key = format!("{slug}/body.txt");
        let subject_path = format!("templates/{slug}/subject.hbs");
        let html_path = format!("templates/{slug}/body.html.hbs");
        let txt_path = format!("templates/{slug}/body.txt.hbs");

        slug_arms.push(quote! { Self::#variant_ident => #slug });
        if is_campaign {
            kind_arms
                .push(quote! { Self::#variant_ident => mailcraft::EmailTemplateKind::Campaign });
        }
        all_idents.push(quote! { Self::#variant_ident });
        register_stmts.push(quote! {
            handlebars.register_template_string(#subject_key, include_str!(#subject_path))?;
            handlebars.register_template_string(#html_key, include_str!(#html_path))?;
            handlebars.register_template_string(#txt_key, include_str!(#txt_path))?;
        });
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
