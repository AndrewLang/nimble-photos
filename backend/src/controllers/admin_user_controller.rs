use async_trait::async_trait;
use uuid::Uuid;

use crate::dtos::admin_user_dto::UpdateUserRolesRequest;
use crate::services::AdminUserService;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct AdminUserController;

impl Controller for AdminUserController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/admin/users", ListAdminUsersHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::put("/api/admin/users/{id}/roles", UpdateUserRolesHandler)
                .with_policy(Policy::Authenticated)
                .build(),
        ]
    }
}

struct ListAdminUsersHandler;

#[async_trait]
impl HttpHandler for ListAdminUsersHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        if !is_admin(context) {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let service = context.service::<AdminUserService>()?;
        let users = service.list_users().await?;
        Ok(ResponseValue::json(users))
    }
}

struct UpdateUserRolesHandler;

#[async_trait]
impl HttpHandler for UpdateUserRolesHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        if !is_admin(context) {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let payload = context
            .read_json::<UpdateUserRolesRequest>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let user_id =
            Uuid::parse_str(id_param).map_err(|_| PipelineError::message("invalid user id"))?;
        let current_user_id = current_user_id(context)?;

        if user_id == current_user_id && !contains_admin_role(&payload.roles) {
            return Err(PipelineError::message(
                "Admin cannot remove the admin role from their own account",
            ));
        }

        let service = context.service::<AdminUserService>()?;
        let updated = service.update_roles(user_id, payload.roles).await?;
        Ok(ResponseValue::json(updated))
    }
}

fn is_admin(context: &HttpContext) -> bool {
    context
        .get::<IdentityContext>()
        .map(|ctx| ctx.identity().claims().roles().contains("admin"))
        .unwrap_or(false)
}

fn current_user_id(context: &HttpContext) -> Result<Uuid, PipelineError> {
    let subject = context
        .get::<IdentityContext>()
        .ok_or_else(|| PipelineError::message("identity not found"))?
        .identity()
        .subject()
        .to_string();
    Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))
}

fn contains_admin_role(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|role| role.trim().eq_ignore_ascii_case("admin"))
}
