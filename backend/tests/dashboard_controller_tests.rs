use nimble_photos::controllers::dashboard_controller::DashboardController;
use nimble_web::Controller;
use nimble_web::security::policy::Policy;

#[test]
fn routes_require_admin_role() {
    let routes = DashboardController::routes();
    assert_eq!(routes.len(), 4);

    let list_route = &routes[0];
    assert_eq!(list_route.route.method(), "GET");
    assert_eq!(list_route.route.path(), "/api/dashboard/settings");
    assert_eq!(
        list_route.endpoint.metadata().policy(),
        Some(&Policy::InRole("admin".to_string()))
    );

    let get_route = &routes[1];
    assert_eq!(get_route.route.method(), "GET");
    assert_eq!(get_route.route.path(), "/api/dashboard/settings/{key}");
    assert_eq!(
        get_route.endpoint.metadata().policy(),
        Some(&Policy::InRole("admin".to_string()))
    );

    let update_route = &routes[2];
    assert_eq!(update_route.route.method(), "PUT");
    assert_eq!(update_route.route.path(), "/api/dashboard/settings/{key}");
    assert_eq!(
        update_route.endpoint.metadata().policy(),
        Some(&Policy::InRole("admin".to_string()))
    );

    let upload_route = &routes[3];
    assert_eq!(upload_route.route.method(), "POST");
    assert_eq!(
        upload_route.route.path(),
        "/api/dashboard/settings/site.logo/upload"
    );
    assert_eq!(
        upload_route.endpoint.metadata().policy(),
        Some(&Policy::InRole("admin".to_string()))
    );
}
