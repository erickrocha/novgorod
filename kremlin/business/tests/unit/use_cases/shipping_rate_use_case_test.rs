use super::*;
use chrono::Utc;
use entity::shipping_rate_entity;
use sea_orm::{DatabaseBackend, DbErr, MockDatabase, MockExecResult};
use uuid::Uuid;

fn mock_shipping_rate_model(id: i64, uf: &str, price_cents: i32) -> shipping_rate_entity::Model {
    shipping_rate_entity::Model {
        id,
        uuid: Uuid::new_v4(),
        tenant_id: Some(1),
        uf: uf.to_string(),
        price_cents,
        created_at: Utc::now().naive_utc(),
        created_by: Some("admin".to_string()),
        updated_at: Utc::now().naive_utc(),
        updated_by: Some("admin".to_string()),
    }
}

fn build_shipping_rate(uf: &str, price_cents: i32) -> ShippingRate {
    ShippingRate {
        id: None,
        uuid: None,
        tenant_id: Some(1),
        uf: uf.to_string(),
        price_cents,
        created_at: None,
        created_by: Some("admin".to_string()),
        updated_at: None,
        updated_by: Some("admin".to_string()),
    }
}

#[tokio::test]
async fn test_create_shipping_rate_success_and_error() {
    let model = mock_shipping_rate_model(1, "SP", 1500);
    let rate = build_shipping_rate("SP", 1500);

    // Success
    let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![model]])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db_ok));
    let created = use_case.create(rate.clone()).await;
    assert!(created.is_some());
    let created = created.unwrap();
    assert_eq!(created.id, Some(1));
    assert_eq!(created.uf, "SP");
    assert_eq!(created.price_cents, 1500);

    // Error
    let db_err = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_errors([DbErr::Custom("Insert error".to_string())])
        .into_connection();
    let use_case_err = ShippingRateUseCase::new(ShippingRateGateway::new(db_err));
    assert!(use_case_err.create(rate).await.is_none());
}

#[tokio::test]
async fn test_find_all_shipping_rates() {
    let m1 = mock_shipping_rate_model(1, "SP", 1500);
    let m2 = mock_shipping_rate_model(2, "RJ", 2000);

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![m1, m2]])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db));
    let rates = use_case.find_all().await;
    assert_eq!(rates.len(), 2);
    assert_eq!(rates[0].uf, "SP");
    assert_eq!(rates[1].uf, "RJ");
}

#[tokio::test]
async fn test_find_by_id() {
    let model = mock_shipping_rate_model(42, "MG", 1800);

    // Found
    let db_found = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![model]])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db_found));
    let found = use_case.find_by_id(42).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().uf, "MG");

    // Not found
    let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<shipping_rate_entity::Model>::new()])
        .into_connection();
    let use_case_nf = ShippingRateUseCase::new(ShippingRateGateway::new(db_not_found));
    assert!(use_case_nf.find_by_id(99).await.is_none());
}

#[tokio::test]
async fn test_find_by_uuid() {
    let model = mock_shipping_rate_model(10, "RS", 2500);
    let uuid_str = model.uuid.to_string();

    // Found
    let db_found = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![model]])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db_found));
    let found = use_case.find_by_uuid(uuid_str.clone()).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().uf, "RS");

    // Not found
    let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<shipping_rate_entity::Model>::new()])
        .into_connection();
    let use_case_nf = ShippingRateUseCase::new(ShippingRateGateway::new(db_not_found));
    assert!(use_case_nf.find_by_uuid(uuid_str).await.is_none());
}

#[tokio::test]
async fn test_update_shipping_rate() {
    let model = mock_shipping_rate_model(5, "PR", 1700);
    let rate = build_shipping_rate("PR", 1700);

    let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![model]])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db_ok));
    let updated = use_case.update(5, rate.clone()).await;
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().id, Some(5));

    // Error
    let db_err = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_errors([DbErr::Custom("Update error".to_string())])
        .into_connection();
    let use_case_err = ShippingRateUseCase::new(ShippingRateGateway::new(db_err));
    assert!(use_case_err.update(5, rate).await.is_none());
}

#[tokio::test]
async fn test_delete_by_id() {
    // Success
    let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let use_case = ShippingRateUseCase::new(ShippingRateGateway::new(db_ok));
    assert!(use_case.delete_by_id(1).await.is_some());

    // Error
    let db_err = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_errors([DbErr::Custom("Delete error".to_string())])
        .into_connection();
    let use_case_err = ShippingRateUseCase::new(ShippingRateGateway::new(db_err));
    assert!(use_case_err.delete_by_id(1).await.is_none());
}
