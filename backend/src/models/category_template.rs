use anyhow::Result;

use crate::models::template::{CompiledTemplate, TemplateContext, TemplateEngine};

pub struct CategoryTemplateParser {
    template: String,
    compiled: CompiledTemplate,
}

impl CategoryTemplateParser {
    pub fn new(template: impl Into<String>) -> Result<Self> {
        let template = Self::normalize_template(template.into());
        let compiled = TemplateEngine::compile(template.clone())?;
        Ok(Self { template, compiled })
    }

    pub fn template(&self) -> &str {
        &self.template
    }

    pub fn requires_hash(&self) -> bool {
        self.compiled.requires_hash()
    }

    pub fn render<C: TemplateContext>(&self, context: &C) -> Result<String> {
        self.compiled.render(context)
    }

    fn normalize_template(template: String) -> String {
        let raw = template.trim();
        if raw.is_empty() {
            return "{year}/{date:%Y-%m-%d}/{fileName}".to_string();
        }
        if raw.eq_ignore_ascii_case("hash") {
            return "{hash:0:2}/{hash:2:2}/{fileName}".to_string();
        }
        if raw.eq_ignore_ascii_case("date") {
            return "{date:%Y-%m-%d}/{fileName}".to_string();
        }
        raw.to_string()
    }
}
