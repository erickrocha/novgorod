use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "payment_transaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub payment_id: i64,
    pub operation: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub idempotency_key: String,
    pub gateway_provider: Option<String>,
    pub gateway_transaction_id: Option<String>,
    pub authorization_code: Option<String>,
    pub response_code: Option<String>,
    pub response_message: Option<String>,
    pub created_at: DateTime,
    pub created_by: Option<String>,
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
#[sea_orm::prelude::async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C: ConnectionTrait>(
        mut self,
        _db: &C,
        insert: bool,
    ) -> Result<Self, DbErr> {
        if insert {
            self.uuid = sea_orm::Set(uuid::Uuid::new_v4());
            self.created_at = sea_orm::Set(chrono::Utc::now().naive_utc());
            self.created_by = sea_orm::Set(
                crate::audit_entity::CURRENT_USER
                    .try_with(|u| u.as_ref().map(|u| u.email.clone()))
                    .ok()
                    .flatten(),
            );
        }
        Ok(self)
    }
}
