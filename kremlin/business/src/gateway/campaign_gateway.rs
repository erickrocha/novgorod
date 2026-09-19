use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::campaign::{Campaign, CampaignEntityMapper};
use entity::campaign_entity;
use entity::prelude::CampaignEntity as CampaignQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CampaignGateway {
    db: DbConn,
}

impl CampaignGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<Campaign, campaign_entity::Model, campaign_entity::ActiveModel> for CampaignGateway {
    async fn persist(&self, entity: Campaign) -> Result<campaign_entity::ActiveModel, DbErr> {
        let active_model = CampaignEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CampaignQuery::delete_many().filter(campaign_entity::Column::Id.eq(id)),
            campaign_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<campaign_entity::Model>, DbErr> {
        tenant_select(CampaignQuery::find(), campaign_entity::Column::TenantId)
            .filter(campaign_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<campaign_entity::Model>, DbErr> {
        tenant_select(CampaignQuery::find(), campaign_entity::Column::TenantId)
            .filter(campaign_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<campaign_entity::Model>, DbErr> {
        tenant_select(CampaignQuery::find(), campaign_entity::Column::TenantId)
            .order_by_asc(campaign_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
