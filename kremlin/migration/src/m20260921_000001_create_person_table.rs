use crate::m20260917_000001_create_table_user::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Person::Table)
                    .if_not_exists()
                    .col(pk_auto(Person::Id).big_integer())
                    .col(uuid_uniq(Person::Uuid))
                    .col(big_integer(Person::TenantId))
                    .col(big_integer(Person::UserId).not_null())
                    .col(string_len(Person::FirstName, 500).not_null())
                    .col(string_len_null(Person::Surname, 500))
                    .col(date(Person::DateOfBirth).null())
                    .col(string_len_null(Person::Gender, 50))
                    .col(string_len(Person::Avatar, 500).null())
                    .col(string_len(Person::Phone, 500).null())
                    .col(string_len(Person::Email, 500).null())
                    .col(date_time(Person::CreatedAt))
                    .col(string_len_null(Person::CreatedBy, 255))
                    .col(date_time(Person::UpdatedAt))
                    .col(string_len_null(Person::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_persosn_tenant_id_id")
                            .col(Person::TenantId)
                            .col(Person::Id)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_person_user")
                    .from_tbl(Person::Table)
                    .from_col(Person::UserId)
                    .to_tbl(User::Table)
                    .to_col(User::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().name("fk_person_user").to_owned()).await?;
        manager.drop_table(Table::drop().table(Person::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Person {
    Table,
    Id,
    Uuid,
    UserId,
    TenantId,
    DateOfBirth,
    FirstName,
    Surname,
    Gender,
    Avatar,
    Email,
    Phone,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
