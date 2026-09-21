use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::person_address::{PersonAddress, PersonAddressEntityMapper};
use entity::person_address_entity;
use entity::prelude::PersonAddressEntity as PersonAddressQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct PersonAddressGateway {
    db: DbConn,
}

impl PersonAddressGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_person_id(
        &self,
        person_id: i64,
    ) -> Result<Vec<person_address_entity::Model>, DbErr> {
        tenant_select(
            PersonAddressQuery::find(),
            person_address_entity::Column::TenantId,
        )
        .filter(person_address_entity::Column::PersonId.eq(person_id))
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<PersonAddress, person_address_entity::Model, person_address_entity::ActiveModel>
    for PersonAddressGateway
{
    async fn persist(
        &self,
        entity: PersonAddress,
    ) -> Result<person_address_entity::ActiveModel, DbErr> {
        let active_model = PersonAddressEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            PersonAddressQuery::delete_many()
                .filter(person_address_entity::Column::Id.eq(id)),
            person_address_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<person_address_entity::Model>, DbErr> {
        tenant_select(
            PersonAddressQuery::find(),
            person_address_entity::Column::TenantId,
        )
        .filter(person_address_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<person_address_entity::Model>, DbErr> {
        tenant_select(
            PersonAddressQuery::find(),
            person_address_entity::Column::TenantId,
        )
        .filter(person_address_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<person_address_entity::Model>, DbErr> {
        tenant_select(
            PersonAddressQuery::find(),
            person_address_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
