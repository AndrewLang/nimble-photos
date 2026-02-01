use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use nimble_photos::dtos::auth_dtos::{
    ChangePasswordRequest, LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest,
    RegisterRequest, ResetPasswordRequest, VerifyEmailRequest,
};
use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_web::testbot::{
    AssertResponse, ComboStep, TestBot, TestError, TestResult, TestScenario, TestStep,
};

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

pub struct AuthScenario {
    email: String,
    password: String,
    display_name: String,
    changed_password: String,
    reset_password: String,
}

impl AuthScenario {
    pub fn new() -> Self {
        let nonce = Uuid::new_v4();
        Self {
            email: format!("test+{nonce}@example.com"),
            password: "TestBotPass#1".to_string(),
            display_name: "Test Bot User".to_string(),
            changed_password: "TestBotPass#2".to_string(),
            reset_password: "TestBotPass#3".to_string(),
        }
    }
}

#[async_trait(?Send)]
impl TestScenario for AuthScenario {
    fn name(&self) -> &'static str {
        "Auth endpoints"
    }

    fn steps(&self) -> Vec<Box<dyn TestStep>> {
        log::info!(
            "Creating steps for AuthScenario {}",
            self.display_name.clone()
        );
        let logout_flow = ComboStep::new(
            "Logout flow",
            "/api/auth/logout",
            vec![
                Box::new(LoginStep::new(self.email.clone(), self.password.clone())),
                Box::new(LogoutStep),
            ],
        );

        let change_password_flow = ComboStep::new(
            "Change Password flow",
            "/api/auth/change-password",
            vec![
                Box::new(LoginStep::new(self.email.clone(), self.password.clone())),
                Box::new(ChangePasswordStep::new(
                    self.password.clone(),
                    self.changed_password.clone(),
                )),
            ],
        );

        vec![
            Box::new(RegisterStep::new(
                self.email.clone(),
                self.password.clone(),
                self.display_name.clone(),
            )),
            Box::new(LoginStep::new(self.email.clone(), self.password.clone())),
            Box::new(MeStep::new(self.email.clone(), self.display_name.clone())),
            Box::new(RefreshStep),
            Box::new(logout_flow),
            Box::new(change_password_flow),
            Box::new(LoginStep::new(
                self.email.clone(),
                self.changed_password.clone(),
            )),
            Box::new(ResetPasswordStep::new(
                self.email.clone(),
                self.reset_password.clone(),
            )),
            Box::new(LoginStep::new(
                self.email.clone(),
                self.reset_password.clone(),
            )),
            Box::new(VerifyEmailStep::new(self.email.clone())),
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

#[async_trait(?Send)]
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
            .set_str("refresh_token", payload.refresh_token.clone());

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

#[async_trait(?Send)]
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
            .set_str("refresh_token", payload.refresh_token.clone());

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

#[async_trait(?Send)]
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
        let email = profile.email.clone();
        let display_name = profile.display_name.clone();

        bot.assert_equals_named("email", email.clone(), self.expected_email.clone());
        bot.assert_equals_named(
            "display_name",
            display_name.clone(),
            self.expected_display_name.clone(),
        );

        Ok(())
    }
}

struct RefreshStep;

#[async_trait(?Send)]
impl TestStep for RefreshStep {
    fn name(&self) -> &'static str {
        "refresh"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/refresh"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let refresh_token = bot
            .context
            .get_str("refresh_token")
            .ok_or_else(|| TestError::msg("refresh token missing"))?;

        let request = RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
        };

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;

        let payload: LoginResponse = response.json()?;
        bot.context.access_token = Some(payload.access_token.clone());
        bot.context
            .set_str("refresh_token", payload.refresh_token.clone());
        bot.log_info("refresh completed");

        Ok(())
    }
}

struct LogoutStep;

#[async_trait(?Send)]
impl TestStep for LogoutStep {
    fn name(&self) -> &'static str {
        "logout"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/logout"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let refresh_token = bot
            .context
            .get_str("refresh_token")
            .ok_or_else(|| TestError::msg("refresh token missing"))?;

        let request = LogoutRequest {
            refresh_token: refresh_token.clone(),
        };

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;

        bot.context.access_token = None;
        bot.context.set("refresh_token", json!(null));
        bot.log_info("logout completed");

        Ok(())
    }
}

struct ChangePasswordStep {
    old_password: String,
    new_password: String,
}

impl ChangePasswordStep {
    fn new(old_password: String, new_password: String) -> Self {
        Self {
            old_password,
            new_password,
        }
    }
}

#[async_trait(?Send)]
impl TestStep for ChangePasswordStep {
    fn name(&self) -> &'static str {
        "change-password"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/change-password"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let request = ChangePasswordRequest {
            old_password: self.old_password.clone(),
            new_password: self.new_password.clone(),
        };

        let response = bot.post_auth(self.endpoint(), &request).await?;
        response.assert_status(200)?;
        bot.log_info("change-password completed");

        Ok(())
    }
}

struct ResetPasswordStep {
    email: String,
    new_password: String,
}

impl ResetPasswordStep {
    fn new(email: String, new_password: String) -> Self {
        Self {
            email,
            new_password,
        }
    }
}

#[async_trait(?Send)]
impl TestStep for ResetPasswordStep {
    fn name(&self) -> &'static str {
        "reset-password"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/reset-password"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let token_resp: TokenResponse = {
            let request = json!({ "email": self.email.clone() });
            let response = bot.post("/api/test/auth/reset-token", &request).await?;
            response.assert_status(200)?;
            response.json()?
        };

        let request = ResetPasswordRequest {
            token: token_resp.token,
            new_password: self.new_password.clone(),
        };

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;
        bot.log_info("reset-password completed");

        Ok(())
    }
}

struct VerifyEmailStep {
    email: String,
}

impl VerifyEmailStep {
    fn new(email: String) -> Self {
        Self { email }
    }
}

#[async_trait(?Send)]
impl TestStep for VerifyEmailStep {
    fn name(&self) -> &'static str {
        "verify-email"
    }

    fn endpoint(&self) -> &'static str {
        "/api/auth/verify-email"
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        let request = json!({ "email": self.email.clone() });
        let response = bot.post("/api/test/auth/verify-token", &request).await?;
        response.assert_status(200)?;
        let token_resp: TokenResponse = response.json()?;
        let request = VerifyEmailRequest {
            token: token_resp.token,
        };

        let response = bot.post(self.endpoint(), &request).await?;
        response.assert_status(200)?;
        bot.log_info("verify-email completed");

        Ok(())
    }
}
