use handlebars::{Context, Handlebars, Helper, HelperDef, JsonTruthy, RenderContext, ScopedJson};

pub struct DefaultHelper;

impl HelperDef for DefaultHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, handlebars::RenderError> {
        let value = h.param(0).map(|v| v.value());
        let fallback = h.param(1).map(|v| v.value());

        let output = match value {
            Some(v) if v.is_truthy(false) => v.clone(),
            _ => match fallback {
                Some(f) => f.clone(),
                None => return Ok(ScopedJson::Missing),
            },
        };

        Ok(ScopedJson::Derived(output))
    }
}
