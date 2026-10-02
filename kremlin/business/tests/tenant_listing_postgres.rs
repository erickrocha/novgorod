//! Run only against a disposable database: KREMLIN_TEST_DATABASE_URL=... cargo test
//! -p business --test tenant_listing_postgres -- --ignored --nocapture
use business::commons::gateway::Gateway;
use business::gateway::tenant_gateway::TenantGateway;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database};

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn listing_flag_defaults_to_off_and_is_set_only_through_set_listed() {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")
        .expect("explicit disposable database URL required");
    assert!(
        url.split('?').next().unwrap().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await.unwrap();
    let schema = format!("tenant_listing_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    db.execute_unprepared(
        "INSERT INTO tenant (uuid, business_name, tax_id, created_at, updated_at) VALUES (gen_random_uuid(), 'T', '1', now(), now())",
    )
    .await
    .unwrap();
    let gateway = TenantGateway::new(db.clone());
    let id = gateway.find_all().await.unwrap()[0].id;

    // fail closed for new and existing tenants (NOV-PLT-017)
    assert!(!gateway.find_by_id(id).await.unwrap().unwrap().listed);
    assert_eq!(gateway.set_listed(id, true).await.unwrap(), 1);
    assert!(gateway.find_by_id(id).await.unwrap().unwrap().listed);
    assert_eq!(gateway.set_listed(id, false).await.unwrap(), 1);
    assert!(!gateway.find_by_id(id).await.unwrap().unwrap().listed);
    // an unknown tenant changes nothing
    assert_eq!(gateway.set_listed(id + 1000, true).await.unwrap(), 0);

    db.close().await.unwrap();
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}
