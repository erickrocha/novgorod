pub use sea_orm_migration::{MigrationTrait, MigratorTrait, async_trait};

mod m20260917_000001_create_table_user;
mod m20260917_000002_create_tenant_table;
mod m20260917_000003_create_province_table;
mod m20260917_000004_create_city_table;
mod m20260917_000005_create_table_category;
mod m20260917_000006_create_table_product;
mod m20260917_000007_create_table_product_category;
mod m20260917_000008_create_table_catalog_attribute;
mod m20260917_000009_create_table_catalog_attribute_value;
mod m20260917_000010_create_table_product_attribute;
mod m20260917_000011_create_table_sku;
mod m20260917_000012_create_table_sku_attribute_value;
mod m20260917_000013_create_table_sku_stock;
mod m20260917_000014_create_table_product_image;
mod m20260917_000015_add_ibge_location_codes;
mod m20260919_000001_create_table_shipping_rate;
mod m20260919_000002_create_table_customer;
mod m20260919_000003_create_table_customer_address;
mod m20260919_000004_create_table_tax_rule;
mod m20260919_000005_create_table_campaign;
mod m20260919_000006_create_table_campaign_target;
mod m20260919_000007_create_table_coupon;
mod m20260919_000009_create_table_cart;
mod m20260919_000010_create_table_cart_item;
mod m20260919_000011_create_table_orders;
mod m20260919_000012_create_table_order_item;
mod m20260919_000013_create_table_order_status_history;
mod m20260919_000014_create_table_coupon_redemption;
mod m20260921_000001_create_person_table;
mod m20260921_000002_create_person_address_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260917_000001_create_table_user::Migration),
            Box::new(m20260917_000002_create_tenant_table::Migration),
            Box::new(m20260917_000003_create_province_table::Migration),
            Box::new(m20260917_000004_create_city_table::Migration),
            Box::new(m20260917_000005_create_table_category::Migration),
            Box::new(m20260917_000006_create_table_product::Migration),
            Box::new(m20260917_000007_create_table_product_category::Migration),
            Box::new(m20260917_000008_create_table_catalog_attribute::Migration),
            Box::new(m20260917_000009_create_table_catalog_attribute_value::Migration),
            Box::new(m20260917_000010_create_table_product_attribute::Migration),
            Box::new(m20260917_000011_create_table_sku::Migration),
            Box::new(m20260917_000012_create_table_sku_attribute_value::Migration),
            Box::new(m20260917_000013_create_table_sku_stock::Migration),
            Box::new(m20260917_000014_create_table_product_image::Migration),
            Box::new(m20260917_000015_add_ibge_location_codes::Migration),
            Box::new(m20260919_000001_create_table_shipping_rate::Migration),
            Box::new(m20260919_000002_create_table_customer::Migration),
            Box::new(m20260919_000003_create_table_customer_address::Migration),
            Box::new(m20260919_000004_create_table_tax_rule::Migration),
            Box::new(m20260919_000005_create_table_campaign::Migration),
            Box::new(m20260919_000006_create_table_campaign_target::Migration),
            Box::new(m20260919_000007_create_table_coupon::Migration),
            Box::new(m20260919_000009_create_table_cart::Migration),
            Box::new(m20260919_000010_create_table_cart_item::Migration),
            Box::new(m20260919_000011_create_table_orders::Migration),
            Box::new(m20260919_000012_create_table_order_item::Migration),
            Box::new(m20260919_000013_create_table_order_status_history::Migration),
            Box::new(m20260919_000014_create_table_coupon_redemption::Migration),
            Box::new(m20260921_000001_create_person_table::Migration),
            Box::new(m20260921_000002_create_person_address_table::Migration),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn migrations_are_registered_once_in_order() {
        let migrations = Migrator::migrations();
        let names: Vec<&str> = migrations
            .iter()
            .map(|migration| migration.name())
            .collect();
        let unique: HashSet<&str> = names.iter().copied().collect();

        assert_eq!(names.len(), 31);
        assert_eq!(unique.len(), names.len());
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            names.last().copied(),
            Some("m20260923_000001_alter_table_customer")
        );
    }
}
