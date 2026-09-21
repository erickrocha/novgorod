use crate::m20260921_000001_create_person_table::Person;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PersonAddress::Table)
                    .if_not_exists()
                    .col(pk_auto(PersonAddress::Id))
                    .col(uuid_uniq(PersonAddress::Uuid))
                    .col(big_integer(PersonAddress::TenantId))
                    .col(big_integer(PersonAddress::PersonId).not_null())
                    .col(string_len_null(PersonAddress::AddressLine1, 500))
                    .col(string_len_null(PersonAddress::AddressLine2, 500))
                    .col(string_len_null(PersonAddress::Locality, 500))
                    .col(string_len_null(PersonAddress::AdministrativeArea, 500))
                    .col(string_len_null(PersonAddress::PostalCode, 20))
                    .col(string_len_null(PersonAddress::CountryCode, 2))
                    .col(date_time(PersonAddress::CreatedAt))
                    .col(string_len_null(PersonAddress::CreatedBy, 255))
                    .col(date_time(PersonAddress::UpdatedAt))
                    .col(string_len_null(PersonAddress::UpdatedBy, 255))
                    .index(
                    Index::create()
                        .name("uq_persons_address_tenant_id_id")
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
                    .name("fk_person_address_person")
                    .from_tbl(PersonAddress::Table)
                    .from_col(PersonAddress::Id)
                    .from_col(PersonAddress::TenantId)
                    .to_tbl(Person::Table)
                    .to_col(Person::Id)
                    .to_col(Person::TenantId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_person_address_person")
                    .to_owned(),
            )
            .await?;
        manager.drop_table(Table::drop().table(PersonAddress::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum PersonAddress {
    Table,
    Id,
    Uuid,
    PersonId,
    TenantId,
    AddressLine1,
    AddressLine2,
    Locality,
    AdministrativeArea,
    PostalCode,
    CountryCode,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
