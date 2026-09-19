use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::campaign_target_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignTarget {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub campaign_id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

pub struct CampaignTargetEntityMapper {}

impl EntityMapper<CampaignTarget, Model, ActiveModel> for CampaignTargetEntityMapper {
    fn build_active_model(d: CampaignTarget) -> ActiveModel {
        ActiveModel {
            id: match d.id {
                Some(id) => Set(id),
                None => NotSet,
            },
            uuid: match d.uuid {
                Some(uuid) => Set(string_to_uuid(&uuid)),
                None => NotSet,
            },
            tenant_id: Set(d.tenant_id),
            campaign_id: Set(d.campaign_id),
            target_type: Set(d.target_type),
            target_id: Set(d.target_id),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> CampaignTarget {
        CampaignTarget {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            campaign_id: e.campaign_id,
            target_type: e.target_type,
            target_id: e.target_id,
            created_at: Some(e.created_at),
            created_by: e.created_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> CampaignTarget {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => CampaignTarget {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                campaign_id: e.campaign_id.take().unwrap_or_default(),
                target_type: e.target_type.take().unwrap_or_default(),
                target_id: e.target_id.take().unwrap_or_default(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
            },
        }
    }
}

