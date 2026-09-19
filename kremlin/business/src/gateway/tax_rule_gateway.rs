use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::tax_rule::{TaxRule, TaxRuleEntityMapper};
use entity::prelude::TaxRuleEntity as TaxRuleQuery;
use entity::tax_rule_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct TaxRuleGateway {
    db: DbConn,
}

impl TaxRuleGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<TaxRule, tax_rule_entity::Model, tax_rule_entity::ActiveModel> for TaxRuleGateway {
    async fn persist(&self, entity: TaxRule) -> Result<tax_rule_entity::ActiveModel, DbErr> {
        let active_model = TaxRuleEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            TaxRuleQuery::delete_many().filter(tax_rule_entity::Column::Id.eq(id)),
            tax_rule_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<tax_rule_entity::Model>, DbErr> {
        tenant_select(TaxRuleQuery::find(), tax_rule_entity::Column::TenantId)
            .filter(tax_rule_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<tax_rule_entity::Model>, DbErr> {
        tenant_select(TaxRuleQuery::find(), tax_rule_entity::Column::TenantId)
            .filter(tax_rule_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<tax_rule_entity::Model>, DbErr> {
        tenant_select(TaxRuleQuery::find(), tax_rule_entity::Column::TenantId)
            .order_by_asc(tax_rule_entity::Column::UfOrigem)
            .order_by_asc(tax_rule_entity::Column::UfDestino)
            .all(&self.db)
            .await
    }
}
