use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};

use nimble_web::testbot::{AssertResponse, TestBot, TestError, TestResult, TestScenario, TestStep};
use uuid::Uuid;

pub struct PhotoScenario;

impl PhotoScenario {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl TestScenario for PhotoScenario {
    fn name(&self) -> &'static str {
        "Photo endpoints"
    }

    fn steps(&self) -> Vec<Box<dyn TestStep>> {
        vec![
            Box::new(ListPhotosStep),
            Box::new(CreatePhotoStep::new()),
            Box::new(GetPhotoStep),
            Box::new(UpdatePhotoStep),
            Box::new(ScanPhotosStep),
            Box::new(DeletePhotoStep),
        ]
    }
}

struct ListPhotosStep;

#[async_trait(?Send)]
impl TestStep for ListPhotosStep {
    fn name(&self) -> &'static str {
        "list-photos"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let list_endpoint = format!("{}/1/20", self.endpoint());
        let response = bot.get_auth(&list_endpoint).await?;
        if response.status >= 500 {
            return Err(TestError::msg(format!(
                "{}: {}",
                response.status,
                response.text()
            )));
        }

        bot.log_info(format_args!(
            "list-photos returned status {}",
            response.status
        ));
        Ok(())
    }
}

struct CreatePhotoStep {
    hash: String,
    path: String,
    name: String,
    format: String,
    size: i64,
    width: u32,
    height: u32,
    storage_id: Uuid,
    day_date: String,
    sort_date: String,
    metadata_extracted: bool,
    is_raw: bool,
}

impl CreatePhotoStep {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            hash: "testhash".to_string(),
            path: "/photos/testhash".to_string(),
            name: "test-photo".to_string(),
            format: "jpeg".to_string(),
            size: 1024,
            width: 1920,
            height: 1080,
            storage_id: Uuid::new_v4(),
            day_date: now.date_naive().to_string(),
            sort_date: now.to_rfc3339(),
            metadata_extracted: true,
            is_raw: false,
        }
    }
}

#[async_trait(?Send)]
impl TestStep for CreatePhotoStep {
    fn name(&self) -> &'static str {
        "create-photo"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let now = Utc::now();
        let payload = json!({
            "storage_id": self.storage_id.to_string(),
            "hash": self.hash,
            "path": self.path,
            "name": self.name,
            "format": self.format,
            "size": self.size,
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339(),
            "date_imported": now.to_rfc3339(),
            "date_taken": now.to_rfc3339(),
            "day_date": self.day_date,
            "sort_date": self.sort_date,
            "metadata_extracted": self.metadata_extracted,
            "is_raw": self.is_raw,
            "width": self.width,
            "height": self.height,
        });

        let response = bot.post_auth(self.endpoint(), &payload).await?;
        if response.status >= 500 {
            return Err(TestError::msg(format!(
                "{}: {}",
                response.status,
                response.text()
            )));
        }

        let created_photo: Value = response.json()?;
        let id = created_photo
            .get("id")
            .and_then(|value| {
                if let Some(text) = value.as_str() {
                    Some(text.to_string())
                } else if let Some(number) = value.as_i64() {
                    Some(number.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| TestError::msg("create photo response missing id"))?;

        bot.context.set_str("created_photo_id", id);
        bot.context.set("photo_snapshot", created_photo.clone());

        bot.log_info(format_args!(
            "create-photo returned status {}",
            response.status
        ));
        Ok(())
    }
}

struct GetPhotoStep;

#[async_trait(?Send)]
impl TestStep for GetPhotoStep {
    fn name(&self) -> &'static str {
        "get-photo"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos/{id}"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let id = bot
            .context
            .get_str("created_photo_id")
            .ok_or_else(|| TestError::msg("photo id missing"))?;
        let path = format!("{}/{}", "/api/photos", id);
        let response = bot.get_auth(&path).await?;
        response.assert_status(200)?;

        let photo: Value = response.json()?;
        bot.context.set("photo_snapshot", photo);
        bot.log_info(format!("get-photo returned status {}", response.status));
        Ok(())
    }
}

struct UpdatePhotoStep;

#[async_trait(?Send)]
impl TestStep for UpdatePhotoStep {
    fn name(&self) -> &'static str {
        "update-photo"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let mut payload = bot
            .context
            .get("photo_snapshot")
            .cloned()
            .ok_or_else(|| TestError::msg("photo snapshot missing"))?;

        let obj = payload
            .as_object_mut()
            .ok_or_else(|| TestError::msg("photo snapshot is not an object"))?;

        obj.insert(
            "name".to_string(),
            Value::String("test-photo-updated".to_string()),
        );
        obj.insert("format".to_string(), Value::String("png".to_string()));
        obj.insert("size".to_string(), Value::from(2048));
        obj.insert("metadata_extracted".to_string(), Value::Bool(false));
        obj.insert(
            "updated_at".to_string(),
            Value::String(Utc::now().to_rfc3339()),
        );

        let response = bot.put_auth(self.endpoint(), &payload).await?;
        response.assert_status(200)?;

        let updated = response.json()?;
        bot.context.set("photo_snapshot", updated);
        bot.log_info(format!("update-photo returned status {}", response.status));
        Ok(())
    }
}

struct ScanPhotosStep;

#[async_trait(?Send)]
impl TestStep for ScanPhotosStep {
    fn name(&self) -> &'static str {
        "scan-photo"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos/scan"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let response = bot.post_auth(self.endpoint(), &json!({})).await?;
        response.assert_status(200)?;

        bot.log_info(format!("scan-photo returned status {}", response.status));
        Ok(())
    }
}

struct DeletePhotoStep;

#[async_trait(?Send)]
impl TestStep for DeletePhotoStep {
    fn name(&self) -> &'static str {
        "delete-photo"
    }

    fn endpoint(&self) -> &'static str {
        "/api/photos/{id}"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let id = bot
            .context
            .get_str("created_photo_id")
            .ok_or_else(|| TestError::msg("photo id missing"))?;
        let path = format!("{}/{}", "/api/photos", id);
        let response = bot.delete_auth(&path).await?;
        response.assert_status(200)?;

        bot.context.set("photo_snapshot", json!(null));
        bot.log_info(format!("delete-photo returned status {}", response.status));
        Ok(())
    }
}
