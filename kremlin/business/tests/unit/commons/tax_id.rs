    use super::*;

    #[test]
    fn accepts_a_valid_cpf_in_any_formatting() {
        assert_eq!(normalize("529.982.247-25").as_deref(), Some("52998224725"));
        assert_eq!(normalize("52998224725").as_deref(), Some("52998224725"));
        assert_eq!(
            normalize(" 529 982 247 25 ").as_deref(),
            Some("52998224725")
        );
    }

    #[test]
    fn rejects_a_cpf_with_a_wrong_check_digit() {
        // One digit off the valid number above.
        assert!(normalize("52998224726").is_none());
        assert!(normalize("12345678901").is_none());
    }

    #[test]
    fn rejects_repeated_digits_even_though_the_arithmetic_accepts_them() {
        // These satisfy the check-digit maths, which is exactly why they need
        // their own guard rather than being left to it.
        assert!(normalize("00000000000").is_none());
        assert!(normalize("11111111111").is_none());
        assert!(normalize("99999999999999").is_none());
    }

    #[test]
    fn accepts_a_valid_cnpj_and_rejects_a_broken_one() {
        assert_eq!(
            normalize("11.222.333/0001-81").as_deref(),
            Some("11222333000181")
        );
        assert!(normalize("11222333000182").is_none());
    }

    #[test]
    fn rejects_anything_that_is_not_a_document_length() {
        assert!(normalize("").is_none());
        assert!(normalize("529982247").is_none());
        assert!(normalize("529982247250").is_none());
        assert!(normalize("not a document").is_none());
    }
