use chrono::Utc;
use entity::category_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set};

use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};

#[derive(Debug, Clone)]
pub struct Category {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i64>,
    pub active: bool,
}

pub struct CategoryEntityMapper {}

impl EntityMapper<Category, Model, ActiveModel> for CategoryEntityMapper {
    fn build_active_model(d: Category) -> ActiveModel {
        let now = Utc::now().naive_utc();
        ActiveModel {
            id: NotSet,
            uuid: match d.uuid {
                Some(uuid) => Set(string_to_uuid(&uuid)),
                None => NotSet,
            },
            tenant_id: Set(d.tenant_id),
            name: Set(d.name),
            slug: Set(d.slug),
            parent_id: Set(d.parent_id),
            active: Set(d.active),
            created_at: Set(now),
            created_by: Default::default(),
            updated_at: Set(now),
            updated_by: Default::default(),
        }
    }

    fn from_model(e: Model) -> Category {
        Category {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            name: e.name,
            slug: e.slug,
            parent_id: e.parent_id,
            active: e.active,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Category {
        use sea_orm::TryIntoModel;
        let model: Result<entity::category_entity::Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Category {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                name: e.name.take().unwrap(),
                parent_id: e.parent_id.take().flatten(),
                tenant_id: e.tenant_id.take().flatten(),
                slug: e.slug.take().unwrap(),
                active: e.active.take().unwrap(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn from_model_converts_i8_and_uuid() {
        let uuid = Uuid::new_v4();
        let model = Model {
            id: 3,
            uuid: uuid.clone(),
            tenant_id: None,
            name: "Eletrônicos".into(),
            slug: "eletronicos".into(),
            parent_id: None,
            active: true,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        };
        let domain = CategoryEntityMapper::from_model(model);
        assert!(domain.active);
        assert_eq!(domain.uuid.unwrap(), uuid_to_string(uuid));
    }
}
