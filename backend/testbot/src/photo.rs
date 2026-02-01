use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use nimble_web::testbot::{TestBot, TestError, TestResult, TestScenario, TestStep};

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
        vec![Box::new(ListPhotosStep), Box::new(CreatePhotoStep::new())]
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
            "rating": json!(null),
            "label": json!(null),
            "description": json!(null),
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

        bot.log_info(format_args!(
            "create-photo returned status {}",
            response.status
        ));
        Ok(())
    }
}
