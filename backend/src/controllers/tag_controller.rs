use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::route::EndpointRoute;

pub struct TagController;

impl Controller for TagController {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}
