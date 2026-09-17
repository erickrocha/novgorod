use entity::catalog_attribute_value_entity;
use sea_orm::{DbConn, DbErr, EntityTrait};

pub struct ProductAttributeValueGateway { db: DbConn }
impl ProductAttributeValueGateway {
    pub fn new(db: DbConn) -> Self { Self { db } }
    pub async fn find_all(&self) -> Result<Vec<catalog_attribute_value_entity::Model>, DbErr> {
        catalog_attribute_value_entity::Entity::find().all(&self.db).await
    }
}
