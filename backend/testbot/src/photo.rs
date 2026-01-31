use async_trait::async_trait;
use serde_json::json;

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
        let response = bot.get_auth(self.endpoint()).await?;
        if response.status >= 500 {
            return Err(TestError::msg(format!(
                "list-photos failed with status {}",
                response.status
            )));
        }

        log::info!("list-photos returned status {}", response.status);
        Ok(())
    }
}

struct CreatePhotoStep {
    hash: String,
    path: String,
    file_name: String,
    file_size: i32,
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
        let request = json!({
            "hash": self.hash,
            "path": self.path,
            "fileName": self.file_name,
            "fileSize": self.file_size,
        });

        let response = bot.post_auth(self.endpoint(), &request).await?;
        if response.status >= 500 {
            return Err(TestError::msg(format!(
                "create-photo failed with status {}",
                response.status
            )));
        }

        log::info!("create-photo returned status {}", response.status);
        Ok(())
    }
}
