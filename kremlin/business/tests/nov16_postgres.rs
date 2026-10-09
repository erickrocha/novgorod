//! NOV-16 (C-019): purchase tenant binding and fixed-mode split shipping. Run only against a
//! disposable database: KREMLIN_TEST_DATABASE_URL=.../x_test cargo test -p business --test
//! nov16_postgres -- --ignored
use business::{
    domain::{enums::Role, marketplace::*, shipping::ShippingSnapshot, user::User},
    gateway::purchase_gateway::PurchaseGateway,
    use_cases::{
        checkout_quote_use_case::{CheckoutQuoteUseCase, QuoteRequest, ShippingSelection},
        purchase_use_case::{CheckoutPurchaseInput, PurchaseUseCase},
    },
};
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
    assert!(schema.starts_with("marketplace_test_"));
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    root.close().await.unwrap();
}
async fn scalar(db: &DbConn, sql: &str) -> i64 {
    db.query_one_raw(Statement::from_string(DbBackend::Postgres, sql.to_string()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}
fn buyer() -> User {
    User {
        id: Some(1),
        uuid: None,
        email: "user1@test.local".into(),
        name: Some("Buyer".into()),
        password: String::new(),
        enabled: true,
        first_login: false,
        tenant_id: None,
        role: Role::Customer,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    }
}
fn request(items: &[(i64, i32)]) -> QuoteRequest {
    QuoteRequest {
        items: items
            .iter()
            .map(|(sku_id, quantity)| PurchaseItemInput {
                sku_id: *sku_id,
                quantity: *quantity,
            })
            .collect(),
        address_id: None,
        shipping_address: Some(AddressInput {
            recipient: "Buyer".into(),
            address_line1: "Street 1".into(),
            address_line2: None,
            locality: "City".into(),
            administrative_area: "SP".into(),
            postal_code: "01001000".into(),
            country_code: "BR".into(),
        }),
        coupons: vec![],
    }
}
fn checkout(quote_id: i64, tenant_id: Option<i64>) -> CheckoutPurchaseInput {
    CheckoutPurchaseInput {
        quote_id,
        email: "buyer@test.local".into(),
        phone: "11999999999".into(),
        tenant_id,
    }
}
async fn fixtures(db: &DbConn) {
    db.execute_unprepared(r#"
INSERT INTO "user" (id, uuid, name, email, password, first_login, enabled, role, created_at, updated_at)
VALUES (1, gen_random_uuid(), 'Buyer', 'user1@test.local', 'unused', false, true, 'Customer', now(), now());
INSERT INTO customer (id, uuid, user_id, name, email, cpf, marketing_consent, active, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 'Buyer', 'user1@test.local', '12345678901', false, true, now(), now());
INSERT INTO tenant (id, uuid, business_name, tax_id, created_at, updated_at)
SELECT n, gen_random_uuid(), 'Seller ' || n, '12345678000100', now(), now() FROM generate_series(1,2) n;
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END, 'Product ' || n, 'product-' || n, true, '12345678', 0, now(), now() FROM generate_series(1,5) n;
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at)
SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END, n, 'SKU-' || n, 'variant-' || n, n * 100, true, now(), now() FROM generate_series(1,5) n;
INSERT INTO sku_stock (uuid, tenant_id, sku_id, quantity, reserved, created_at, updated_at)
SELECT gen_random_uuid(), tenant_id, id, 10, 0, now(), now() FROM sku;
INSERT INTO shipping_rate(uuid,tenant_id,uf,price_cents,created_at,updated_at) SELECT gen_random_uuid(),id,'SP',0,now(),now() FROM tenant;
"#).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn purchase_tenant_binding_scoped_reads_and_backfill() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await;
    let quotes = CheckoutQuoteUseCase::new(db.clone());
    let purchases = PurchaseUseCase::new(PurchaseGateway::new(db.clone()));
    let buyer = buyer();

    // SR-MKT-007: a tenant-bound purchase refuses another seller's items before any insert.
    let mixed = quotes
        .create(1, request(&[(1, 1), (4, 1)]))
        .await
        .unwrap();
    assert!(matches!(
        purchases
            .create_checkout(&buyer, "mixed", checkout(mixed.id, Some(1)))
            .await,
        Err(PurchaseError::Validation(_))
    ));
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM purchase").await, 0);

    // SR-MKT-005/006: the marketplace purchase has no tenant, the bound one records it.
    let marketplace = purchases
        .create_checkout(&buyer, "mkt", checkout(mixed.id, None))
        .await
        .unwrap();
    assert_eq!(marketplace.detail.purchase.tenant_id, None);
    assert_eq!(marketplace.detail.orders.len(), 2);
    let single = quotes.create(1, request(&[(1, 1)])).await.unwrap();
    let bound = purchases
        .create_checkout(&buyer, "bound", checkout(single.id, Some(1)))
        .await
        .unwrap();
    assert_eq!(bound.detail.purchase.tenant_id, Some(1));
    let (mkt_id, bound_id) = (
        marketplace.detail.purchase.id,
        bound.detail.purchase.id,
    );

    // SR-MKT-008: other tenants' and marketplace purchases are not found in a bound context.
    assert!(matches!(
        purchases.get_scoped(&buyer, bound_id, Some(2)).await,
        Err(PurchaseError::NotFound)
    ));
    assert!(matches!(
        purchases.get_scoped(&buyer, mkt_id, Some(1)).await,
        Err(PurchaseError::NotFound)
    ));
    purchases.get_scoped(&buyer, bound_id, Some(1)).await.unwrap();
    purchases.get_scoped(&buyer, mkt_id, None).await.unwrap();
    let filter = |tenant_id| OrderFilter {
        offset: 0,
        limit: 50,
        query: None,
        status: None,
        tenant_id,
        customer_id: None,
        sort_by: "id".into(),
        descending: false,
    };
    assert_eq!(
        purchases.purchases(&buyer, &filter(Some(2))).await.unwrap().total,
        0
    );
    assert_eq!(
        purchases.purchases(&buyer, &filter(Some(1))).await.unwrap().total,
        1
    );
    assert_eq!(
        purchases.purchases(&buyer, &filter(None)).await.unwrap().total,
        2
    );

    // The migration backfills single-seller history only; the rollback drops the column.
    Migrator::down(&db, Some(1)).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM purchase WHERE tenant_id=1").await,
        1
    );
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM purchase WHERE tenant_id IS NULL").await,
        1
    );
    cleanup(root, db, schema).await;
}

