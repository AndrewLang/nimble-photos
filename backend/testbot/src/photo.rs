use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use nimble_web::testbot::{AssertResponse, TestBot, TestError, TestResult, TestScenario, TestStep};

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
    file_name: String,
    file_size: i64,
}

impl CreatePhotoStep {
    fn new() -> Self {
        Self {
            hash: "testhash".to_string(),
            path: "/photos/testhash".to_string(),
            file_name: "test-file.jpg".to_string(),
            file_size: 1024,
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
            "id": Uuid::new_v4().to_string(),
            "hash": self.hash,
            "path": self.path,
            "file_name": self.file_name,
            "file_size": self.file_size,
            "rating": 3,
            "label": "red",
            "description": "This is a test photo",
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339(),
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
            .and_then(Value::as_str)
            .ok_or_else(|| TestError::msg("create photo response missing id"))?
            .to_string();

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
            "file_name".to_string(),
            Value::String("test-file-updated.jpg".to_string()),
        );
        obj.insert(
            "description".to_string(),
            Value::String("Updated via testbot".to_string()),
        );
        obj.insert("rating".to_string(), Value::from(5));
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
