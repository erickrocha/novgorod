use entity::product_attribute_entity;
use sea_orm::{DbConn, DbErr, EntityTrait};

pub struct ProductAttributeGateway { db: DbConn }
impl ProductAttributeGateway {
    pub fn new(db: DbConn) -> Self { Self { db } }
    pub async fn find_all(&self) -> Result<Vec<product_attribute_entity::Model>, DbErr> {
        product_attribute_entity::Entity::find().all(&self.db).await
    }
}
