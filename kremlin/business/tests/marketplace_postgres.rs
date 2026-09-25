//! Run only against a disposable database: KREMLIN_TEST_DATABASE_URL=... cargo test
//! -p business --test marketplace_postgres -- --ignored --nocapture
use business::{
    domain::{enums::Role, marketplace::*, user::User},
    gateway::purchase_gateway::PurchaseGateway,
    use_cases::purchase_use_case::PurchaseUseCase,
};
use entity::audit_entity::{AuditUser, run_with_user};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, Statement};

async fn database() -> (DbConn, DbConn, String) {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")
        .expect("explicit disposable database URL required");
    assert!(
        url.split('?').next().unwrap().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await.unwrap();
    let schema = format!("marketplace_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options
        .set_schema_search_path(&schema)
        .sqlx_logging(false)
        .max_connections(8);
    let db = Database::connect(options).await.unwrap();
    (root, db, schema)
}
async fn cleanup(root: DbConn, db: DbConn, schema: String) {
    db.close().await.unwrap();
    assert!(
        schema.starts_with("marketplace_test_")
            && schema
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    root.close().await.unwrap();
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
fn user(id: i64, role: Role, tenant_id: Option<i64>) -> User {
    User {
        id: Some(id),
        uuid: None,
        email: format!("user{id}@test.local"),
        name: Some("Buyer".into()),
        password: String::new(),
        enabled: true,
        first_login: false,
        tenant_id,
        role,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    }
}
fn input() -> CreatePurchaseInput {
    CreatePurchaseInput {
        items: (1..=5)
            .map(|sku_id| PurchaseItemInput {
                sku_id,
                quantity: 1,
            })
            .collect(),
        shipping_address: AddressInput {
            recipient: "Buyer".into(),
            address_line1: "Street 1".into(),
            address_line2: None,
            locality: "City".into(),
            administrative_area: "SP".into(),
            postal_code: "01001000".into(),
            country_code: "BR".into(),
        },
        billing_address: None,
    }
}
fn filter() -> OrderFilter {
    OrderFilter {
        offset: 0,
        limit: 25,
        query: None,
        status: None,
        tenant_id: None,
        customer_id: None,
        sort_by: "id".into(),
        descending: false,
    }
}
async fn fixtures(db: &DbConn) {
    db.execute_unprepared(r#"
INSERT INTO "user" (id, uuid, name, email, password, first_login, enabled, role, created_at, updated_at)
SELECT n, gen_random_uuid(), 'Buyer', 'user' || n || '@test.local', 'unused', false, true, 'Customer', now(), now() FROM generate_series(1,2) n;
INSERT INTO customer (id, uuid, user_id, name, email, cpf, marketing_consent, active, created_at, updated_at)
SELECT n, gen_random_uuid(), n, 'Buyer', 'user' || n || '@test.local', '12345678901', false, true, now(), now() FROM generate_series(1,2) n;
INSERT INTO tenant (id, uuid, business_name, tax_id, created_at, updated_at)
SELECT n, gen_random_uuid(), 'Seller ' || n, '12345678000100', now(), now() FROM generate_series(1,2) n;
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END, 'Product ' || n, 'product-' || n, true, '12345678', 0, now(), now() FROM generate_series(1,5) n;
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at)
SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END, n, 'SKU-' || n, 'variant-' || n, n * 100, true, now(), now() FROM generate_series(1,5) n;
INSERT INTO sku_stock (uuid, tenant_id, sku_id, quantity, reserved, created_at, updated_at)
SELECT gen_random_uuid(), tenant_id, id, 10, 0, now(), now() FROM sku;
"#).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn migration_roundtrip_and_global_customer_upgrade() {
    let (root, db, schema) = database().await;
    let base_count = Migrator::migrations()
        .iter()
        .position(|m| m.name() == "m20260924_000000_create_table_purchase")
        .unwrap() as u32;
    Migrator::up(&db, Some(base_count)).await.unwrap();
    // Emulate a deployed cart schema with the original seller/customer relation.
    db.execute_unprepared("CREATE UNIQUE INDEX legacy_customer_tenant_id ON customer(tenant_id,id); ALTER TABLE cart DROP CONSTRAINT fk_cart_customer; ALTER TABLE cart ADD CONSTRAINT fk_cart_customer FOREIGN KEY(tenant_id,customer_id) REFERENCES customer(tenant_id,id);").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await;
    db.execute_unprepared("INSERT INTO cart(uuid,tenant_id,customer_id,status,created_at,updated_at) VALUES(gen_random_uuid(),1,1,'open',now(),now());").await.unwrap();
    let new_count = Migrator::migrations().len() as u32 - base_count;
    Migrator::down(&db, Some(new_count)).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    assert_eq!(count(&db, "purchase").await, 0);
    assert_eq!(count(&db, "payment_allocation").await, 0);
    cleanup(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn split_purchase_retries_isolation_snapshots_and_atomicity() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await;
    let use_case = PurchaseUseCase::new(PurchaseGateway::new(db.clone()));
    let buyer = user(1, Role::Customer, None);
    // Match the authentication middleware's tenant-less customer audit scope.
    let created = run_with_user(
        Some(AuditUser {
            id: 1,
            email: buyer.email.clone(),
            tenant_id: None,
            enforce_tenant: true,
        }),
        use_case.create(&buyer, "first", input()),
    )
    .await
    .unwrap();
    assert!(!created.replayed);
    let detail = created.detail;
    assert_eq!(detail.purchase.total_cents, 1500);
    assert_eq!(detail.orders.len(), 2);
    assert_eq!(detail.orders[0].items.len(), 3);
    assert_eq!(detail.orders[1].items.len(), 2);
    assert_eq!(detail.orders[0].order.tenant_id, 1);
    assert_eq!(detail.orders[1].order.tenant_id, 2);
    assert_eq!(detail.orders[0].order.total_cents, 600);
    assert_eq!(detail.orders[1].order.total_cents, 900);
    assert_eq!(detail.payments.len(), 1);
    assert_eq!(detail.payments[0].payment.status, "pending_provider");
    assert!(detail.payments[0].payment.gateway_provider.is_none());
    assert!(detail.payments[0].card.is_none());
    for order in &detail.orders {
        assert_eq!(order.order.created_by.as_deref(), Some("user1@test.local"));
        assert_eq!(order.addresses.len(), 2);
        assert_eq!(order.allocations[0].amount_cents, order.order.total_cents);
        assert!(
            order
                .items
                .iter()
                .all(|i| i.cfop.is_none() && i.csosn.is_none() && i.tax_cents == 0)
        );
        let history = use_case.history(&buyer, order.order.id).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].actor_id, Some(1));
    }
    assert_eq!(count(&db, "payment_transaction").await, 0);
    assert_eq!(count(&db, "coupon_redemption").await, 0);
    let stock = db.query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT sum(quantity)::bigint AS quantity, sum(reserved)::bigint AS reserved FROM sku_stock".to_string())).await.unwrap().unwrap();
    assert_eq!(stock.try_get::<i64>("", "quantity").unwrap(), 50);
    assert_eq!(stock.try_get::<i64>("", "reserved").unwrap(), 0);

    let replay = use_case.create(&buyer, "first", input()).await.unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.detail.purchase.id, detail.purchase.id);
    let mut changed = input();
    changed.items[0].quantity = 2;
    assert!(matches!(
        use_case.create(&buyer, "first", changed).await,
        Err(PurchaseError::Conflict)
    ));
    let concurrent_input_a = input();
    let concurrent_input_b = input();
    let (a, b) = tokio::join!(
        use_case.create(&buyer, "concurrent", concurrent_input_a),
        use_case.create(&buyer, "concurrent", concurrent_input_b)
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_eq!(a.detail.purchase.id, b.detail.purchase.id);
    assert_ne!(a.replayed, b.replayed);
    assert_eq!(count(&db, "purchase").await, 2);

    let stranger = user(2, Role::Customer, None);
    assert!(matches!(
        use_case.get(&stranger, detail.purchase.id).await,
        Err(PurchaseError::NotFound)
    ));
    assert!(matches!(
        use_case.order(&stranger, detail.orders[0].order.id).await,
        Err(PurchaseError::NotFound)
    ));
    assert_eq!(
        use_case.orders(&stranger, &filter()).await.unwrap().total,
        0
    );
    let seller = user(3, Role::TenantOwner, Some(1));
    assert!(
        use_case
            .order(&seller, detail.orders[0].order.id)
            .await
            .is_ok()
    );
    assert!(matches!(
        use_case.order(&seller, detail.orders[1].order.id).await,
        Err(PurchaseError::NotFound)
    ));
    assert!(
        use_case
            .history(&seller, detail.orders[1].order.id)
            .await
            .is_err()
    );
    assert!(use_case.get(&seller, detail.purchase.id).await.is_err());
    assert!(
        use_case
            .transactions(&seller, detail.payments[0].payment.id)
            .await
            .is_err()
    );
    assert!(use_case.create(&seller, "seller", input()).await.is_err());
    assert_eq!(use_case.orders(&seller, &filter()).await.unwrap().total, 2);
    assert!(
        use_case
            .get(&user(4, Role::SysAdmin, None), detail.purchase.id)
            .await
            .is_ok()
    );

    // Failure after the first seller's records have been inserted must roll back
    // purchase, payment, orders, addresses, items, allocations and histories.
    db.execute_unprepared("ALTER TABLE orders ADD CONSTRAINT test_reject_second_seller CHECK (tenant_id <> 2) NOT VALID;").await.unwrap();
    assert!(matches!(
        use_case.create(&buyer, "rollback", input()).await,
        Err(PurchaseError::Persistence(_))
    ));
    assert_eq!(count(&db, "purchase").await, 2);
    assert_eq!(count(&db, "orders").await, 4);
    assert_eq!(count(&db, "order_item").await, 10);
    assert_eq!(count(&db, "order_address").await, 8);
    assert_eq!(count(&db, "payment").await, 2);
    assert_eq!(count(&db, "payment_allocation").await, 4);
    assert_eq!(count(&db, "order_status_history").await, 4);
    db.execute_unprepared("ALTER TABLE orders DROP CONSTRAINT test_reject_second_seller;")
        .await
        .unwrap();

    // Database constraints protect relationships even outside the use case.
    assert!(
        db.execute_unprepared(&format!(
            "UPDATE order_item SET tenant_id=2 WHERE order_id={}",
            detail.orders[0].order.id
        ))
        .await
        .is_err()
    );
    assert!(
        db.execute_unprepared(&format!(
            "UPDATE payment_allocation SET purchase_id={} WHERE order_id={}",
            a.detail.purchase.id, detail.orders[0].order.id
        ))
        .await
        .is_err()
    );
    assert!(db.execute_unprepared(&format!("INSERT INTO payment SELECT nextval('payment_id_seq'),gen_random_uuid(),purchase_id,method,status,installments,amount_cents,currency,gateway_provider,gateway_reference,created_at,created_by,updated_at,updated_by FROM payment WHERE id={}",detail.payments[0].payment.id)).await.is_err());

    db.execute_unprepared("UPDATE customer SET name='Changed' WHERE id=1; UPDATE sku SET price_cents=1 WHERE id=1; UPDATE product SET name='Changed' WHERE id=1;").await.unwrap();
    let snapshot = use_case.get(&buyer, detail.purchase.id).await.unwrap();
    assert_eq!(snapshot.purchase.customer_name, "Buyer");
    assert_eq!(snapshot.orders[0].items[0].unit_price_cents, 100);
    assert_eq!(snapshot.orders[0].items[0].product_name, "Product 1");
    assert_eq!(
        use_case
            .create(&buyer, "first", input())
            .await
            .unwrap()
            .detail
            .purchase
            .total_cents,
        1500
    );

    db.execute_unprepared("UPDATE sku SET active=false WHERE id=1")
        .await
        .unwrap();
    assert!(matches!(
        use_case.create(&buyer, "inactive", input()).await,
        Err(PurchaseError::Validation(_))
    ));
    db.execute_unprepared(
        "UPDATE sku SET active=true WHERE id=1; UPDATE product SET active=false WHERE id=1",
    )
    .await
    .unwrap();
    assert!(matches!(
        use_case.create(&buyer, "inactive-product", input()).await,
        Err(PurchaseError::Validation(_))
    ));
    db.execute_unprepared(
        "UPDATE product SET active=true WHERE id=1; UPDATE customer SET cpf=NULL WHERE id=1",
    )
    .await
    .unwrap();
    assert!(matches!(
        use_case.create(&buyer, "missing-tax", input()).await,
        Err(PurchaseError::Validation(_))
    ));
    db.execute_unprepared(
        "UPDATE customer SET cpf='12345678901' WHERE id=1; UPDATE sku SET price_cents=2147483647",
    )
    .await
    .unwrap();
    let mut overflow = input();
    for item in &mut overflow.items {
        item.quantity = i32::MAX;
    }
    assert!(matches!(
        use_case.create(&buyer, "overflow", overflow).await,
        Err(PurchaseError::Validation(_))
    ));
    assert_eq!(count(&db, "purchase").await, 2);
    cleanup(root, db, schema).await;
}
