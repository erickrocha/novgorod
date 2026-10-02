//! Run only against a disposable database: KREMLIN_TEST_DATABASE_URL=... cargo test
//! -p business --test cart_tenant_optional_postgres -- --ignored --nocapture
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, Statement};

async fn count(db: &sea_orm::DbConn, table: &str) -> i64 {
    db.query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT count(*) AS n FROM {table}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn marketplace_cart_has_no_tenant_and_still_cascades_and_stays_unique() {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")
        .expect("explicit disposable database URL required");
    assert!(
        url.split('?').next().unwrap().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await.unwrap();
    let schema = format!("cart_tenant_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    // a tenant, a product and a SKU to reference
    db.execute_unprepared(
        "INSERT INTO tenant (uuid, business_name, tax_id, created_at, updated_at) \
         VALUES (gen_random_uuid(), 'T', '1', now(), now())",
    )
    .await
    .unwrap();
    let tenant_id: i64 = db
        .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT id FROM tenant".to_string()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "id")
        .unwrap();
    db.execute_unprepared(&format!(
        "INSERT INTO product (uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at) \
         VALUES (gen_random_uuid(), {tenant_id}, 'P', 'p', true, '12345678', 0, now(), now())"
    ))
    .await
    .unwrap();
    db.execute_unprepared(&format!(
        "INSERT INTO sku (uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at) \
         SELECT gen_random_uuid(), {tenant_id}, id, 'S1', 'v1', 100, true, now(), now() FROM product"
    ))
    .await
    .unwrap();

    // a cart and an item without tenant (marketplace)
    db.execute_unprepared(
        "INSERT INTO cart (uuid, tenant_id, status, created_at, updated_at) \
         VALUES (gen_random_uuid(), NULL, 'active', now(), now())",
    )
    .await
    .unwrap();
    let item = "INSERT INTO cart_item (uuid, tenant_id, cart_id, sku_id, quantity, unit_price_cents, \
                created_at, updated_at) SELECT gen_random_uuid(), NULL, c.id, s.id, 1, 100, now(), now() \
                FROM cart c, sku s";
    db.execute_unprepared(item).await.unwrap();
    // the same SKU twice in one cart is refused even without a tenant
    assert!(db.execute_unprepared(item).await.is_err());
    // an unknown SKU is refused even without a tenant
    assert!(db
        .execute_unprepared(
            "INSERT INTO cart_item (uuid, tenant_id, cart_id, sku_id, quantity, unit_price_cents, \
             created_at, updated_at) SELECT gen_random_uuid(), NULL, id, 999999, 1, 100, now(), now() FROM cart",
        )
        .await
        .is_err());
    // deleting the cart removes its items
    db.execute_unprepared("DELETE FROM cart").await.unwrap();
    assert_eq!(count(&db, "cart_item").await, 0);

    db.close().await.unwrap();
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}
