//! Run only against a disposable database: KREMLIN_TEST_DATABASE_URL=... cargo test
//! -p business --test cart_cleanup_postgres -- --ignored --nocapture
use business::gateway::cart_gateway::CartGateway;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, Statement, TransactionTrait,
};
use std::time::{Duration, Instant};

async fn setup() -> (DbConn, DbConn, String) {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")
        .expect("explicit disposable database URL required");
    assert!(
        url.split('?').next().unwrap().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await.unwrap();
    let schema = format!("cart_cleanup_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options
        .set_schema_search_path(&schema)
        .sqlx_logging(false)
        .max_connections(8);
    let db = Database::connect(options).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        r#"
INSERT INTO "user" (id, uuid, name, email, password, first_login, enabled, role, created_at, updated_at)
SELECT n, gen_random_uuid(), 'Buyer', 'user' || n || '@test.local', 'unused', false, true, 'Customer', now(), now() FROM generate_series(1,2) n;
INSERT INTO customer (id, uuid, user_id, name, email, cpf, marketing_consent, active, created_at, updated_at)
SELECT n, gen_random_uuid(), n, 'Buyer', 'user' || n || '@test.local', '12345678901', false, true, now(), now() FROM generate_series(1,2) n;
INSERT INTO tenant (id, uuid, business_name, tax_id, created_at, updated_at)
SELECT n, gen_random_uuid(), 'Seller ' || n, '12345678000100', now(), now() FROM generate_series(1,2) n;
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
SELECT n, gen_random_uuid(), n, 'Product ' || n, 'product-' || n, true, '12345678', 0, now(), now() FROM generate_series(1,2) n;
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at)
SELECT n, gen_random_uuid(), n, n, 'SKU-' || n, 'v' || n, 100, true, now(), now() FROM generate_series(1,2) n;
"#,
    )
    .await
    .unwrap();
    (root, db, schema)
}

async fn teardown(root: DbConn, db: DbConn, schema: String) {
    db.close().await.unwrap();
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}

/// A cart of `customer` for `tenant` (None: marketplace) holding one item; returns the cart id.
async fn cart(db: &DbConn, customer: i64, tenant: Option<i64>) -> i64 {
    let tenant_sql = tenant.map_or("NULL".to_string(), |t| t.to_string());
    let sku = tenant.unwrap_or(1);
    let row = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "INSERT INTO cart (uuid, tenant_id, customer_id, status, created_at, updated_at) \
                 VALUES (gen_random_uuid(), {tenant_sql}, {customer}, 'active', now(), now()) RETURNING id"
            ),
        ))
        .await
        .unwrap()
        .unwrap();
    let id: i64 = row.try_get("", "id").unwrap();
    db.execute_unprepared(&format!(
        "INSERT INTO cart_item (uuid, tenant_id, cart_id, sku_id, quantity, unit_price_cents, created_at, updated_at) \
         VALUES (gen_random_uuid(), {tenant_sql}, {id}, {sku}, 1, 100, now(), now())"
    ))
    .await
    .unwrap();
    id
}

async fn carts(db: &DbConn) -> Vec<(i64, i64, Option<i64>)> {
    db.query_all_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT id, customer_id, tenant_id FROM cart ORDER BY id".to_string(),
    ))
    .await
    .unwrap()
    .into_iter()
    .map(|r| {
        (
            r.try_get("", "id").unwrap(),
            r.try_get("", "customer_id").unwrap(),
            r.try_get("", "tenant_id").unwrap(),
        )
    })
    .collect()
}

