use nimble_photos::entities::photo_browse::{BrowseDimension, BrowseOptions, SortDirection};
use nimble_photos::models::browse_dimension_sql_adapter::{BrowseDimensionSqlAdapter, SqlParam};

#[test]
fn group_select_builds_expected_sql() {
    let year = BrowseDimensionSqlAdapter::new(BrowseDimension::Year);
    let date = BrowseDimensionSqlAdapter::new(BrowseDimension::Date);
    let month = BrowseDimensionSqlAdapter::new(BrowseDimension::Month);
    let camera = BrowseDimensionSqlAdapter::new(BrowseDimension::Camera);
    let rating = BrowseDimensionSqlAdapter::new(BrowseDimension::Rating);

    assert_eq!(
        year.group_select(),
        (
            "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int AS folder",
            "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int"
        )
    );
    assert_eq!(
        date.group_select(),
        (
            "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') AS folder",
            "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD')"
        )
    );
    assert_eq!(
        month.group_select(),
        (
            "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM') AS folder",
            "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM')"
        )
    );
    assert_eq!(
        camera.group_select(),
        ("p.camera_model AS folder", "p.camera_model")
    );
    assert_eq!(rating.group_select(), ("p.rating AS folder", "p.rating"));
}

#[test]
fn filter_clause_builds_expected_sql() {
    let year = BrowseDimensionSqlAdapter::new(BrowseDimension::Year);
    let date = BrowseDimensionSqlAdapter::new(BrowseDimension::Date);
    let month = BrowseDimensionSqlAdapter::new(BrowseDimension::Month);
    let camera = BrowseDimensionSqlAdapter::new(BrowseDimension::Camera);
    let rating = BrowseDimensionSqlAdapter::new(BrowseDimension::Rating);

    assert_eq!(
        year.filter_clause(1),
        "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int = $1"
    );
    assert_eq!(
        date.filter_clause(2),
        "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') = $2"
    );
    assert_eq!(
        month.filter_clause(3),
        "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM') = $3"
    );
    assert_eq!(camera.filter_clause(4), "p.camera_model = $4");
    assert_eq!(rating.filter_clause(5), "p.rating = $5");
}

#[test]
fn parse_segment_value_validates_input() {
    let year = BrowseDimensionSqlAdapter::new(BrowseDimension::Year);
    let date = BrowseDimensionSqlAdapter::new(BrowseDimension::Date);
    let month = BrowseDimensionSqlAdapter::new(BrowseDimension::Month);
    let camera = BrowseDimensionSqlAdapter::new(BrowseDimension::Camera);
    let rating = BrowseDimensionSqlAdapter::new(BrowseDimension::Rating);

    assert_eq!(
        year.parse_segment_value("2026").unwrap(),
        SqlParam::Int(2026)
    );
    assert_eq!(
        date.parse_segment_value("2026-01-25").unwrap(),
        SqlParam::String("2026-01-25".to_string())
    );
    assert_eq!(
        month.parse_segment_value("2026-01").unwrap(),
        SqlParam::String("2026-01".to_string())
    );
    assert_eq!(
        camera.parse_segment_value("Fujifilm X100V").unwrap(),
        SqlParam::String("Fujifilm X100V".to_string())
    );
    assert_eq!(rating.parse_segment_value("5").unwrap(), SqlParam::Int(5));

    assert!(year.parse_segment_value("bad").is_err());
    assert!(date.parse_segment_value("2026/01/25").is_err());
    assert!(month.parse_segment_value("2026-13").is_err());
    assert!(rating.parse_segment_value("five").is_err());
}

#[test]
fn order_direction_maps_values() {
    assert_eq!(
        BrowseDimensionSqlAdapter::order_direction(&SortDirection::Asc),
        "ASC"
    );
    assert_eq!(
        BrowseDimensionSqlAdapter::order_direction(&SortDirection::Desc),
        "DESC"
    );
}

#[test]
fn browse_options_default_date_format_is_yyyy_mm_dd() {
    let options = BrowseOptions::default();
    assert_eq!(options.date_format, "yyyy-MM-dd");
}
