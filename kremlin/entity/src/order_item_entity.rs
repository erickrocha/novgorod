use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "order_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub tenant_id: i64,
    pub order_id: i64,
    pub sku_id: i64,
    pub sku_code: String,
    pub product_name: String,
    pub attributes_desc: Option<String>,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub ncm: String,
    pub cfop: Option<String>,
    pub csosn: Option<String>,
    pub icms_rate_bp: i32,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Orders,
    Sku,
}
impl RelationTrait for Relation {
 fn def(&self) -> RelationDef {
 match self {
Self::Orders => Entity::belongs_to(super::orders_entity::Entity).from(Column::OrderId).to(super::orders_entity::Column::Id).into(),
Self::Sku => Entity::belongs_to(super::sku_entity::Entity).from(Column::SkuId).to(super::sku_entity::Column::Id).into(),
}
}
}
impl Related<super::orders_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Orders.def() } }
impl Related<super::sku_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Sku.def() } }
crate::impl_auditable_before_save!(ActiveModel);
