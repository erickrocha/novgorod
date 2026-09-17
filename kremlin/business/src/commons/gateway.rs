use entity::audit_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DbErr, DeleteMany, DeleteResult, EntityTrait, QueryFilter, Select};

#[async_trait]
pub trait Gateway<D, M, AM> {
    async fn persist(&self, domain: D) -> Result<AM, DbErr>;
    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr>;
    async fn find_by_id(&self, id: i64) -> Result<Option<M>, DbErr>;
    async fn find_by_uuid(&self, uuid: String) -> Result<Option<M>, DbErr>;
    async fn find_all(&self) -> Result<Vec<M>, DbErr>;
}

pub fn tenant_select<E, C>(query: Select<E>, tenant_column: C) -> Select<E>
where
    E: EntityTrait,
    C: ColumnTrait,
{
    match audit_entity::tenant_scope() {
        audit_entity::TenantScope::Unrestricted => query,
        audit_entity::TenantScope::Tenant(id) => query.filter(tenant_column.eq(id)),
        audit_entity::TenantScope::Denied => query.filter(Expr::cust("1 = 0")),
    }
}

pub fn tenant_delete<E, C>(query: DeleteMany<E>, tenant_column: C) -> DeleteMany<E>
where
    E: EntityTrait,
    C: ColumnTrait,
{
    match audit_entity::tenant_scope() {
        audit_entity::TenantScope::Unrestricted => query,
        audit_entity::TenantScope::Tenant(id) => query.filter(tenant_column.eq(id)),
        audit_entity::TenantScope::Denied => query.filter(Expr::cust("1 = 0")),
    }
}
