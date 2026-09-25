    use super::{canonical_address, country_code, optional_text};

    #[test]
    fn canonical_address_wins_and_legacy_alias_fills_missing_value() {
        assert_eq!(
            canonical_address(Some("  Campinas ".into()), Some("São Paulo".into())),
            Some("Campinas".into())
        );
        assert_eq!(
            canonical_address(Some("  ".into()), Some(" São Paulo ".into())),
            Some("São Paulo".into())
        );
    }

    #[test]
    fn optional_address_values_are_cleaned_and_country_is_uppercase() {
        assert_eq!(optional_text(Some("  ".into())), None);
        assert_eq!(country_code(Some(" br ".into())), Some("BR".into()));
    }
