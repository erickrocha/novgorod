use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub purchase_id: i64,
    pub tenant_id: i64,
    pub number: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub customer_tax_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub status: String,
    pub payment_status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub shipping_cents: i64,
    pub tax_total_cents: i64,
    pub total_cents: i64,
    pub coupon_id: Option<i64>,
    pub coupon_code: Option<String>,
    pub placed_at: DateTime,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Purchase,
    Customer,
    Tenant,
    Coupon,
    OrderItem,
    OrderAddress,
    OrderStatusHistory,
    CouponRedemption,
    PaymentAllocation,
}
impl RelationTrait for Relation {
 fn def(&self) -> RelationDef {
 match self {
Self::Purchase => Entity::belongs_to(super::purchase_entity::Entity).from(Column::PurchaseId).to(super::purchase_entity::Column::Id).into(),
Self::Customer => Entity::belongs_to(super::customer_entity::Entity).from(Column::CustomerId).to(super::customer_entity::Column::Id).into(),
Self::Tenant => Entity::belongs_to(super::tenant_entity::Entity).from(Column::TenantId).to(super::tenant_entity::Column::Id).into(),
Self::Coupon => Entity::belongs_to(super::coupon_entity::Entity).from(Column::CouponId).to(super::coupon_entity::Column::Id).into(),
Self::OrderItem => Entity::has_many(super::order_item_entity::Entity).into(),
Self::OrderAddress => Entity::has_many(super::order_address_entity::Entity).into(),
Self::OrderStatusHistory => Entity::has_many(super::order_status_history_entity::Entity).into(),
Self::CouponRedemption => Entity::has_many(super::coupon_redemption_entity::Entity).into(),
Self::PaymentAllocation => Entity::has_many(super::payment_allocation_entity::Entity).into(),
}
}
}
impl Related<super::purchase_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Purchase.def() } }
impl Related<super::customer_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Customer.def() } }
impl Related<super::tenant_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Tenant.def() } }
impl Related<super::coupon_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Coupon.def() } }
impl Related<super::order_item_entity::Entity> for Entity { fn to() -> RelationDef { Relation::OrderItem.def() } }
impl Related<super::order_address_entity::Entity> for Entity { fn to() -> RelationDef { Relation::OrderAddress.def() } }
impl Related<super::order_status_history_entity::Entity> for Entity { fn to() -> RelationDef { Relation::OrderStatusHistory.def() } }
impl Related<super::coupon_redemption_entity::Entity> for Entity { fn to() -> RelationDef { Relation::CouponRedemption.def() } }
impl Related<super::payment_allocation_entity::Entity> for Entity { fn to() -> RelationDef { Relation::PaymentAllocation.def() } }
crate::impl_auditable_before_save!(ActiveModel);