async fn fixed_fixtures(db: &DbConn) {
    fixtures(db).await;
    db.execute_unprepared(r#"
DELETE FROM sku_stock WHERE tenant_id=1;
DELETE FROM shipping_rate WHERE tenant_id=1;
INSERT INTO warehouse(id,uuid,tenant_id,name,origin_cep,city,uf,is_default,active,created_at,updated_at) VALUES
 (1,gen_random_uuid(),1,'W1','01001000','City','SP',true,true,now(),now()),
 (2,gen_random_uuid(),1,'W2','02002000','City','SP',false,true,now(),now()),
 (3,gen_random_uuid(),1,'W3','03003000','City','SP',false,false,now(),now());
INSERT INTO sku_stock(uuid,tenant_id,sku_id,warehouse_id,quantity,reserved,created_at,updated_at) VALUES
 (gen_random_uuid(),1,1,1,3,0,now(),now()),
 (gen_random_uuid(),1,2,2,3,0,now(),now()),
 (gen_random_uuid(),1,3,3,9,0,now(),now());
INSERT INTO shipping_rate(uuid,tenant_id,origin_warehouse_id,uf,price_cents,transit_days_min,transit_days_max,created_at,updated_at) VALUES
 (gen_random_uuid(),1,1,'SP',1000,1,2,now(),now()),
 (gen_random_uuid(),1,2,'SP',2000,3,6,now(),now());
"#).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn fixed_mode_split_option_selection_and_purchase_total() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixed_fixtures(&db).await;
    let quotes = CheckoutQuoteUseCase::new(db.clone());
    let purchases = PurchaseUseCase::new(PurchaseGateway::new(db.clone()));
    let cart = request(&[(1, 3), (2, 3)]);

    // TC-17 steps 1-2: no warehouse covers the cart, W1+W2 do, W3 (inactive) is ignored.
    let quote = quotes.create(1, cart.clone()).await.unwrap();
    let snapshot = quote.sellers[0].shipping.clone().unwrap();
    assert_eq!(snapshot.options.len(), 1);
    let split = &snapshot.options[0];
    assert_eq!(split.service_code, "fixed-split");
    assert_eq!(split.price_cents, 3000);
    assert_eq!(split.transit_days, Some(6));
    let warehouses: Vec<_> = split.parcels.iter().map(|p| p.warehouse_id).collect();
    assert_eq!(warehouses, [Some(1), Some(2)]);
    assert_eq!(split.parcels[0].items.len(), 1);
    // The inactive W3 holds sku3 but is never used.
    assert!(quotes.create(1, request(&[(3, 1)])).await.is_err());
    assert_eq!(snapshot.selected_option, *split);

    // TC-19 step 2: the order carries the split total.
    let selected = quotes
        .select(
            1,
            quote.id,
            vec![ShippingSelection {
                tenant_id: 1,
                option_id: split.id.clone(),
            }],
        )
        .await
        .unwrap();
    let created = purchases
        .create_checkout(&buyer(), "split", checkout(selected.id, None))
        .await
        .unwrap();
    let order = &created.detail.orders[0].order;
    assert_eq!(order.shipping_cents, 3000);
    let stored: ShippingSnapshot =
        serde_json::from_value(order.shipping_snapshot.clone().unwrap()).unwrap();
    assert_eq!(stored.selected_option.parcels.len(), 2);

    // TC-17 step 3: stock no longer covers a quantity, so no option remains.
    db.execute_unprepared("UPDATE sku_stock SET quantity=0, reserved=0 WHERE sku_id=2")
        .await
        .unwrap();
    assert!(quotes.create(1, cart.clone()).await.is_err());

    // TC-17 step 4 and TC-19 step 3: W1 alone covers; both options are offered.
    db.execute_unprepared(
        "UPDATE sku_stock SET quantity=20, reserved=0, warehouse_id=1 WHERE sku_id IN (1,2)",
    )
    .await
    .unwrap();
    let quote = quotes.create(1, cart).await.unwrap();
    let options = quote.sellers[0].shipping.clone().unwrap().options;
    let codes: Vec<_> = options.iter().map(|o| o.service_code.as_str()).collect();
    assert_eq!(codes, ["fixed", "fixed-split"]);
    assert_eq!(options[0].price_cents, 1000);
    assert_eq!(options[0].parcels[0].warehouse_id, Some(1));
    let selected = quotes
        .select(
            1,
            quote.id,
            vec![ShippingSelection {
                tenant_id: 1,
                option_id: options[0].id.clone(),
            }],
        )
        .await
        .unwrap();
    let created = purchases
        .create_checkout(&buyer(), "single", checkout(selected.id, None))
        .await
        .unwrap();
    assert_eq!(created.detail.orders[0].order.shipping_cents, 1000);
    cleanup(root, db, schema).await;
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn fixed_mode_rates_tie_breaks_surcharge_and_free_threshold() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixed_fixtures(&db).await;
    // Only W1 stocks the cart, so the single-origin option is W1 and the split mirrors it.
    db.execute_unprepared(r#"
UPDATE sku_stock SET quantity=20, warehouse_id=1 WHERE tenant_id=1;
UPDATE sku SET weight_g=1000;
DELETE FROM shipping_rate WHERE tenant_id=1;
INSERT INTO shipping_rate(id,uuid,tenant_id,origin_warehouse_id,uf,price_cents,transit_days_min,transit_days_max,max_weight_g,extra_weight_per_kg_cents,created_at,updated_at) VALUES
 (11,gen_random_uuid(),1,1,'SP',1000,1,5,2000,300,now(),now()),
 (12,gen_random_uuid(),1,1,'SP',1000,1,3,2000,300,now(),now()),
 (13,gen_random_uuid(),1,1,'SP',1500,1,1,2000,300,now(),now());
"#).await.unwrap();
    let quotes = CheckoutQuoteUseCase::new(db.clone());

    // TC-18: equal price, shorter transit wins (rate 12); 5 kg is 3 started kg above 2 kg.
    let quote = quotes.create(1, request(&[(1, 5)])).await.unwrap();
    let option = &quote.sellers[0].shipping.clone().unwrap().options[0];
    assert_eq!(option.parcels[0].rate_id, 12);
    assert_eq!(option.parcels[0].weight_g, 5000);
    assert_eq!(option.price_cents, 1000 + 3 * 300);

    // The free threshold compares the pre-discount subtotal (5 x 100 = 500).
    db.execute_unprepared("UPDATE shipping_rate SET free_shipping_threshold_cents=500")
        .await
        .unwrap();
    let quote = quotes.create(1, request(&[(1, 5)])).await.unwrap();
    assert_eq!(quote.sellers[0].shipping_cents, 0);
    db.execute_unprepared("UPDATE shipping_rate SET free_shipping_threshold_cents=501")
        .await
        .unwrap();
    let quote = quotes.create(1, request(&[(1, 5)])).await.unwrap();
    assert_eq!(quote.sellers[0].shipping_cents, 1900);
    cleanup(root, db, schema).await;
}
