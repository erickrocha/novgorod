use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::tax_rule_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxRule {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub uf_origem: String,
    pub uf_destino: String,
    pub ncm_prefix: Option<String>,
    pub regime: String,
    pub csosn: Option<String>,
    pub cfop: String,
    pub icms_rate_bp: i32,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct TaxRuleEntityMapper {}

impl EntityMapper<TaxRule, Model, ActiveModel> for TaxRuleEntityMapper {
    fn build_active_model(d: TaxRule) -> ActiveModel {
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
            uf_origem: Set(d.uf_origem),
            uf_destino: Set(d.uf_destino),
            ncm_prefix: Set(d.ncm_prefix),
            regime: Set(d.regime),
            csosn: Set(d.csosn),
            cfop: Set(d.cfop),
            icms_rate_bp: Set(d.icms_rate_bp),
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

    fn from_model(e: Model) -> TaxRule {
        TaxRule {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            uf_origem: e.uf_origem,
            uf_destino: e.uf_destino,
            ncm_prefix: e.ncm_prefix,
            regime: e.regime,
            csosn: e.csosn,
            cfop: e.cfop,
            icms_rate_bp: e.icms_rate_bp,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> TaxRule {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => TaxRule {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                uf_origem: e.uf_origem.take().unwrap_or_default(),
                uf_destino: e.uf_destino.take().unwrap_or_default(),
                ncm_prefix: e.ncm_prefix.take().flatten(),
                regime: e.regime.take().unwrap_or_default(),
                csosn: e.csosn.take().flatten(),
                cfop: e.cfop.take().unwrap_or_default(),
                icms_rate_bp: e.icms_rate_bp.take().unwrap_or_default(),
                active: e.active.take().unwrap_or(true),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}
