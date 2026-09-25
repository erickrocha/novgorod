use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "checkout_quote")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i64,
    pub customer_id: i64,
    pub request: Json,
    pub result: Json,
    pub expires_at: DateTime,
    pub purchase_id: Option<i64>,
    pub created_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
