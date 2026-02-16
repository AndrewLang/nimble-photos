use nimble_photos::controllers::admin_user_controller::AdminUserController;
use nimble_web::Controller;
use nimble_web::security::policy::Policy;

#[test]
fn routes_require_authenticated() {
    let routes = AdminUserController::routes();
    assert_eq!(routes.len(), 2);

    let list_route = &routes[0];
    assert_eq!(list_route.route.method(), "GET");
    assert_eq!(list_route.route.path(), "/api/admin/users");
    assert_eq!(
        list_route.endpoint.metadata().policy(),
        Some(&Policy::Authenticated)
    );

    let update_route = &routes[1];
    assert_eq!(update_route.route.method(), "PUT");
    assert_eq!(update_route.route.path(), "/api/admin/users/{id}/roles");
    assert_eq!(
        update_route.endpoint.metadata().policy(),
        Some(&Policy::Authenticated)
    );
}
