    use super::normalize_country_code;

    #[test]
    fn country_code_is_trimmed_and_uppercased() {
        assert_eq!(normalize_country_code(" br "), Some("BR".to_string()));
        assert_eq!(normalize_country_code("Us"), Some("US".to_string()));
    }

    #[test]
    fn country_code_rejects_missing_or_malformed_values() {
        for invalid in ["", "B", "BRA", "B1", "éR"] {
            assert_eq!(normalize_country_code(invalid), None);
        }
    }
