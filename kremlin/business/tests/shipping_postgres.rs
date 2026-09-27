//! Run only against a disposable database: KREMLIN_TEST_DATABASE_URL=... cargo test
//! -p business --test marketplace_postgres -- --ignored --nocapture
use business::{
    domain::{enums::Role, marketplace::*, user::User},
    gateway::purchase_gateway::PurchaseGateway,
    use_cases::{
        checkout_quote_use_case::{CheckoutQuoteUseCase, QuoteRequest},
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
INSERT INTO shipping_rate(uuid,tenant_id,uf,price_cents,created_at,updated_at) SELECT gen_random_uuid(),id,'SP',0,now(),now() FROM tenant;
"#).await.unwrap();
}

use business::{
    domain::shipping::*, gateway::shipping_provider_gateway::*,
    use_cases::checkout_quote_use_case::ShippingSelection,
};
use sea_orm::prelude::async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct Carrier {
    calls: AtomicUsize,
    fail: bool,
}
#[async_trait]
impl ShippingProviderGateway for Carrier {
    async fn quote(
        &self,
        r: ShippingRequest,
    ) -> Result<Vec<ShippingOption>, ShippingProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(r.tenant_id, 2);
        assert_eq!(r.parcel.weight_g, 600);
        if self.fail {
            return Err(ShippingProviderError::Unavailable);
        }
        Ok(vec![
            ShippingOption {
                id: "fast".into(),
                provider: "correios".into(),
                service_code: "03220".into(),
                service_name: "SEDEX".into(),
                price_cents: 2000,
                transit_days: Some(2),
            },
            ShippingOption {
                id: "cheap".into(),
                provider: "correios".into(),
                service_code: "03298".into(),
                service_name: "PAC".into(),
                price_cents: 1000,
                transit_days: Some(5),
            },
        ])
    }
}
fn request() -> QuoteRequest {
    let i = input();
    QuoteRequest {
        items: i.items,
        address_id: None,
        shipping_address: Some(i.shipping_address),
        coupons: vec![],
    }
}
fn checkout(quote_id: i64) -> CheckoutPurchaseInput {
    CheckoutPurchaseInput {
        quote_id,
        email: "buyer@test.local".into(),
        phone: "11999999999".into(),
    }
}
fn selections(
    quote: &business::use_cases::checkout_quote_use_case::QuoteResult,
) -> Vec<ShippingSelection> {
    quote
        .sellers
        .iter()
        .map(|s| ShippingSelection {
            tenant_id: s.tenant_id,
            option_id: if s.tenant_id == 2 {
                "fast".into()
            } else {
                s.shipping.as_ref().unwrap().selected_option.id.clone()
            },
        })
        .collect()
}
#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn mixed_modes_selection_expiry_inputs_orders_and_idempotency() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await;
    db.execute_unprepared(r#"
UPDATE tenant SET postal_code='01001000';
UPDATE sku SET weight_g=300,length_mm=150,width_mm=100,height_mm=50;
UPDATE shipping_rate SET price_cents=500 WHERE tenant_id=1;
INSERT INTO tenant_shipping_settings(tenant_id,configuration) VALUES(2,'{"mode":"correios","originCep":"01001000","services":[{"code":"03220","name":"SEDEX"},{"code":"03298","name":"PAC"}],"packaging":{"weightG":0,"lengthMm":0,"widthMm":0,"heightMm":0}}');
"#).await.unwrap();
    let provider = Arc::new(Carrier {
        calls: AtomicUsize::new(0),
        fail: false,
    });
    let quotes = CheckoutQuoteUseCase::with_provider(db.clone(), provider.clone());
    let quote = quotes.create(1, request()).await.unwrap();
    assert_eq!(quote.shipping_cents, 1500);
    assert_eq!(quote.total_cents, 3000);
    assert_eq!(
        quote.sellers[1]
            .shipping
            .as_ref()
            .unwrap()
            .selected_option
            .id,
        "cheap"
    );
    assert!(matches!(
        quotes.select(2, quote.id, selections(&quote)).await,
        Err(PurchaseError::NotFound)
    ));
    let mut invalid = selections(&quote);
    invalid[1].option_id = "fake".into();
    assert!(quotes.select(1, quote.id, invalid).await.is_err());
    let selected = quotes
        .select(1, quote.id, selections(&quote))
        .await
        .unwrap();
    assert_ne!(selected.id, quote.id);
    assert_eq!(selected.expires_at, quote.expires_at);
    assert_eq!(selected.shipping_cents, 2500);
    assert_eq!(selected.total_cents, 4000);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    let purchases = PurchaseUseCase::new(PurchaseGateway::new(db.clone()));
    let buyer = user(1, Role::Customer, None);
    assert!(purchases.create(&buyer, "legacy", input()).await.is_err());
    // A changed SKU input invalidates even if the parcel aggregate remains unchanged.
    db.execute_unprepared("UPDATE sku SET length_mm=140 WHERE id=4")
        .await
        .unwrap();
    assert!(matches!(
        purchases
            .create_checkout(&buyer, "stale", checkout(selected.id))
            .await,
        Err(PurchaseError::Conflict)
    ));
    db.execute_unprepared("UPDATE sku SET length_mm=150 WHERE id=4")
        .await
        .unwrap();
    // Insufficient inventory is rechecked before purchase.
    db.execute_unprepared("UPDATE sku_stock SET quantity=0 WHERE sku_id=4")
        .await
        .unwrap();
    assert!(matches!(
        purchases
            .create_checkout(&buyer, "stock", checkout(selected.id))
            .await,
        Err(PurchaseError::Validation(_))
    ));
    db.execute_unprepared("UPDATE sku_stock SET quantity=10 WHERE sku_id=4")
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        purchases.create_checkout(&buyer, "buy", checkout(selected.id)),
        purchases.create_checkout(&buyer, "buy", checkout(selected.id))
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_ne!(a.replayed, b.replayed);
    assert_eq!(a.detail.purchase.id, b.detail.purchase.id);
    assert_eq!(a.detail.purchase.total_cents, 4000);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    let order = &a.detail.orders[1].order;
    let snapshot: ShippingSnapshot =
        serde_json::from_value(order.shipping_snapshot.clone().unwrap()).unwrap();
    assert_eq!(snapshot.selected_option.id, "fast");
    assert_eq!(snapshot.selected_option.price_cents, order.shipping_cents);
    assert!(matches!(
        purchases
            .create_checkout(&buyer, "buy", checkout(quote.id))
            .await,
        Err(PurchaseError::Conflict)
    ));
    db.execute_unprepared("UPDATE checkout_quote SET expires_at=now()-interval '1 minute'")
        .await
        .unwrap();
    assert!(
        purchases
            .create_checkout(&buyer, "buy", checkout(selected.id))
            .await
            .unwrap()
            .replayed
    );
    assert!(matches!(
        quotes.select(1, quote.id, selections(&quote)).await,
        Err(PurchaseError::Conflict)
    ));
    let fresh = quotes.create(1, request()).await.unwrap();
    db.execute_unprepared(
        "UPDATE tenant_shipping_settings SET version=version+1 WHERE tenant_id=2",
    )
    .await
    .unwrap();
    assert!(matches!(
        purchases
            .create_checkout(&buyer, "config", checkout(fresh.id))
            .await,
        Err(PurchaseError::Conflict)
    ));
    let unavailable = CheckoutQuoteUseCase::with_provider(
        db.clone(),
        Arc::new(Carrier {
            calls: AtomicUsize::new(0),
            fail: true,
        }),
    );
    assert!(matches!(
        unavailable.create(1, request()).await,
        Err(PurchaseError::ShippingUnavailable)
    ));
    db.execute_unprepared("UPDATE sku SET weight_g=NULL WHERE id=4")
        .await
        .unwrap();
    assert!(matches!(
        quotes.create(1, request()).await,
        Err(PurchaseError::Validation(_))
    ));
    assert_eq!(count(&db, "purchase").await, 1);
    cleanup(root, db, schema).await;
}
#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn legacy_fixed_snapshots_and_migration_roundtrip() {
    let (root, db, schema) = database().await;
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await;
    let quotes = CheckoutQuoteUseCase::new(db.clone());
    let quote = quotes.create(1, request()).await.unwrap();
    db.execute_unprepared("UPDATE checkout_quote SET result=jsonb_set(result,'{sellers}',(SELECT jsonb_agg(s - 'shipping') FROM jsonb_array_elements(result->'sellers') s))").await.unwrap();
    let (_, old) = CheckoutQuoteUseCase::load(&db, 1, quote.id).await.unwrap();
    assert!(old.sellers.iter().all(|s| s.shipping.is_none()));
    let purchases = PurchaseUseCase::new(PurchaseGateway::new(db.clone()));
    let result = purchases
        .create_checkout(&user(1, Role::Customer, None), "old", checkout(quote.id))
        .await
        .unwrap();
    assert!(
        result
            .detail
            .orders
            .iter()
            .all(|s| s.order.shipping_snapshot.is_none())
    );
    Migrator::down(&db, Some(1)).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    assert_eq!(
        purchases
            .get(&user(1, Role::Customer, None), result.detail.purchase.id)
            .await
            .unwrap()
            .purchase
            .id,
        result.detail.purchase.id
    );
    cleanup(root, db, schema).await;
}
