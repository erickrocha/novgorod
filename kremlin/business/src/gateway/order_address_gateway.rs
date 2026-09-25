use entity::order_address_entity as record;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
};

pub struct OrderAddressGateway;
impl OrderAddressGateway {
    pub(crate) async fn insert<C: ConnectionTrait>(
        db: &C,
        model: record::ActiveModel,
    ) -> Result<record::Model, DbErr> {
        model.insert(db).await
    }
    pub(crate) async fn for_parent<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<Vec<record::Model>, DbErr> {
        record::Entity::find()
            .filter(record::Column::OrderId.eq(id))
            .order_by_asc(record::Column::Id)
            .all(db)
            .await
    }
}
