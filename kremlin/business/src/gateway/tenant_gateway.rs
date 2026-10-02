use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::Gateway;
use crate::domain::tenant::{Tenant, TenantEntityMapper};
use entity::prelude::TenantEntity as TenantQuery;
use entity::tenant_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
};
use sea_orm::sea_query::Expr;

pub struct TenantGateway {
    db: DbConn,
}

impl TenantGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl TenantGateway {
    /// Sets only the listing flag (SR-TEN-019); returns the number of rows changed.
    pub async fn set_listed(&self, id: i64, listed: bool) -> Result<u64, DbErr> {
        let result = TenantQuery::update_many()
            .col_expr(tenant_entity::Column::Listed, Expr::value(listed))
            .filter(tenant_entity::Column::Id.eq(id))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected)
    }
}

#[async_trait]
impl Gateway<Tenant, tenant_entity::Model, tenant_entity::ActiveModel> for TenantGateway {
    async fn persist(&self, entity: Tenant) -> Result<tenant_entity::ActiveModel, DbErr> {
        let active_model = TenantEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        TenantQuery::delete_by_id(id).exec(&self.db).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<tenant_entity::Model>, DbErr> {
        TenantQuery::find()
            .filter(tenant_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<tenant_entity::Model>, DbErr> {
        TenantQuery::find()
            .filter(tenant_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<tenant_entity::Model>, DbErr> {
        TenantQuery::find().all(&self.db).await
    }
}
