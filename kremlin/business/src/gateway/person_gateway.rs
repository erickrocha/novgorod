use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::person::{Person, PersonEntityMapper};
use entity::person_entity;
use entity::prelude::PersonEntity as PersonQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct PersonGateway {
    db: DbConn,
}

impl PersonGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> Result<Option<person_entity::Model>, DbErr> {
        tenant_select(PersonQuery::find(), person_entity::Column::TenantId)
            .filter(person_entity::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    pub async fn find_by_email(&self, email: String) -> Result<Option<person_entity::Model>, DbErr> {
        tenant_select(PersonQuery::find(), person_entity::Column::TenantId)
            .filter(person_entity::Column::Email.eq(email))
            .one(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<Person, person_entity::Model, person_entity::ActiveModel> for PersonGateway {
    async fn persist(&self, entity: Person) -> Result<person_entity::ActiveModel, DbErr> {
        let active_model = PersonEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            PersonQuery::delete_many().filter(person_entity::Column::Id.eq(id)),
            person_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<person_entity::Model>, DbErr> {
        tenant_select(PersonQuery::find(), person_entity::Column::TenantId)
            .filter(person_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<person_entity::Model>, DbErr> {
        tenant_select(PersonQuery::find(), person_entity::Column::TenantId)
            .filter(person_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<person_entity::Model>, DbErr> {
        tenant_select(PersonQuery::find(), person_entity::Column::TenantId)
            .order_by_asc(person_entity::Column::Surname)
            .order_by_asc(person_entity::Column::FirstName)
            .all(&self.db)
            .await
    }
}
