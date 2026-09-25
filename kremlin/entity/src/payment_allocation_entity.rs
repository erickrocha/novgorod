use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "payment_allocation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub purchase_id: i64,
    pub payment_id: i64,
    pub order_id: i64,
    pub amount_cents: i64,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Payment,
    Orders,
    Purchase,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Payment => Entity::belongs_to(super::payment_entity::Entity)
                .from(Column::PaymentId)
                .to(super::payment_entity::Column::Id)
                .into(),
            Self::Orders => Entity::belongs_to(super::orders_entity::Entity)
                .from(Column::OrderId)
                .to(super::orders_entity::Column::Id)
                .into(),
            Self::Purchase => Entity::belongs_to(super::purchase_entity::Entity)
                .from(Column::PurchaseId)
                .to(super::purchase_entity::Column::Id)
                .into(),
        }
    }
}
impl Related<super::payment_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}
impl Related<super::orders_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}
impl Related<super::purchase_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Purchase.def()
    }
}
crate::impl_auditable_before_save!(ActiveModel);
