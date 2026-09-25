    use super::*;

    #[test]
    fn defaults_and_clamping_work_as_expected() {
        let empty_query = PageQuery {
            page: None,
            page_size: None,
            q: None,
            sort_by: None,
            sort_dir: None,
        };
        let p = NormalizedPagination::new(&empty_query, &["name", "id"], "id");
        assert_eq!(p.page, 1);
        assert_eq!(p.page_size, 25);
        assert_eq!(p.offset, 0);
        assert_eq!(p.sort_by, "id");
        assert_eq!(p.sort_dir, SortDirection::Asc);
        assert_eq!(p.q, None);

        let clamped_query = PageQuery {
            page: Some(0),
            page_size: Some(999),
            q: Some("  term  ".to_string()),
            sort_by: Some("NAME".to_string()),
            sort_dir: Some("DESC".to_string()),
        };
        let p2 = NormalizedPagination::new(&clamped_query, &["name", "id"], "id");
        assert_eq!(p2.page, 1);
        assert_eq!(p2.page_size, 100);
        assert_eq!(p2.offset, 0);
        assert_eq!(p2.sort_by, "name");
        assert_eq!(p2.sort_dir, SortDirection::Desc);
        assert_eq!(p2.q, Some("term".to_string()));
    }

    #[test]
    fn unknown_sort_field_falls_back_safely() {
        let bad_sort_query = PageQuery {
            page: Some(2),
            page_size: Some(10),
            q: None,
            sort_by: Some("malicious_column; DROP TABLE".to_string()),
            sort_dir: Some("asc".to_string()),
        };
        let p = NormalizedPagination::new(&bad_sort_query, &["name", "email", "id"], "id");
        assert_eq!(p.page, 2);
        assert_eq!(p.page_size, 10);
        assert_eq!(p.offset, 10);
        assert_eq!(p.sort_by, "id");
    }

    #[test]
    fn page_offset_and_boundaries() {
        let q = PageQuery {
            page: Some(5),
            page_size: Some(20),
            q: None,
            sort_by: None,
            sort_dir: None,
        };
        let p = NormalizedPagination::new(&q, &["id"], "id");
        assert_eq!(p.page, 5);
        assert_eq!(p.page_size, 20);
        assert_eq!(p.offset, 80);
    }

    #[test]
    fn paged_response_serializes_with_camel_case() {
        let resp = PagedResponse::new(vec!["test"], 100, 2, 25);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""pageSize":25"#));
        assert!(json.contains(r#""page":2"#));
        assert!(json.contains(r#""total":100"#));
        assert!(json.contains(r#""items":["test"]"#));

        let empty: PagedResponse<String> = PagedResponse::empty(1, 25);
        assert_eq!(empty.total.unwrap(), 0);
        assert_eq!(empty.items.len(), 0);
        assert_eq!(empty.page.unwrap(), 1);
        assert_eq!(empty.page_size.unwrap(), 25);
    }

    #[test]
    fn sort_direction_normalization() {
        assert_eq!(SortDirection::from_optional_str(None), SortDirection::Asc);
        assert_eq!(SortDirection::from_optional_str(Some("asc")), SortDirection::Asc);
        assert_eq!(SortDirection::from_optional_str(Some("ASC")), SortDirection::Asc);
        assert_eq!(SortDirection::from_optional_str(Some("  desc  ")), SortDirection::Desc);
        assert_eq!(SortDirection::from_optional_str(Some("DESC")), SortDirection::Desc);
        assert_eq!(SortDirection::from_optional_str(Some("invalid")), SortDirection::Asc);
    }
