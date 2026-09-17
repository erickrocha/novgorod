use entity::category_entity;
use sea_orm::{DbConn, DbErr, EntityTrait, QueryOrder};

pub struct CategoryGateway { db: DbConn }
impl CategoryGateway {
    pub fn new(db: DbConn) -> Self { Self { db } }
    pub async fn find_all(&self) -> Result<Vec<category_entity::Model>, DbErr> {
        category_entity::Entity::find().order_by_asc(category_entity::Column::Name).all(&self.db).await
    }
}
