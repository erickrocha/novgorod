use super::*;
use entity::shipping_rate_entity::Model as Rate;

fn rate(id: i64) -> Rate {
    let now = chrono::DateTime::UNIX_EPOCH.naive_utc();
    Rate {
        id,
        uuid: uuid::Uuid::nil(),
        tenant_id: Some(1),
        origin_warehouse_id: None,
        region_name: None,
        uf: "SP".into(),
        destination_cep_start: None,
        destination_cep_end: None,
        price_cents: 1000,
        transit_days_min: 1,
        transit_days_max: 5,
        max_weight_g: None,
        extra_weight_per_kg_cents: None,
        free_shipping_threshold_cents: None,
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    }
}
fn wh(id: i64, is_default: bool) -> WarehouseRef {
    WarehouseRef {
        id,
        origin_cep: format!("0100{id}000"),
        is_default,
    }
}
fn stock(warehouse_id: i64, sku_id: i64, available: i64) -> StockRef {
    StockRef {
        warehouse_id,
        sku_id,
        available,
    }
}
fn line(sku_id: i64, quantity: i64) -> Line {
    Line {
        sku_id,
        quantity,
        weight_g: 500,
    }
}

#[test]
fn default_warehouse_first_then_ascending_id() {
    let mut ws = vec![wh(3, false), wh(2, false), wh(5, true)];
    order_warehouses(&mut ws);
    assert_eq!(ws.iter().map(|w| w.id).collect::<Vec<_>>(), [5, 2, 3]);
}

#[test]
fn single_origin_prefers_default_then_lowest_id() {
    let ws = [wh(2, false), wh(1, true), wh(3, false)];
    let lines = [line(10, 2)];
    let all = [stock(1, 10, 2), stock(2, 10, 5), stock(3, 10, 5)];
    assert_eq!(single_origin(&ws, &all, &lines), Some(1));
    let no_default = [stock(1, 10, 1), stock(2, 10, 5), stock(3, 10, 5)];
    assert_eq!(single_origin(&ws, &no_default, &lines), Some(2));
    let none = [stock(1, 10, 1), stock(2, 10, 1)];
    assert_eq!(single_origin(&ws, &none, &lines), None);
}

#[test]
fn split_allocates_greedily_and_ignores_unlisted_warehouses() {
    // W4 is inactive, so it is absent from the active list.
    let ws = [wh(2, false), wh(1, true), wh(3, false)];
    let lines = [line(10, 3), line(20, 2)];
    let st = [
        stock(1, 10, 2),
        stock(2, 10, 5),
        stock(2, 20, 2),
        stock(4, 10, 9),
        stock(4, 20, 9),
    ];
    let allocation = split_allocation(&ws, &st, &lines).unwrap();
    assert_eq!(
        allocation,
        vec![
            Allocation {
                warehouse_id: Some(1),
                items: vec![(10, 2)]
            },
            Allocation {
                warehouse_id: Some(2),
                items: vec![(10, 1), (20, 2)]
            },
        ]
    );
}

#[test]
fn split_is_not_offered_when_stock_is_short() {
    let ws = [wh(1, true), wh(2, false)];
    let lines = [line(10, 5)];
    let st = [stock(1, 10, 2), stock(2, 10, 2)];
    assert_eq!(split_allocation(&ws, &st, &lines), None);
    let reserved = [stock(1, 10, -3), stock(2, 10, 5)];
    assert!(split_allocation(&ws, &reserved, &lines).is_some());
}

#[test]
fn rate_specificity_beats_price() {
    let mut general = rate(1);
    general.price_cents = 100;
    let mut by_warehouse = rate(2);
    by_warehouse.origin_warehouse_id = Some(7);
    by_warehouse.price_cents = 900;
    let mut by_range = rate(3);
    by_range.destination_cep_start = Some("01000000".into());
    by_range.destination_cep_end = Some("01999999".into());
    by_range.price_cents = 500;
    let rates = [general, by_warehouse, by_range];
    assert_eq!(pick_rate(&rates, Some(7), "01310100", "SP").unwrap().id, 2);
    assert_eq!(pick_rate(&rates, Some(8), "01310100", "SP").unwrap().id, 3);
    assert_eq!(pick_rate(&rates, None, "09999999", "sp").unwrap().id, 1);
    assert!(pick_rate(&rates, None, "09999999", "RJ").is_none());
}

#[test]
fn rate_ties_break_by_price_then_transit_then_id() {
    let mut a = rate(5);
    a.price_cents = 800;
    let mut b = rate(4);
    b.price_cents = 700;
    b.transit_days_max = 9;
    let mut c = rate(3);
    c.price_cents = 700;
    c.transit_days_max = 4;
    let mut d = rate(2);
    d.price_cents = 700;
    d.transit_days_max = 4;
    let rates = [a, b, c, d];
    assert_eq!(pick_rate(&rates, None, "01310100", "SP").unwrap().id, 2);
}

#[test]
fn surcharge_applies_per_started_kg_above_maximum() {
    let mut r = rate(1);
    r.price_cents = 1000;
    r.max_weight_g = Some(1000);
    r.extra_weight_per_kg_cents = Some(300);
    assert_eq!(charge(&r, 1000, 0), 1000);
    assert_eq!(charge(&r, 1001, 0), 1300);
    assert_eq!(charge(&r, 2500, 0), 1600);
}

#[test]
fn free_threshold_uses_pre_discount_subtotal() {
    let mut r = rate(1);
    r.free_shipping_threshold_cents = Some(10_000);
    assert_eq!(charge(&r, 100, 9_999), 1000);
    assert_eq!(charge(&r, 100, 10_000), 0);
}

#[test]
fn split_packages_are_priced_separately_and_free_threshold_waives_each() {
    let ws = [wh(1, true), wh(2, false)];
    let lines = [line(10, 3)];
    let mut near = rate(1);
    near.origin_warehouse_id = Some(1);
    near.price_cents = 1000;
    near.transit_days_max = 2;
    near.free_shipping_threshold_cents = Some(5_000);
    let mut far = rate(2);
    far.origin_warehouse_id = Some(2);
    far.price_cents = 2000;
    far.transit_days_max = 6;
    let allocations = [
        Allocation {
            warehouse_id: Some(1),
            items: vec![(10, 2)],
        },
        Allocation {
            warehouse_id: Some(2),
            items: vec![(10, 1)],
        },
    ];
    let rates = [near, far];
    let parcels = price_packages(
        &allocations,
        &ws,
        &rates,
        &lines,
        100,
        None,
        "01310100",
        "SP",
        6_000,
    )
    .unwrap();
    assert_eq!(parcels.iter().map(|p| p.price_cents).collect::<Vec<_>>(), [0, 2000]);
    assert_eq!(parcels[0].weight_g, 1100);
    assert_eq!(parcels[1].weight_g, 600);
    assert_eq!(parcels[1].origin_cep.as_deref(), Some("01002000"));
}

#[test]
fn package_without_rate_refuses_the_option() {
    let ws = [wh(1, true)];
    let lines = [line(10, 1)];
    let mut r = rate(1);
    r.uf = "RJ".into();
    let allocations = [Allocation {
        warehouse_id: Some(1),
        items: vec![(10, 1)],
    }];
    assert!(
        price_packages(&allocations, &ws, &[r], &lines, 0, None, "01310100", "SP", 0).is_none()
    );
}
