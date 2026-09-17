use entity::sku_entity;
use sea_orm::{DbConn, DbErr, EntityTrait, QueryOrder};

pub struct SkuGateway { db: DbConn }
impl SkuGateway {
    pub fn new(db: DbConn) -> Self { Self { db } }
    pub async fn find_all(&self) -> Result<Vec<sku_entity::Model>, DbErr> {
        sku_entity::Entity::find().order_by_asc(sku_entity::Column::Code).all(&self.db).await
    }
}
