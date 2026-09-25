    use super::*;
    use chrono::Utc;
    use entity::tenant_entity;
    use sea_orm::{DatabaseBackend, DbErr, MockDatabase};
    use uuid::Uuid;

    fn build_test_tenant(business_name: &str, country_code: Option<&str>) -> Tenant {
        Tenant {
            id: None,
            uuid: None,
            business_name: business_name.to_string(),
            company_name: Some("Acme Corp".to_string()),
            tax_id: "12.345.678/0001-90".to_string(),
            email: Some("contact@acme.com".to_string()),
            phone: Some("11999999999".to_string()),
            web_site: Some("https://acme.com".to_string()),
            address_line1: Some("Av Paulista, 100".to_string()),
            address_line2: None,
            locality: Some("São Paulo".to_string()),
            administrative_area: Some("SP".to_string()),
            postal_code: Some("01310-100".to_string()),
            country_code: country_code.map(|s| s.to_string()),
            created_at: None,
            updated_at: None,
            created_by: None,
            updated_by: None,
        }
    }

    fn mock_tenant_model(id: i64, business_name: &str, country_code: Option<&str>) -> tenant_entity::Model {
        tenant_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            business_name: business_name.to_string(),
            company_name: Some("Acme Corp".to_string()),
            tax_id: "12.345.678/0001-90".to_string(),
            email: Some("contact@acme.com".to_string()),
            phone: Some("11999999999".to_string()),
            web_site: Some("https://acme.com".to_string()),
            address_line1: Some("Av Paulista, 100".to_string()),
            address_line2: None,
            locality: Some("São Paulo".to_string()),
            administrative_area: Some("SP".to_string()),
            postal_code: Some("01310-100".to_string()),
            country_code: country_code.map(|s| s.to_string()),
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        }
    }

    #[test]
    fn test_valid_country_code() {
        assert!(valid_country_code(&None));
        assert!(valid_country_code(&Some("BR".to_string())));
        assert!(valid_country_code(&Some("us".to_string())));
        assert!(!valid_country_code(&Some("BRA".to_string())));
        assert!(!valid_country_code(&Some("B".to_string())));
        assert!(!valid_country_code(&Some("B1".to_string())));
        assert!(!valid_country_code(&Some("12".to_string())));
    }

    #[tokio::test]
    async fn test_create_tenant_validation() {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let use_case = TenantUseCase::new(TenantGateway::new(db));

        // Empty business name
        let t1 = build_test_tenant("", Some("BR"));
        assert!(use_case.create(t1).await.is_none());

        // Invalid country code
        let t2 = build_test_tenant("Acme Store", Some("BRA"));
        assert!(use_case.create(t2).await.is_none());
    }

    #[tokio::test]
    async fn test_create_and_persist_tenant_success_and_db_error() {
        let t = build_test_tenant("Acme Store", Some("BR"));
        let model = mock_tenant_model(1, "Acme Store", Some("BR"));

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model.clone()]])
            .into_connection();
        let use_case = TenantUseCase::new(TenantGateway::new(db_ok));
        let created = use_case.create(t.clone()).await.expect("tenant created");
        assert_eq!(created.id, Some(1));
        assert_eq!(created.business_name, "Acme Store");

        // Persist alias
        let db_persist = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case_persist = TenantUseCase::new(TenantGateway::new(db_persist));
        let persisted = use_case_persist.persist(t.clone()).await.expect("tenant persisted");
        assert_eq!(persisted.id, Some(1));

        // DB failure
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Insert failed".to_string())])
            .into_connection();
        let use_case_err = TenantUseCase::new(TenantGateway::new(db_err));
        assert!(use_case_err.create(t).await.is_none());
    }

    #[tokio::test]
    async fn test_find_by_id_found_not_found_error() {
        let model = mock_tenant_model(10, "Acme 10", Some("BR"));

        // Found
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = TenantUseCase::new(TenantGateway::new(db_ok));
        let found = use_case.find_by_id(10).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().business_name, "Acme 10");

        // Not found
        let db_nf = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<tenant_entity::Model>::new()])
            .into_connection();
        let use_case_nf = TenantUseCase::new(TenantGateway::new(db_nf));
        assert!(use_case_nf.find_by_id(999).await.is_none());

        // Error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("DB down".to_string())])
            .into_connection();
        let use_case_err = TenantUseCase::new(TenantGateway::new(db_err));
        assert!(use_case_err.find_by_id(10).await.is_none());
    }

    #[tokio::test]
    async fn test_find_by_uuid_found_not_found_error() {
        let model = mock_tenant_model(20, "Acme UUID", Some("US"));
        let uuid_str = model.uuid.to_string();

        // Found
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = TenantUseCase::new(TenantGateway::new(db_ok));
        let found = use_case.find_by_uuid(uuid_str.clone()).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, Some(20));

        // Not found
        let db_nf = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<tenant_entity::Model>::new()])
            .into_connection();
        let use_case_nf = TenantUseCase::new(TenantGateway::new(db_nf));
        assert!(use_case_nf.find_by_uuid(Uuid::new_v4().to_string()).await.is_none());

        // Error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Error".to_string())])
            .into_connection();
        let use_case_err = TenantUseCase::new(TenantGateway::new(db_err));
        assert!(use_case_err.find_by_uuid(uuid_str).await.is_none());
    }

    #[tokio::test]
    async fn test_find_all_tenants() {
        let model1 = mock_tenant_model(1, "Tenant 1", Some("BR"));
        let model2 = mock_tenant_model(2, "Tenant 2", Some("US"));

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model1, model2]])
            .into_connection();
        let use_case = TenantUseCase::new(TenantGateway::new(db_ok));
        let list = use_case.find_all().await;
        assert_eq!(list.len(), 2);

        // Error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Error".to_string())])
            .into_connection();
        let use_case_err = TenantUseCase::new(TenantGateway::new(db_err));
        assert!(use_case_err.find_all().await.is_empty());
    }

    #[tokio::test]
    async fn test_update_tenant_all_cases() {
        let existing = mock_tenant_model(5, "Old Business", Some("BR"));

        // Case 1: Existing tenant not found
        let db_nf = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<tenant_entity::Model>::new()])
            .into_connection();
        let uc_nf = TenantUseCase::new(TenantGateway::new(db_nf));
        assert!(uc_nf.update(5, build_test_tenant("New", Some("BR"))).await.is_none());

        // Case 2: Missing company name
        let db_no_company = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing.clone()]])
            .into_connection();
        let uc_no_company = TenantUseCase::new(TenantGateway::new(db_no_company));
        let mut t_no_comp = build_test_tenant("New", Some("BR"));
        t_no_comp.company_name = None;
        assert!(uc_no_company.update(5, t_no_comp).await.is_none());

        // Case 3: Empty whitespace company name
        let db_ws_company = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing.clone()]])
            .into_connection();
        let uc_ws_company = TenantUseCase::new(TenantGateway::new(db_ws_company));
        let mut t_ws_comp = build_test_tenant("New", Some("BR"));
        t_ws_comp.company_name = Some("   ".to_string());
        assert!(uc_ws_company.update(5, t_ws_comp).await.is_none());

        // Case 4: Invalid country code
        let db_inv_cc = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing.clone()]])
            .into_connection();
        let uc_inv_cc = TenantUseCase::new(TenantGateway::new(db_inv_cc));
        let t_inv_cc = build_test_tenant("New", Some("BRA"));
        assert!(uc_inv_cc.update(5, t_inv_cc).await.is_none());

        // Case 5: Persist error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing.clone()]])
            .append_query_errors([DbErr::Custom("Update failed".to_string())])
            .into_connection();
        let uc_err = TenantUseCase::new(TenantGateway::new(db_err));
        assert!(uc_err.update(5, build_test_tenant("New", Some("BR"))).await.is_none());

        // Case 6: Success
        let updated_model = mock_tenant_model(5, "Updated Business", Some("BR"));
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing]])
            .append_query_results([vec![updated_model]])
            .into_connection();
        let uc_ok = TenantUseCase::new(TenantGateway::new(db_ok));
        let res = uc_ok.update(5, build_test_tenant("Updated Business", Some("BR"))).await;
        assert!(res.is_some());
        assert_eq!(res.unwrap().business_name, "Updated Business");
    }
