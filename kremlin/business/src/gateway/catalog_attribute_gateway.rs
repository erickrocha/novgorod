use entity::catalog_attribute_entity;
use sea_orm::{DbConn, DbErr, EntityTrait, QueryOrder};
pub struct CatalogAttributeGateway {
    db: DbConn,
}
impl CatalogAttributeGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
    pub async fn find_all(&self) -> Result<Vec<catalog_attribute_entity::Model>, DbErr> {
        catalog_attribute_entity::Entity::find()
            .order_by_asc(catalog_attribute_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
