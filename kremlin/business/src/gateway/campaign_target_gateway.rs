use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::campaign_target::{CampaignTarget, CampaignTargetEntityMapper};
use entity::campaign_target_entity;
use entity::prelude::CampaignTargetEntity as CampaignTargetQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct CampaignTargetGateway {
    db: DbConn,
}

impl CampaignTargetGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_campaign_id(
        &self,
        campaign_id: i64,
    ) -> Result<Vec<campaign_target_entity::Model>, DbErr> {
        tenant_select(
            CampaignTargetQuery::find(),
            campaign_target_entity::Column::TenantId,
        )
        .filter(campaign_target_entity::Column::CampaignId.eq(campaign_id))
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<CampaignTarget, campaign_target_entity::Model, campaign_target_entity::ActiveModel>
    for CampaignTargetGateway
{
    async fn persist(
        &self,
        entity: CampaignTarget,
    ) -> Result<campaign_target_entity::ActiveModel, DbErr> {
        let active_model = CampaignTargetEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CampaignTargetQuery::delete_many()
                .filter(campaign_target_entity::Column::Id.eq(id)),
            campaign_target_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<campaign_target_entity::Model>, DbErr> {
        tenant_select(
            CampaignTargetQuery::find(),
            campaign_target_entity::Column::TenantId,
        )
        .filter(campaign_target_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<campaign_target_entity::Model>, DbErr> {
        tenant_select(
            CampaignTargetQuery::find(),
            campaign_target_entity::Column::TenantId,
        )
        .filter(campaign_target_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<campaign_target_entity::Model>, DbErr> {
        tenant_select(
            CampaignTargetQuery::find(),
            campaign_target_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
