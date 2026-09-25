use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "order_status_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub tenant_id: i64,
    pub order_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: DateTime,
    pub created_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Orders,
}
impl RelationTrait for Relation {
 fn def(&self) -> RelationDef {
 match self {
Self::Orders => Entity::belongs_to(super::orders_entity::Entity).from(Column::OrderId).to(super::orders_entity::Column::Id).into(),
}
}
}
impl Related<super::orders_entity::Entity> for Entity { fn to() -> RelationDef { Relation::Orders.def() } }
#[sea_orm::prelude::async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
 async fn before_save<C: ConnectionTrait>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr> {
 if insert {
 self.uuid = sea_orm::Set(uuid::Uuid::new_v4());
 self.created_at = sea_orm::Set(chrono::Utc::now().naive_utc());
 self.created_by = sea_orm::Set(crate::audit_entity::CURRENT_USER.try_with(|u| u.as_ref().map(|u| u.email.clone())).ok().flatten());
 }
 Ok(self)
 }
}
