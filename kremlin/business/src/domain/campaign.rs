use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::campaign_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campaign {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub campaign_type: String,
    pub value: i32,
    pub scope: String,
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct CampaignEntityMapper {}

impl EntityMapper<Campaign, Model, ActiveModel> for CampaignEntityMapper {
    fn build_active_model(d: Campaign) -> ActiveModel {
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
            name: Set(d.name),
            campaign_type: Set(d.campaign_type),
            value: Set(d.value),
            scope: Set(d.scope),
            starts_at: Set(d.starts_at),
            ends_at: Set(d.ends_at),
            active: Set(d.active),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
            updated_at: NotSet,
            updated_by: match d.updated_by {
                Some(ub) => Set(Some(ub)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> Campaign {
        Campaign {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            name: e.name,
            campaign_type: e.campaign_type,
            value: e.value,
            scope: e.scope,
            starts_at: e.starts_at,
            ends_at: e.ends_at,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Campaign {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Campaign {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                name: e.name.take().unwrap_or_default(),
                campaign_type: e.campaign_type.take().unwrap_or_default(),
                value: e.value.take().unwrap_or_default(),
                scope: e.scope.take().unwrap_or_default(),
                starts_at: e.starts_at.take().unwrap_or_default(),
                ends_at: e.ends_at.take().unwrap_or_default(),
                active: e.active.take().unwrap_or(true),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

