use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::product_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub active: bool,
    pub ncm: String,
    pub cest: Option<String>,
    pub origem_mercadoria: i16,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct ProductEntityMapper {}

impl EntityMapper<Product, Model, ActiveModel> for ProductEntityMapper {
    fn build_active_model(d: Product) -> ActiveModel {
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
            slug: Set(d.slug),
            description: Set(d.description),
            brand: Set(d.brand),
            active: Set(d.active),
            ncm: Set(d.ncm),
            cest: Set(d.cest),
            origem_mercadoria: Set(d.origem_mercadoria),
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

    fn from_model(e: Model) -> Product {
        Product {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            name: e.name,
            slug: e.slug,
            description: e.description,
            brand: e.brand,
            active: e.active,
            ncm: e.ncm,
            cest: e.cest,
            origem_mercadoria: e.origem_mercadoria,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Product {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Product {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                name: e.name.take().unwrap_or_default(),
                slug: e.slug.take().unwrap_or_default(),
                description: e.description.take().flatten(),
                brand: e.brand.take().flatten(),
                active: e.active.take().unwrap_or(true),
                ncm: e.ncm.take().unwrap_or_default(),
                cest: e.cest.take().flatten(),
                origem_mercadoria: e.origem_mercadoria.take().unwrap_or_default(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

