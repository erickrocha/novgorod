use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use entity::orders_entity as record;

pub struct OrdersGateway;
impl OrdersGateway {
 pub(crate) async fn insert<C: ConnectionTrait>(db: &C, model: record::ActiveModel) -> Result<record::Model, DbErr> { model.insert(db).await }
 pub(crate) async fn for_parent<C: ConnectionTrait>(db: &C, id: i64) -> Result<Vec<record::Model>, DbErr> { record::Entity::find().filter(record::Column::PurchaseId.eq(id)).order_by_asc(record::Column::Id).all(db).await }
}
