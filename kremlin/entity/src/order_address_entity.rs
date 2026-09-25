use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "order_address")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Uuid,
    pub tenant_id: i64,
    pub order_id: i64,
    pub address_type: String,
    pub recipient: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub locality: String,
    pub administrative_area: String,
    pub postal_code: String,
    pub country_code: String,
    pub created_at: DateTime,
    pub created_by: Option<String>,
    pub updated_at: DateTime,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Orders,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Orders => Entity::belongs_to(super::orders_entity::Entity)
                .from(Column::OrderId)
                .to(super::orders_entity::Column::Id)
                .into(),
        }
    }
}
impl Related<super::orders_entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}
crate::impl_auditable_before_save!(ActiveModel);
