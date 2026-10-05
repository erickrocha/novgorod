use super::*;
use chrono::Utc;
use entity::product_entity;
use sea_orm::{DatabaseBackend, MockDatabase};

fn product(id: i64, tenant_id: i64, name: &str) -> product_entity::Model {
    let now = Utc::now().naive_utc();
    product_entity::Model {
        id,
        uuid: uuid::Uuid::new_v4(),
        tenant_id: Some(tenant_id),
        name: name.into(),
        slug: name.to_lowercase().replace(' ', "-"),
        description: None,
        brand: None,
        active: true,
        ncm: "12345678".into(),
        cest: None,
        origem_mercadoria: 0,
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    }
}

#[tokio::test]
async fn marketplace_catalog_without_tenant_contains_products_from_multiple_sellers() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![
            product(1, 10, "Seller One Product"),
            product(2, 20, "Seller Two Product"),
        ]])
        .into_connection();
    let use_case = ProductUseCase::new(ProductGateway::new(db));

    let products = use_case.find_all_for_tenant(None).await;

    assert_eq!(
        products
            .iter()
            .filter_map(|product| product.tenant_id)
            .collect::<Vec<_>>(),
        [10, 20]
    );
}
