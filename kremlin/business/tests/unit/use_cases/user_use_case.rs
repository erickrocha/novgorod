    use super::*;
    use crate::domain::enums::Role;

    #[test]
    fn test_build_person_for_user_with_name() {
        let user = User {
            id: Some(42),
            uuid: None,
            email: "john.doe@example.com".to_string(),
            name: Some("John Doe".to_string()),
            password: "secret".to_string(),
            enabled: true,
            first_login: false,
            tenant_id: Some(7),
            role: Role::TenantUser,
            created_at: None,
            created_by: Some("admin".to_string()),
            updated_at: None,
            updated_by: None,
        };

        let person = UserUseCase::build_person_for_user(&user).expect("person should be built");
        assert_eq!(person.user_id, 42);
        assert_eq!(person.tenant_id, Some(7));
        assert_eq!(person.first_name, "John Doe");
        assert_eq!(person.surname, None);
        assert_eq!(person.date_of_birth, None);
        assert_eq!(person.gender, None);
        assert_eq!(person.email, None);
        assert_eq!(person.created_by, Some("admin".to_string()));
    }

    #[test]
    fn test_build_person_for_user_fallback_to_email_prefix() {
        let user = User {
            id: Some(10),
            uuid: None,
            email: "alice@company.com".to_string(),
            name: None,
            password: "secret".to_string(),
            enabled: true,
            first_login: false,
            tenant_id: None,
            role: Role::TenantUser,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };

        let person = UserUseCase::build_person_for_user(&user).expect("person should be built");
        assert_eq!(person.user_id, 10);
        assert_eq!(person.tenant_id, None);
        assert_eq!(person.first_name, "alice");
        assert_eq!(person.surname, None);
        assert_eq!(person.date_of_birth, None);
        assert_eq!(person.gender, None);
        assert_eq!(person.email, None);
    }

    #[test]
    fn test_build_person_for_user_empty_whitespace_name_fallback() {
        let user = User {
            id: Some(10),
            uuid: None,
            email: "bob@company.com".to_string(),
            name: Some("   ".to_string()),
            password: "secret".to_string(),
            enabled: true,
            first_login: false,
            tenant_id: None,
            role: Role::TenantUser,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };

        let person = UserUseCase::build_person_for_user(&user).expect("person should be built");
        assert_eq!(person.first_name, "bob");
    }

    #[test]
    fn test_build_person_for_user_without_id_returns_none() {
        let user = User {
            id: None,
            uuid: None,
            email: "test@example.com".to_string(),
            name: Some("Test".to_string()),
            password: "secret".to_string(),
            enabled: true,
            first_login: false,
            tenant_id: None,
            role: Role::TenantUser,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };

        assert!(UserUseCase::build_person_for_user(&user).is_none());
    }
