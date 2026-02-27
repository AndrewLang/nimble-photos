use crate::dtos::photo_dtos::TagRef;
use crate::entities::tag::Tag;

use async_trait::async_trait;
use nimble_web::PipelineError;
use nimble_web::Repository;
use nimble_web::data::query::Value;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

#[async_trait]
pub trait TagRepositoryExtensions {
    async fn set_photo_tags(
        &self,
        photo_id: Uuid,
        tag_refs: &[TagRef],
    ) -> Result<(), PipelineError>;

    async fn resolve_tag_ids(
        &self,
        refs: &[TagRef],
        default_visibility: i16,
    ) -> Result<Vec<Uuid>, PipelineError>;

    fn normalize_tag_name(&self, raw: &str) -> Option<(String, String)>;

    fn normalize_tag_names(&self, raw_tags: &[String]) -> Vec<(String, String)>;
}

#[async_trait]
impl TagRepositoryExtensions for Repository<Tag> {
    async fn set_photo_tags(
        &self,
        photo_id: Uuid,
        tag_refs: &[TagRef],
    ) -> Result<(), PipelineError> {
        let ids = self.resolve_tag_ids(tag_refs, 0).await?;

        if ids.is_empty() {
            self.raw_query::<serde_json::Value>(
                "DELETE FROM photo_tags WHERE photo_id = $1",
                &[Value::Uuid(photo_id)],
            )
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            return Ok(());
        }

        let mut params = Vec::with_capacity(ids.len() + 1);
        params.push(Value::Uuid(photo_id));
        for id in &ids {
            params.push(Value::Uuid(*id));
        }

        let values = (0..ids.len())
            .map(|idx| format!("(${})", idx + 2))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            r#"
            WITH deleted AS (
                DELETE FROM photo_tags WHERE photo_id = $1
            )
            INSERT INTO photo_tags (photo_id, tag_id)
            SELECT $1, v.tag_id
            FROM (VALUES {values}) AS v(tag_id)
            ON CONFLICT (photo_id, tag_id) DO NOTHING
            "#
        );

        self.raw_query::<serde_json::Value>(&sql, &params)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(())
    }

    async fn resolve_tag_ids(
        &self,
        refs: &[TagRef],
        default_visibility: i16,
    ) -> Result<Vec<Uuid>, PipelineError> {
        #[derive(Deserialize)]
        struct TagIdRow {
            id: Uuid,
        }

        let mut ids = Vec::<Uuid>::new();
        let mut names = Vec::<String>::new();

        for item in refs {
            match item {
                TagRef::Id(id) => ids.push(*id),
                TagRef::Name(name) => names.push(name.clone()),
            }
        }

        let normalized = self.normalize_tag_names(&names);
        let sql = r#"
            INSERT INTO tags (name, name_norm, visibility, created_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (name_norm) DO UPDATE
            SET name = EXCLUDED.name
            RETURNING id
        "#;

        for (name, name_norm) in normalized {
            let rows = self
                .raw_query::<TagIdRow>(
                    sql,
                    &[
                        Value::String(name),
                        Value::String(name_norm),
                        Value::I16(default_visibility),
                    ],
                )
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

            if let Some(row) = rows.first() {
                ids.push(row.id);
            }
        }

        ids.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        ids.dedup();
        Ok(ids)
    }

    fn normalize_tag_name(&self, raw: &str) -> Option<(String, String)> {
        let name = raw.trim();
        if name.is_empty() {
            return None;
        }
        Some((name.to_string(), name.to_lowercase()))
    }

    fn normalize_tag_names(&self, raw_tags: &[String]) -> Vec<(String, String)> {
        let mut dedup = BTreeMap::<String, String>::new();
        for raw in raw_tags {
            if let Some((name, name_norm)) = self.normalize_tag_name(raw) {
                dedup.entry(name_norm).or_insert(name);
            }
        }
        dedup.into_iter().map(|(norm, name)| (name, norm)).collect()
    }
}
