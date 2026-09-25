use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "credit_card_details")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub payment_id: i64,
    pub cardholder_name: String,
    pub brand: String,
    pub last_four_digits: String,
    pub expiration_month: i32,
    pub expiration_year: i32,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Payment,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Payment => Entity::belongs_to(super::payment_entity::Entity)
                .from(Column::PaymentId)
                .to(super::payment_entity::Column::Id)
                .into(),
        }
    }
}
impl Related<super::payment_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}
crate::impl_auditable_before_save!(ActiveModel);
