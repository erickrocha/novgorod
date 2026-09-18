use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Province::Table)
                    .add_column(string_len_null(Province::IbgeCode, 2))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(City::Table)
                    .add_column(string_len_null(City::IbgeCode, 7))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("uq_province_ibge_code")
                    .table(Province::Table)
                    .col(Province::IbgeCode)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("uq_city_ibge_code")
                    .table(City::Table)
                    .col(City::IbgeCode)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_city_ibge_code")
                    .table(City::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("uq_province_ibge_code")
                    .table(Province::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(City::Table)
                    .drop_column(City::IbgeCode)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Province::Table)
                    .drop_column(Province::IbgeCode)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Province {
    Table,
    IbgeCode,
}

#[derive(DeriveIden)]
enum City {
    Table,
    IbgeCode,
}
