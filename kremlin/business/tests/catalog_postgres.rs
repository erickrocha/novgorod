//! Requires KREMLIN_TEST_DATABASE_URL pointing to a disposable database ending in `_test`.
use business::{domain::product::ProductSearchQuery, gateway::product_gateway::ProductGateway};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database};

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn catalog_filters_prices_sellers_and_pagination() {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL").expect("disposable database URL required");
    assert!(url.split('?').next().unwrap().ends_with("_test"));
    let root = Database::connect(url.clone()).await.unwrap();
    let schema = format!("catalog_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await.unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(r#"
        INSERT INTO tenant (id, uuid, business_name, tax_id, created_at, updated_at)
        SELECT n, gen_random_uuid(), 'Seller ' || n, '12345678000100', now(), now()
        FROM generate_series(1,2) n;
        INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
        SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END,
               'Product ' || n, 'product-' || n, n <> 5, '12345678', 0,
               now() + n * interval '1 minute', now()
        FROM generate_series(1,5) n;
        INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at)
        SELECT n, gen_random_uuid(), CASE WHEN n <= 3 THEN 1 ELSE 2 END,
               n, 'SKU-' || n, 'variant-' || n, n * 100, true, now(), now()
        FROM generate_series(1,5) n;
        INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, created_at, updated_at)
        VALUES (10, gen_random_uuid(), 1, 2, 'SKU-10', 'other', 900, true, now(), now()),
               (11, gen_random_uuid(), 1, 2, 'SKU-11', 'inactive', 50, false, now(), now());
        INSERT INTO category (id, uuid, tenant_id, name, slug, active, created_at, updated_at)
        VALUES (1, gen_random_uuid(), 1, 'Wine', 'wine', true, now(), now()),
               (2, gen_random_uuid(), 2, 'Food', 'food', true, now(), now());
        INSERT INTO product_category (uuid, tenant_id, product_id, category_id, is_primary, created_at)
        VALUES (gen_random_uuid(), 1, 1, 1, true, now()),
               (gen_random_uuid(), 1, 2, 1, true, now()),
               (gen_random_uuid(), 2, 4, 2, true, now());
    "#).await.unwrap();

    let gateway = ProductGateway::new(db.clone());
    let (first, cursor, total) = gateway.find_webstore_products(None, ProductSearchQuery {
        category: Some("wine".into()), min_price: Some(200), max_price: Some(200),
        sort_by: Some("price-asc".into()), limit: Some(1), ..Default::default()
    }).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(cursor, None);
    assert_eq!(first[0].id, 2);
    assert_eq!(first[0].price_cents, Some(200));
    assert_eq!(first[0].category_slugs, ["wine"]);
    assert_eq!(first[0].seller.as_ref().unwrap().business_name, "Seller 1");

    let (page1, cursor, total) = gateway.find_webstore_products(None, ProductSearchQuery {
        sort_by: Some("price-desc".into()), limit: Some(2), ..Default::default()
    }).await.unwrap();
    assert_eq!(total, 4);
    assert_eq!(page1.iter().map(|p| p.id).collect::<Vec<_>>(), [4, 3]);
    let (page2, end, next_total) = gateway.find_webstore_products(None, ProductSearchQuery {
        sort_by: Some("price-desc".into()), limit: Some(2), cursor, ..Default::default()
    }).await.unwrap();
    assert_eq!(next_total, 4);
    assert_eq!(page2.iter().map(|p| p.id).collect::<Vec<_>>(), [2, 1]);
    assert_eq!(end, None);

    let (tenant, _, count) = gateway.find_webstore_products(Some(2), ProductSearchQuery {
        q: Some("Product".into()), ..Default::default()
    }).await.unwrap();
    assert_eq!(count, 1);
    assert_eq!(tenant[0].seller.as_ref().unwrap().id, 2);

    db.close().await.unwrap();
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await.unwrap();
    root.close().await.unwrap();
}
