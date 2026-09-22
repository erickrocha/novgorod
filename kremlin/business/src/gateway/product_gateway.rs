use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product::{Product, ProductEntityMapper, ProductSearchQuery};
use entity::prelude::ProductEntity as ProductQuery;
use entity::product_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct ProductGateway {
    db: DbConn,
}

impl ProductGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_paged_by_cursor(&self,query: ProductSearchQuery) -> Result<(Vec<product_entity::Model>, Option<i64>), DbErr> {
        let mut conditions = Condition::all();

        if let Some(active) = query.active {
            conditions = conditions.add(product_entity::Column::Active.eq(active));
        }

        if let Some(ref q) = query.q {
            let pattern = format!("%{}%", q);
            conditions = conditions.add(
                Condition::any()
                    .add(product_entity::Column::Name.like(&pattern))
                    .add(product_entity::Column::Slug.like(&pattern))
                    .add(product_entity::Column::Brand.like(&pattern))
                    .add(product_entity::Column::Description.like(&pattern))
                    .add(product_entity::Column::Ncm.like(&pattern)),
            );
        }

        let select = tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(conditions);

        let mut paginator = select.cursor_by(product_entity::Column::Id);
        if let Some(cursor) = query.cursor {
            paginator.after(cursor);
        }

        let limit = query.limit.unwrap_or(25);
        let mut rows = paginator.first(limit + 1).all(&self.db).await?;
        
        let next_cursor = if rows.len() as u64 > limit {
            rows.pop();
            rows.last().map(|r| r.id)
        } else {
            None
        };

        Ok((rows, next_cursor))
    }
}

#[async_trait]
impl Gateway<Product, product_entity::Model, product_entity::ActiveModel> for ProductGateway {
    async fn persist(&self, entity: Product) -> Result<product_entity::ActiveModel, DbErr> {
        let active_model = ProductEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ProductQuery::delete_many().filter(product_entity::Column::Id.eq(id)),
            product_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .order_by_asc(product_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
