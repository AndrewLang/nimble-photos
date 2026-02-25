use async_trait::async_trait;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::dtos::admin_user_dto::UpdateUserRolesRequest;
use crate::services::AdminUserService;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{get, put};

pub struct AdminUserController;

impl Controller for AdminUserController {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct ListAdminUsersHandler;

#[async_trait]
#[get("/api/admin/users", policy = Policy::Authenticated)]
impl HttpHandler for ListAdminUsersHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        if !context.is_admin() {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let service = context.service::<AdminUserService>()?;
        let users = service.list_users().await?;
        Ok(ResponseValue::json(users))
    }
}

struct UpdateUserRolesHandler;

impl UpdateUserRolesHandler {
    fn contains_admin_role(&self, roles: &[String]) -> bool {
        roles
            .iter()
            .any(|role| role.trim().eq_ignore_ascii_case("admin"))
    }
}

#[async_trait]
#[put("/api/admin/users/{id}/roles", policy = Policy::Authenticated)]
impl HttpHandler for UpdateUserRolesHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        if !context.is_admin() {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let payload = context
            .read_json::<UpdateUserRolesRequest>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let user_id = context.entity_id()?;
        let current_user_id = context.current_user_id()?;

        if user_id == current_user_id && !self.contains_admin_role(&payload.roles) {
            return Err(PipelineError::message(
                "Admin cannot remove the admin role from their own account",
            ));
        }

        let service = context.service::<AdminUserService>()?;
        let updated = service.update_roles(user_id, payload.roles).await?;
        Ok(ResponseValue::json(updated))
    }
}