async fn count(db: &DbConn, table: &str) -> i64 {
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

async fn lock_timeout(tx: &sea_orm::DatabaseTransaction) -> String {
    tx.query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT current_setting('lock_timeout') AS v".to_string(),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "v")
    .unwrap()
}

async fn purge(db: &DbConn, customer: i64, tenants: &[i64]) {
    let tx = db.begin().await.unwrap();
    CartGateway::purge_purchased(&tx, 1, customer, tenants).await;
    tx.commit().await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn store_purchase_deletes_only_the_customers_cart_of_that_tenant() {
    let (root, db, schema) = setup().await;
    let target = cart(&db, 1, Some(1)).await;
    let other_tenant = cart(&db, 1, Some(2)).await;
    let marketplace = cart(&db, 1, None).await;
    let other_customer = cart(&db, 2, Some(1)).await;

    purge(&db, 1, &[1]).await;

    let left: Vec<i64> = carts(&db).await.into_iter().map(|c| c.0).collect();
    assert!(!left.contains(&target));
    assert_eq!(left, vec![other_tenant, marketplace, other_customer]);
    // the items of the deleted cart went with it, the others stayed
    assert_eq!(count(&db, "cart_item").await, 3);
    teardown(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn marketplace_purchase_with_several_tenants_deletes_the_tenantless_cart() {
    let (root, db, schema) = setup().await;
    let marketplace = cart(&db, 1, None).await;
    let store = cart(&db, 1, Some(1)).await;

    purge(&db, 1, &[1, 2, 2]).await;

    let left: Vec<i64> = carts(&db).await.into_iter().map(|c| c.0).collect();
    assert!(!left.contains(&marketplace));
    assert_eq!(left, vec![store]);
    assert_eq!(count(&db, "cart_item").await, 1);
    teardown(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn single_seller_marketplace_purchase_leaves_the_tenantless_cart_and_no_orders_delete_nothing() {
    let (root, db, schema) = setup().await;
    let marketplace = cart(&db, 1, None).await;

    purge(&db, 1, &[1]).await; // looks like a store purchase of tenant 1 (Q1)
    purge(&db, 1, &[]).await; // no orders
    purge(&db, 1, &[1]).await; // repeated: nothing more to delete, no error

    let left: Vec<i64> = carts(&db).await.into_iter().map(|c| c.0).collect();
    assert_eq!(left, vec![marketplace]);
    teardown(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn a_cleanup_failure_does_not_roll_back_the_purchase() {
    let (root, db, schema) = setup().await;
    let target = cart(&db, 1, Some(1)).await;
    db.execute_unprepared(
        "CREATE FUNCTION no_cart_delete() RETURNS trigger LANGUAGE plpgsql AS \
         $$ BEGIN RAISE EXCEPTION 'cart delete refused'; END $$; \
         CREATE TRIGGER no_cart_delete BEFORE DELETE ON cart FOR EACH ROW EXECUTE FUNCTION no_cart_delete();",
    )
    .await
    .unwrap();

    let tx = db.begin().await.unwrap();
    tx.execute_unprepared("UPDATE tenant SET business_name = 'paid' WHERE id = 1")
        .await
        .unwrap();
    CartGateway::purge_purchased(&tx, 1, 1, &[1]).await;
    // the transaction is still usable and commits
    tx.execute_unprepared("UPDATE tenant SET tax_id = 'after' WHERE id = 1")
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let name = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT business_name, tax_id FROM tenant WHERE id = 1".to_string(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(name.try_get::<String>("", "business_name").unwrap(), "paid");
    assert_eq!(name.try_get::<String>("", "tax_id").unwrap(), "after");
    let left: Vec<i64> = carts(&db).await.into_iter().map(|c| c.0).collect();
    assert_eq!(left, vec![target]);
    teardown(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn a_locked_cart_holds_the_payment_only_for_the_bounded_wait_and_the_timeout_is_restored() {
    let (root, db, schema) = setup().await;
    let target = cart(&db, 1, Some(1)).await;

    // another session keeps a row lock on the cart
    let holder = db.begin().await.unwrap();
    holder
        .execute_unprepared(&format!("SELECT id FROM cart WHERE id = {target} FOR UPDATE"))
        .await
        .unwrap();

    let tx = db.begin().await.unwrap();
    let before = lock_timeout(&tx).await;
    let started = Instant::now();
    CartGateway::purge_purchased(&tx, 1, 1, &[1]).await;
    let waited = started.elapsed();
    // the transaction is alive and its lock timeout is what it was
    assert_eq!(lock_timeout(&tx).await, before);
    tx.commit().await.unwrap();
    holder.rollback().await.unwrap();

    assert!(waited >= Duration::from_millis(1500), "waited {waited:?}");
    assert!(waited < Duration::from_secs(6), "waited {waited:?}");
    let left: Vec<i64> = carts(&db).await.into_iter().map(|c| c.0).collect();
    assert_eq!(left, vec![target]);
    teardown(root, db, schema).await;
}
