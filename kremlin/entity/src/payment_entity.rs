use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub purchase_id: i64,
    pub method: String,
    pub status: String,
    pub installments: i32,
    pub amount_cents: i64,
    pub currency: String,
    pub gateway_provider: Option<String>,
    pub gateway_reference: Option<String>,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Purchase,
    PaymentAllocation,
    CreditCardDetails,
    PaymentTransaction,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Purchase => Entity::belongs_to(super::purchase_entity::Entity)
                .from(Column::PurchaseId)
                .to(super::purchase_entity::Column::Id)
                .into(),
            Self::PaymentAllocation => {
                Entity::has_many(super::payment_allocation_entity::Entity).into()
            }
            Self::CreditCardDetails => {
                Entity::has_many(super::credit_card_details_entity::Entity).into()
            }
            Self::PaymentTransaction => {
                Entity::has_many(super::payment_transaction_entity::Entity).into()
            }
        }
    }
}
impl Related<super::purchase_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Purchase.def()
    }
}
impl Related<super::payment_allocation_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentAllocation.def()
    }
}
impl Related<super::credit_card_details_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreditCardDetails.def()
    }
}
impl Related<super::payment_transaction_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentTransaction.def()
    }
}
crate::impl_auditable_before_save!(ActiveModel);
