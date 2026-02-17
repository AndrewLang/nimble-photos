use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::fs;

use crate::models::property_map::PropertyMap;
use crate::models::template::PropertyMapTemplateContext;
use crate::models::template::TemplateEngine;
use crate::services::image_process_constants::ImageProcessKeys;

#[derive(Debug)]
pub struct CategorizeRequest<'a> {
    source_file: &'a Path,
    properties: &'a PropertyMap,
}

impl<'a> CategorizeRequest<'a> {
    pub fn new(source_file: &'a Path, properties: &'a PropertyMap) -> Self {
        Self {
            source_file,
            properties,
        }
    }

    pub fn source_file(&self) -> &Path {
        self.source_file
    }

    pub fn properties(&self) -> &'a PropertyMap {
        self.properties
    }
}

#[derive(Debug, Clone)]
pub struct CategorizeResult {
    pub final_path: PathBuf,
    pub hash: Option<String>,
}

pub trait ImageCategorizer: Send + Sync {
    fn name(&self) -> &'static str;
    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult>;
}

pub struct TemplateCategorizer {
    template: String,
}

impl TemplateCategorizer {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    fn effective_template(&self) -> &str {
        let raw = self.template.trim();
        if raw.is_empty() {
            return "{year}/{date:%Y-%m-%d}/{fileName}";
        }
        if raw.eq_ignore_ascii_case("hash") {
            return "{hash:0:2}/{hash:2:2}/{fileName}";
        }
        if raw.eq_ignore_ascii_case("date") {
            return "{date:%Y-%m-%d}/{fileName}";
        }
        raw
    }

    fn requires_hash(&self) -> bool {
        self.effective_template().contains("{hash")
    }

    fn move_file(source: &Path, destination: &Path) -> Result<()> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create destination directory {}", parent.display())
            })?;
        }

        match fs::rename(source, destination) {
            Ok(()) => Ok(()),
            Err(_) => {
                fs::copy(source, destination).with_context(|| {
                    format!(
                        "failed to copy source {} to {}",
                        source.display(),
                        destination.display()
                    )
                })?;
                fs::remove_file(source)
                    .with_context(|| format!("failed to remove source {}", source.display()))?;
                Ok(())
            }
        }
    }
}

impl ImageCategorizer for TemplateCategorizer {
    fn name(&self) -> &'static str {
        "template"
    }

    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult> {
        let working_dir = request
            .properties()
            .get_by_alias::<PathBuf>(ImageProcessKeys::WORKING_DIRECTORY)
            .ok_or_else(|| anyhow!("working directory not found in properties for categorization"))?;

        let file_name = request
            .source_file()
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| anyhow!("invalid source file name"))?
            .to_string();

        let mut render_props = PropertyMap::new();
        render_props.insert::<String>(file_name).alias("file_name");
        render_props
            .insert::<DateTime<Utc>>(
                request
                    .properties()
                    .get_by_alias::<Option<DateTime<Utc>>>(ImageProcessKeys::EXIF_DATE_TAKEN)
                    .and_then(|value| value.as_ref().cloned())
                    .unwrap_or_else(Utc::now),
            )
            .alias("effective_date");

        let hash = request
            .properties()
            .get_by_alias::<String>(ImageProcessKeys::HASH)
            .cloned();

        if let Some(hash_value) = hash.clone() {
            render_props.insert::<String>(hash_value).alias("hash");
        }

        let relative = TemplateEngine::compile(self.effective_template())?
            .render(&PropertyMapTemplateContext::new(render_props))?;
        let final_path = working_dir.join(relative);

        Self::move_file(request.source_file(), &final_path)?;

        Ok(CategorizeResult {
            final_path,
            hash,
        })
    }
}
