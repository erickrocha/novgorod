use super::*;
use business::sea_orm::DbBackend;

fn sql(query: Select<cart_item_entity::Entity>) -> String {
    let all = query.build(DbBackend::Postgres).to_string();
    // only the WHERE clause matters: the column list names every column
    all.split_once(" WHERE ").map(|(_, rest)| rest.to_string()).unwrap_or_default()
}

#[test]
fn customer_items_are_limited_to_the_owner_carts_and_the_store_tenant() {
    let store = sql(customer_items(5, Some(7)));
    assert!(store.contains("\"customer_id\" = 5"), "{store}");
    assert!(store.contains("\"tenant_id\" = 7"), "{store}");

    let marketplace = sql(customer_items(5, None));
    assert!(marketplace.contains("\"customer_id\" = 5"), "{marketplace}");
    assert!(!marketplace.contains("tenant_id"), "{marketplace}");
}
