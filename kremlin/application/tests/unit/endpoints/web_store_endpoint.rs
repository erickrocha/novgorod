use super::WebStorePageQuery;

#[test]
fn accepts_current_and_legacy_filter_parameter_names() {
    for query in [
        r#"{"minPrice":100,"maxPrice":500,"sortBy":"price-asc"}"#,
        r#"{"min_price":100,"max_price":500,"sort_by":"price-asc"}"#,
    ] {
        let parsed: WebStorePageQuery = serde_json::from_str(query).unwrap();
        assert_eq!(parsed.min_price, Some(100));
        assert_eq!(parsed.max_price, Some(500));
        assert_eq!(parsed.sort_by.as_deref(), Some("price-asc"));
    }
}
