use crate::m20260917_000002_create_tenant_table::Tenant;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaxRule::Table)
                    .if_not_exists()
                    .col(pk_auto(TaxRule::Id).big_integer())
                    .col(uuid_uniq(TaxRule::Uuid))
                    .col(big_integer(TaxRule::TenantId))
                    .col(string_len(TaxRule::UfOrigem, 2))
                    .col(string_len(TaxRule::UfDestino, 2))
                    .col(string_len_null(TaxRule::NcmPrefix, 8))
                    .col(string_len(TaxRule::Regime, 20))
                    .col(string_len_null(TaxRule::Csosn, 4))
                    .col(string_len(TaxRule::Cfop, 4))
                    .col(integer(TaxRule::IcmsRateBp).check(Expr::cust("icms_rate_bp >= 0")))
                    .col(boolean(TaxRule::Active).default(true))
                    .col(date_time(TaxRule::CreatedAt))
                    .col(string_len_null(TaxRule::CreatedBy, 255))
                    .col(date_time(TaxRule::UpdatedAt))
                    .col(string_len_null(TaxRule::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_tax_rule_tenant_id_id")
                            .col(TaxRule::TenantId)
                            .col(TaxRule::Id)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tax_rule_tenant")
                            .from(TaxRule::Table, TaxRule::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_rule_tenant_destino_ncm")
                    .table(TaxRule::Table)
                    .col(TaxRule::TenantId)
                    .col(TaxRule::UfDestino)
                    .col(TaxRule::NcmPrefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaxRule::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TaxRule {
    Table,
    Id,
    Uuid,
    TenantId,
    UfOrigem,
    UfDestino,
    NcmPrefix,
    Regime,
    Csosn,
    Cfop,
    IcmsRateBp,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
