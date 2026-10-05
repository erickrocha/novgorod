use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "checkout_stock_reservation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub purchase_id: i64,
    pub order_id: i64,
    pub sku_id: i64,
    pub tenant_id: i64,
    pub quantity: i32,
    pub status: String,
    pub expires_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::purchase_entity::Entity",
        from = "Column::PurchaseId",
        to = "super::purchase_entity::Column::Id"
    )]
    Purchase,
    #[sea_orm(
        belongs_to = "super::orders_entity::Entity",
        from = "(Column::PurchaseId, Column::OrderId)",
        to = "(super::orders_entity::Column::PurchaseId, super::orders_entity::Column::Id)"
    )]
    Order,
    #[sea_orm(
        belongs_to = "super::sku_stock_entity::Entity",
        from = "(Column::TenantId, Column::SkuId)",
        to = "(super::sku_stock_entity::Column::TenantId, super::sku_stock_entity::Column::SkuId)"
    )]
    SkuStock,
}

impl Related<super::purchase_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Purchase.def()
    }
}

impl Related<super::orders_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl Related<super::sku_stock_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SkuStock.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}