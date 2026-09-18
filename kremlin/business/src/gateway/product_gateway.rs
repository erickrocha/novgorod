use entity::product_entity;
use sea_orm::{DbConn, DbErr, EntityTrait, QueryOrder};

pub struct ProductGateway {
    db: DbConn,
}
impl ProductGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
    pub async fn find_all(&self) -> Result<Vec<product_entity::Model>, DbErr> {
        product_entity::Entity::find()
            .order_by_asc(product_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
