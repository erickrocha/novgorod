use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "purchase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub customer_id: i64,
    pub customer_name: String,
    pub customer_tax_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub currency: String,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub shipping_cents: i64,
    pub tax_total_cents: i64,
    pub total_cents: i64,
    pub idempotency_key: String,
    pub request_hash: String,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Customer,
    Orders,
    Payment,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Customer => Entity::belongs_to(super::customer_entity::Entity)
                .from(Column::CustomerId)
                .to(super::customer_entity::Column::Id)
                .into(),
            Self::Orders => Entity::has_many(super::orders_entity::Entity).into(),
            Self::Payment => Entity::has_many(super::payment_entity::Entity).into(),
        }
    }
}
impl Related<super::customer_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}
impl Related<super::orders_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}
impl Related<super::payment_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}
crate::impl_auditable_before_save!(ActiveModel);
