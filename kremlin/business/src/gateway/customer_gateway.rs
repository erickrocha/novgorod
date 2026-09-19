use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::customer::{Customer, CustomerEntityMapper};
use entity::customer_entity;
use entity::prelude::CustomerEntity as CustomerQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CustomerGateway {
    db: DbConn,
}

impl CustomerGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_email(&self, email: String) -> Result<Option<customer_entity::Model>, DbErr> {
        tenant_select(CustomerQuery::find(), customer_entity::Column::TenantId)
            .filter(customer_entity::Column::Email.eq(email))
            .one(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<Customer, customer_entity::Model, customer_entity::ActiveModel> for CustomerGateway {
    async fn persist(&self, entity: Customer) -> Result<customer_entity::ActiveModel, DbErr> {
        let active_model = CustomerEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CustomerQuery::delete_many().filter(customer_entity::Column::Id.eq(id)),
            customer_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<customer_entity::Model>, DbErr> {
        tenant_select(CustomerQuery::find(), customer_entity::Column::TenantId)
            .filter(customer_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<customer_entity::Model>, DbErr> {
        tenant_select(CustomerQuery::find(), customer_entity::Column::TenantId)
            .filter(customer_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<customer_entity::Model>, DbErr> {
        tenant_select(CustomerQuery::find(), customer_entity::Column::TenantId)
            .order_by_asc(customer_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
