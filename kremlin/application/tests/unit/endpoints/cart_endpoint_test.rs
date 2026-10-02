use super::*;
use business::sea_orm::{DbBackend, QueryTrait};

fn sql(query: Select<cart_entity::Entity>) -> String {
    let all = query.build(DbBackend::Postgres).to_string();
    // only the WHERE clause matters: the column list names every column
    all.split_once(" WHERE ").map(|(_, rest)| rest.to_string()).unwrap_or_default()
}

#[test]
fn customer_carts_filter_by_owner_and_by_tenant_only_in_store_mode() {
    let store = sql(customer_carts(5, Some(7)));
    assert!(store.contains("\"customer_id\" = 5"), "{store}");
    assert!(store.contains("\"tenant_id\" = 7"), "{store}");

    let marketplace = sql(customer_carts(5, None));
    assert!(marketplace.contains("\"customer_id\" = 5"), "{marketplace}");
    assert!(!marketplace.contains("tenant_id"), "{marketplace}");
}
