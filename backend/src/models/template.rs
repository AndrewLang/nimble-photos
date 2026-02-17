use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::models::property_map::PropertyMap;

enum TemplatePart {
    Literal(String),
    Token(TemplateToken),
}

enum TemplateToken {
    FileName,
    FileStem,
    Extension,

    Year,
    Month,
    Day,

    DateFormat(String),

    HashFull,
    HashSlice { start: usize, len: usize },

    Camera,
    Rating,
}

impl TemplateToken {
    fn resolve<C: TemplateContext>(&self, context: &C) -> Result<String> {
        match self {
            TemplateToken::FileName => Ok(context
                .get_property::<String>("file_name")
                .map(|v| v.as_str())
                .unwrap_or("")
                .to_string()),

            TemplateToken::FileStem => {
                let name = context
                    .get_property::<String>("file_name")
                    .map(|v| v.as_str())
                    .unwrap_or("");
                Ok(Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string())
            }

            TemplateToken::Extension => {
                let name = context
                    .get_property::<String>("file_name")
                    .map(|v| v.as_str())
                    .unwrap_or("");
                Ok(Path::new(name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string())
            }

            TemplateToken::Year => {
                let d = context
                    .get_property::<DateTime<Utc>>("effective_date")
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing effective_date"))?;
                Ok(d.format("%Y").to_string())
            }

            TemplateToken::Month => {
                let d = context
                    .get_property::<DateTime<Utc>>("effective_date")
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing effective_date"))?;
                Ok(d.format("%m").to_string())
            }

            TemplateToken::Day => {
                let d = context
                    .get_property::<DateTime<Utc>>("effective_date")
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing effective_date"))?;
                Ok(d.format("%d").to_string())
            }

            TemplateToken::DateFormat(fmt) => {
                let d = context
                    .get_property::<DateTime<Utc>>("effective_date")
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing effective_date"))?;
                Ok(d.format(fmt).to_string())
            }

            TemplateToken::HashFull => Ok(context
                .get_property::<String>("hash")
                .map(String::as_str)
                .ok_or_else(|| anyhow!("Missing hash"))?
                .to_string()),

            TemplateToken::HashSlice { start, len } => {
                let h = context
                    .get_property::<String>("hash")
                    .map(String::as_str)
                    .ok_or_else(|| anyhow!("Missing hash"))?;
                Ok(h.chars().skip(*start).take(*len).collect())
            }

            TemplateToken::Camera => Ok(context
                .get_property::<String>("camera")
                .map(|v| v.as_str())
                .unwrap_or("")
                .to_string()),

            TemplateToken::Rating => Ok(context
                .get_property::<i32>("rating")
                .unwrap_or(&0)
                .to_string()),
        }
    }
}

pub struct TemplateTokenNames {}

impl TemplateTokenNames {
    pub const FILE_NAME: &'static str = "fileName";
    pub const FILE_STEM: &'static str = "fileStem";
    pub const EXTENSION: &'static str = "extension";
    pub const YEAR: &'static str = "year";
    pub const MONTH: &'static str = "month";
    pub const DAY: &'static str = "day";
    pub const DATE_FORMAT: &'static str = "date";
    pub const HASH: &'static str = "hash";
    pub const HASH_SLICE: &'static str = "hashSlice";
    pub const CAMERA: &'static str = "camera";
    pub const RATING: &'static str = "rating";
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn compile(template: impl Into<String>) -> Result<CompiledTemplate> {
        CompiledTemplate::new(template.into())
    }
}

pub struct CompiledTemplate {
    template: String,
    parts: Vec<TemplatePart>,
}

impl CompiledTemplate {
    fn new(template: String) -> Result<Self> {
        let mut instance = Self {
            template,
            parts: Vec::new(),
        };

        instance.parse()?;
        Ok(instance)
    }

    pub fn render<C: TemplateContext>(&self, context: &C) -> Result<String> {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                TemplatePart::Literal(l) => result.push_str(l),
                TemplatePart::Token(t) => {
                    let value = t.resolve(context)?;
                    result.push_str(&self.sanitize(&value));
                }
            }
        }

        Ok(self.normalize(&result))
    }

    pub fn requires_hash(&self) -> bool {
        self.parts.iter().any(|p| {
            matches!(
                p,
                TemplatePart::Token(TemplateToken::HashFull | TemplateToken::HashSlice { .. })
            )
        })
    }
}

impl CompiledTemplate {
    fn parse(&mut self) -> Result<()> {
        let mut chars = self.template.chars().peekable();
        let mut literal = String::new();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if !literal.is_empty() {
                    self.parts.push(TemplatePart::Literal(literal.clone()));
                    literal.clear();
                }

                let mut token_str = String::new();

                while let Some(next) = chars.next() {
                    if next == '}' {
                        break;
                    }
                    token_str.push(next);
                }

                let token = self.parse_token(&token_str)?;
                self.parts.push(TemplatePart::Token(token));
            } else {
                literal.push(ch);
            }
        }

        if !literal.is_empty() {
            self.parts.push(TemplatePart::Literal(literal));
        }

        Ok(())
    }

    fn parse_token(&self, input: &str) -> Result<TemplateToken> {
        match input {
            TemplateTokenNames::FILE_NAME => return Ok(TemplateToken::FileName),
            TemplateTokenNames::FILE_STEM => return Ok(TemplateToken::FileStem),
            TemplateTokenNames::EXTENSION => return Ok(TemplateToken::Extension),
            TemplateTokenNames::YEAR => return Ok(TemplateToken::Year),
            TemplateTokenNames::MONTH => return Ok(TemplateToken::Month),
            TemplateTokenNames::DAY => return Ok(TemplateToken::Day),
            TemplateTokenNames::HASH => return Ok(TemplateToken::HashFull),
            TemplateTokenNames::CAMERA => return Ok(TemplateToken::Camera),
            TemplateTokenNames::RATING => return Ok(TemplateToken::Rating),
            _ => {}
        }

        if input.starts_with("hash:") {
            let parts: Vec<&str> = input.split(':').collect();
            if parts.len() == 3 {
                let start = parts[1].parse()?;
                let len = parts[2].parse()?;
                return Ok(TemplateToken::HashSlice { start, len });
            }
        }

        if input.starts_with("date:") {
            let format = input.strip_prefix("date:").unwrap().to_string();
            return Ok(TemplateToken::DateFormat(format));
        }

        Err(anyhow!("Unknown token: {}", input))
    }

    fn sanitize(&self, input: &str) -> String {
        input
            .replace("/", "_")
            .replace("\\", "_")
            .replace("..", "_")
            .trim()
            .to_string()
    }

    fn normalize(&self, path: &str) -> String {
        path.split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    }
}

pub trait TemplateContext {
    fn get_property<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T>;
}

pub struct PropertyMapTemplateContext {
    properties: PropertyMap,
}

impl PropertyMapTemplateContext {
    pub fn new(properties: PropertyMap) -> Self {
        Self { properties }
    }
}

impl TemplateContext for PropertyMapTemplateContext {
    fn get_property<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.properties.get_by_alias(key)
    }
}
