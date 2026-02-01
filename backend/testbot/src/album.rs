use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use nimble_web::testbot::{AssertResponse, TestBot, TestError, TestResult, TestScenario, TestStep};

pub struct AlbumScenario;

impl AlbumScenario {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl TestScenario for AlbumScenario {
    fn name(&self) -> &'static str {
        "Album endpoints"
    }

    fn steps(&self) -> Vec<Box<dyn TestStep>> {
        vec![
            Box::new(ListAlbumsStep),
            Box::new(CreateAlbumStep::new()),
            Box::new(GetAlbumStep),
            Box::new(UpdateAlbumStep),
            Box::new(DeleteAlbumStep),
        ]
    }
}

struct ListAlbumsStep;

#[async_trait(?Send)]
impl TestStep for ListAlbumsStep {
    fn name(&self) -> &'static str {
        "list-albums"
    }

    fn endpoint(&self) -> &'static str {
        "/api/albums"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let path = format!("{}/{}", self.endpoint(), "1/20");
        let response = bot.get_auth(&path).await?;
        response.assert_status(200)?;
        bot.log_info(format!("list-albums returned status {}", response.status));
        Ok(())
    }
}

struct CreateAlbumStep {
    id: String,
}

impl CreateAlbumStep {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }
}

#[async_trait(?Send)]
impl TestStep for CreateAlbumStep {
    fn name(&self) -> &'static str {
        "create-album"
    }

    fn endpoint(&self) -> &'static str {
        "/api/albums"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let now = Utc::now();
        let payload = json!({
            "id": self.id,
            "name": "TestBot Album",
            "description": "Created by TestBot",
            "owner_id": json!(null),
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339(),
        });

        let response = bot.post_auth(self.endpoint(), &payload).await?;
        response.assert_status(200)?;

        let created: Value = response.json()?;
        let id = created
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| TestError::msg("create album response missing id"))?
            .to_string();

        bot.context.set_str("album_id", id);
        bot.context.set("album_snapshot", created);
        bot.log_info(format!("create-album returned status {}", response.status));
        Ok(())
    }
}

struct GetAlbumStep;

#[async_trait(?Send)]
impl TestStep for GetAlbumStep {
    fn name(&self) -> &'static str {
        "get-album"
    }

    fn endpoint(&self) -> &'static str {
        "/api/albums/{id}"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let id = bot
            .context
            .get_str("album_id")
            .ok_or_else(|| TestError::msg("album id missing"))?;
        let path = format!("{}/{}", "/api/albums", id);
        let response = bot.get_auth(&path).await?;
        response.assert_status(200)?;

        let album: Value = response.json()?;
        bot.context.set("album_snapshot", album);
        bot.log_info(format!("get-album returned status {}", response.status));
        Ok(())
    }
}

struct UpdateAlbumStep;

#[async_trait(?Send)]
impl TestStep for UpdateAlbumStep {
    fn name(&self) -> &'static str {
        "update-album"
    }

    fn endpoint(&self) -> &'static str {
        "/api/albums"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let mut snapshot = bot
            .context
            .get("album_snapshot")
            .cloned()
            .ok_or_else(|| TestError::msg("album snapshot missing"))?;

        let obj = snapshot
            .as_object_mut()
            .ok_or_else(|| TestError::msg("album snapshot not an object"))?;

        obj.insert(
            "name".to_string(),
            Value::String("Updated TestBot Album".to_string()),
        );
        obj.insert(
            "description".to_string(),
            Value::String("Updated by TestBot".to_string()),
        );
        obj.insert(
            "updated_at".to_string(),
            Value::String(Utc::now().to_rfc3339()),
        );

        let response = bot.put_auth(self.endpoint(), &snapshot).await?;
        response.assert_status(200)?;

        let updated: Value = response.json()?;
        bot.context.set("album_snapshot", updated);
        bot.log_info(format!("update-album returned status {}", response.status));
        Ok(())
    }
}

struct DeleteAlbumStep;

#[async_trait(?Send)]
impl TestStep for DeleteAlbumStep {
    fn name(&self) -> &'static str {
        "delete-album"
    }

    fn endpoint(&self) -> &'static str {
        "/api/albums/{id}"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let id = bot
            .context
            .get_str("album_id")
            .ok_or_else(|| TestError::msg("album id missing"))?;
        let path = format!("{}/{}", "/api/albums", id);
        let response = bot.delete_auth(&path).await?;
        response.assert_status(200)?;

        bot.context.set("album_snapshot", json!(null));
        bot.log_info(format!("delete-album returned status {}", response.status));
        Ok(())
    }
}
