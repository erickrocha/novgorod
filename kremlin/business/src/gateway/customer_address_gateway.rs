use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::customer_address::{CustomerAddress, CustomerAddressEntityMapper};
use entity::customer_address_entity;
use entity::prelude::CustomerAddressEntity as CustomerAddressQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct CustomerAddressGateway {
    db: DbConn,
}

impl CustomerAddressGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_customer_id(
        &self,
        customer_id: i64,
    ) -> Result<Vec<customer_address_entity::Model>, DbErr> {
        tenant_select(
            CustomerAddressQuery::find(),
            customer_address_entity::Column::TenantId,
        )
        .filter(customer_address_entity::Column::CustomerId.eq(customer_id))
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<CustomerAddress, customer_address_entity::Model, customer_address_entity::ActiveModel>
    for CustomerAddressGateway
{
    async fn persist(
        &self,
        entity: CustomerAddress,
    ) -> Result<customer_address_entity::ActiveModel, DbErr> {
        let active_model = CustomerAddressEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CustomerAddressQuery::delete_many()
                .filter(customer_address_entity::Column::Id.eq(id)),
            customer_address_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<customer_address_entity::Model>, DbErr> {
        tenant_select(
            CustomerAddressQuery::find(),
            customer_address_entity::Column::TenantId,
        )
        .filter(customer_address_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<customer_address_entity::Model>, DbErr> {
        tenant_select(
            CustomerAddressQuery::find(),
            customer_address_entity::Column::TenantId,
        )
        .filter(customer_address_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<customer_address_entity::Model>, DbErr> {
        tenant_select(
            CustomerAddressQuery::find(),
            customer_address_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
