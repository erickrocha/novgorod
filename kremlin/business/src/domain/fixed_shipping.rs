//! Fixed-mode shipping policy (SR-MKT-011, NOV-SHP-017): warehouse allocation, rate choice and
//! package charge. Everything here is a pure function, so the money rules are unit tested without
//! a database.

use crate::domain::shipping::{ShippingParcel, ShippingParcelItem};
use entity::shipping_rate_entity;
use std::cmp::Reverse;
use std::collections::BTreeMap;

/// An active warehouse of the seller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarehouseRef {
    pub id: i64,
    pub origin_cep: String,
    pub is_default: bool,
}

/// A requested item with its unit weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub sku_id: i64,
    pub quantity: i64,
    pub weight_g: i64,
}

/// Available (unreserved) stock of one SKU in one warehouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockRef {
    pub warehouse_id: i64,
    pub sku_id: i64,
    pub available: i64,
}

/// One allocated package: the warehouse (`None` for the configured origin) and its items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub warehouse_id: Option<i64>,
    pub items: Vec<(i64, i64)>,
}

/// Active default warehouse first, then ascending id.
pub fn order_warehouses(warehouses: &mut [WarehouseRef]) {
    warehouses.sort_by_key(|w| (Reverse(w.is_default), w.id));
}

fn available(stock: &[StockRef], warehouse_id: i64, sku_id: i64) -> i64 {
    stock
        .iter()
        .filter(|s| s.warehouse_id == warehouse_id && s.sku_id == sku_id)
        .map(|s| s.available.max(0))
        .sum()
}

/// First warehouse (in policy order) that can fulfill every line alone.
pub fn single_origin(
    warehouses: &[WarehouseRef],
    stock: &[StockRef],
    lines: &[Line],
) -> Option<i64> {
    let mut ordered = warehouses.to_vec();
    order_warehouses(&mut ordered);
    ordered
        .iter()
        .find(|w| {
            lines
                .iter()
                .all(|l| available(stock, w.id, l.sku_id) >= l.quantity)
        })
        .map(|w| w.id)
}

/// Greedy allocation in policy order; `None` when the active stock does not cover every line.
pub fn split_allocation(
    warehouses: &[WarehouseRef],
    stock: &[StockRef],
    lines: &[Line],
) -> Option<Vec<Allocation>> {
    let mut ordered = warehouses.to_vec();
    order_warehouses(&mut ordered);
    let mut remaining: BTreeMap<i64, i64> = BTreeMap::new();
    for line in lines {
        *remaining.entry(line.sku_id).or_default() += line.quantity;
    }
    let mut out = Vec::new();
    for warehouse in &ordered {
        let mut items = Vec::new();
        for (sku_id, left) in remaining.iter_mut() {
            if *left == 0 {
                continue;
            }
            let take = available(stock, warehouse.id, *sku_id).min(*left);
            if take > 0 {
                items.push((*sku_id, take));
                *left -= take;
            }
        }
        if !items.is_empty() {
            out.push(Allocation {
                warehouse_id: Some(warehouse.id),
                items,
            });
        }
    }
    remaining.values().all(|left| *left == 0).then_some(out)
}

/// Best matching rate: warehouse-specific before general, CEP range before UF, then lower price,
/// shorter maximum transit time and lower id. `None` when no rate matches.
pub fn pick_rate<'a>(
    rates: &'a [shipping_rate_entity::Model],
    origin_warehouse_id: Option<i64>,
    destination_cep: &str,
    destination_uf: &str,
) -> Option<&'a shipping_rate_entity::Model> {
    rates
        .iter()
        .filter_map(|r| {
            let origin_specific = match r.origin_warehouse_id {
                Some(id) if origin_warehouse_id == Some(id) => 1_u8,
                Some(_) => return None,
                None => 0,
            };
            let range = match (&r.destination_cep_start, &r.destination_cep_end) {
                (Some(start), Some(end)) if !start.trim().is_empty() && !end.trim().is_empty() => {
                    if destination_cep >= start.trim() && destination_cep <= end.trim() {
                        1_u8
                    } else {
                        return None;
                    }
                }
                _ => 0,
            };
            if range == 0 && !r.uf.trim().eq_ignore_ascii_case(destination_uf) {
                return None;
            }
            Some((
                (
                    Reverse(origin_specific),
                    Reverse(range),
                    r.price_cents,
                    r.transit_days_max,
                    r.id,
                ),
                r,
            ))
        })
        .min_by_key(|(key, _)| *key)
        .map(|(_, r)| r)
}

/// Package charge: base price plus the surcharge for each started kilogram above the rate maximum;
/// waived when the seller's pre-discount merchandise subtotal reaches the free-shipping threshold.
pub fn charge(rate: &shipping_rate_entity::Model, weight_g: i64, seller_subtotal: i64) -> i64 {
    if rate
        .free_shipping_threshold_cents
        .is_some_and(|t| seller_subtotal >= i64::from(t))
    {
        return 0;
    }
    let mut price = i64::from(rate.price_cents);
    if let (Some(max), Some(per_kg)) = (rate.max_weight_g, rate.extra_weight_per_kg_cents) {
        let max = i64::from(max);
        if weight_g > max && per_kg > 0 {
            let started_kg = (weight_g - max + 999) / 1000;
            price = price.saturating_add(started_kg.saturating_mul(i64::from(per_kg)));
        }
    }
    price
}

/// Builds the priced packages of an allocation, or `None` when any package has no matching rate.
#[allow(clippy::too_many_arguments)]
pub fn price_packages(
    allocations: &[Allocation],
    warehouses: &[WarehouseRef],
    rates: &[shipping_rate_entity::Model],
    lines: &[Line],
    packaging_weight_g: i64,
    configured_origin_cep: Option<&str>,
    destination_cep: &str,
    destination_uf: &str,
    seller_subtotal: i64,
) -> Option<Vec<ShippingParcel>> {
    let mut parcels = Vec::new();
    for allocation in allocations {
        let rate = pick_rate(
            rates,
            allocation.warehouse_id,
            destination_cep,
            destination_uf,
        )?;
        let weight_g = packaging_weight_g
            + allocation
                .items
                .iter()
                .map(|(sku, qty)| {
                    lines
                        .iter()
                        .find(|l| l.sku_id == *sku)
                        .map_or(0, |l| l.weight_g)
                        .saturating_mul(*qty)
                })
                .sum::<i64>();
        let origin_cep = allocation
            .warehouse_id
            .and_then(|id| warehouses.iter().find(|w| w.id == id))
            .map(|w| w.origin_cep.clone())
            .or_else(|| configured_origin_cep.map(str::to_owned));
        parcels.push(ShippingParcel {
            warehouse_id: allocation.warehouse_id,
            origin_cep,
            rate_id: rate.id,
            region_name: rate.region_name.clone(),
            price_cents: charge(rate, weight_g, seller_subtotal),
            transit_days: (rate.transit_days_max > 0).then_some(rate.transit_days_max as u32),
            weight_g,
            items: allocation
                .items
                .iter()
                .map(|(sku_id, quantity)| ShippingParcelItem {
                    sku_id: *sku_id,
                    quantity: *quantity,
                })
                .collect(),
        });
    }
    Some(parcels)
}

#[cfg(test)]
#[path = "../../tests/unit/domain/fixed_shipping_test.rs"]
mod fixed_shipping_tests;
