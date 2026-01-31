use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use nimble_photos::dtos::auth_dtos::{LoginRequest, LoginResponse, RegisterRequest};
use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_web::testbot::{AssertResponse, TestBot, TestError, TestResult, TestScenario, TestStep};

pub struct AuthScenario {
    email: String,
    password: String,
    display_name: String,
}

impl AuthScenario {
    pub fn new() -> Self {
        let nonce = Uuid::new_v4();
        Self {
            email: format!("test+{nonce}@example.com"),
            password: "TestBotPass#1".to_string(),
            display_name: "Test Bot User".to_string(),
        }
    }
}

#[async_trait]
impl TestScenario for AuthScenario {
    fn name(&self) -> &'static str {
        "auth-scenario"
    }

    fn steps(&self) -> Vec<Box<dyn TestStep>> {
        log::info!(
            "Creating steps for AuthScenario {}",
            self.display_name.clone()
        );
        vec![
            Box::new(RegisterStep::new(
                self.email.clone(),
                self.password.clone(),
                self.display_name.clone(),
            )),
            Box::new(LoginStep::new(self.email.clone(), self.password.clone())),
            Box::new(MeStep::new(self.email.clone(), self.display_name.clone())),
        ]
    }
}

struct RegisterStep {
    email: String,
    password: String,
    display_name: String,
}

impl RegisterStep {
    fn new(email: String, password: String, display_name: String) -> Self {
        Self {
            email,
            password,
            display_name,
        }
    }
}

#[async_trait]
impl TestStep for RegisterStep {
    fn name(&self) -> &'static str {
        "register"
    }
    fn endpoint(&self) -> &'static str {
        "/api/auth/register"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let request = RegisterRequest {
            email: self.email.clone(),
            password: self.password.clone(),
            confirm_password: self.password.clone(),
            display_name: self.display_name.clone(),
        };

        log::info!(
            "Running RegisterStep for email: {}, display_name: {}",
            self.email,
            &request.display_name
        );

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;

        let payload: LoginResponse = response.json()?;
        bot.context.access_token = Some(payload.access_token.clone());
        bot.context
            .set("refresh_token", json!(payload.refresh_token));

        Ok(())
    }
}

struct LoginStep {
    email: String,
    password: String,
}

impl LoginStep {
    fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

#[async_trait]
impl TestStep for LoginStep {
    fn name(&self) -> &'static str {
        "login"
    }
    fn endpoint(&self) -> &'static str {
        "/api/auth/login"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let request = LoginRequest {
            email: self.email.clone(),
            password: self.password.clone(),
        };

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;

        let payload: LoginResponse = response.json()?;
        bot.context.access_token = Some(payload.access_token.clone());
        bot.context
            .set("refresh_token", json!(payload.refresh_token));

        Ok(())
    }
}

struct MeStep {
    expected_email: String,
    expected_display_name: String,
}

impl MeStep {
    fn new(expected_email: String, expected_display_name: String) -> Self {
        log::info!(
            "Creating MeStep with expected_email: {}, expected_display_name: {}",
            expected_email,
            expected_display_name
        );
        Self {
            expected_email,
            expected_display_name,
        }
    }
}

#[async_trait]
impl TestStep for MeStep {
    fn name(&self) -> &'static str {
        "me"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/me"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let response = bot.get_auth(self.endpoint()).await?;
        response.assert_status(200)?;

        let profile: UserProfileDto = response.json()?;
        log::info!(
            "MeStep received profile: email={}, display_name={}",
            profile.email,
            profile.display_name
        );

        if profile.email != self.expected_email {
            return Err(TestError::msg(format!(
                "❌ Expected email [{}], got [{}]",
                self.expected_email, profile.email
            )));
        }
        if profile.display_name != self.expected_display_name {
            return Err(TestError::msg(format!(
                "❌ Expected display name [{}], got [{}]",
                self.expected_display_name, profile.display_name
            )));
        }

        Ok(())
    }
}
