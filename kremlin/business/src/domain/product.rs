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

#[derive(Debug, Clone, Default)]
pub struct ProductSearchQuery {
    pub cursor: Option<i64>,
    pub limit: Option<u64>,
    pub q: Option<String>,
    pub active: Option<bool>,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
    pub sort_by: Option<String>,
}

pub struct WebStoreProductDto {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub price_cents: Option<i32>,
    pub compare_at_price_cents: Option<i32>,
    pub primary_image_url: Option<String>,
    pub primary_image_alt: Option<String>,
    pub category_slugs: Vec<String>,
    pub rating: Option<f32>,
    pub review_count: Option<i32>,
    pub is_featured: bool,
    pub is_new: bool,
}

#[derive(Debug, Clone)]
pub struct WebStoreSellerDto {
    pub id: i64,
    pub business_name: String,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub web_site: Option<String>,
    pub locality: Option<String>,
    pub administrative_area: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebStoreImageDto {
    pub id: i64,
    pub object_key: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct WebStoreSkuAttributeValueDto {
    pub attribute_id: i64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct WebStoreSkuDto {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub variant_key: String,
    pub price_cents: i32,
    pub compare_at_price_cents: Option<i32>,
    pub weight_g: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub length_mm: Option<i32>,
    pub active: bool,
    pub stock: i32,
    pub attributes: Vec<WebStoreSkuAttributeValueDto>,
}

#[derive(Debug, Clone)]
pub struct WebStoreAttributeDto {
    pub id: i64,
    pub attribute_id: i64,
    pub name: String,
    pub display_type: String,
    pub required: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct WebStoreProductDetailDto {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub active: bool,
    pub ncm: String,
    pub cest: Option<String>,
    pub origem_mercadoria: i16,
    pub seller: WebStoreSellerDto,
    pub images: Vec<WebStoreImageDto>,
    pub skus: Vec<WebStoreSkuDto>,
    pub attributes: Vec<WebStoreAttributeDto>,
    pub category_slugs: Vec<String>,
    pub rating: Option<f32>,
    pub review_count: Option<i32>,
}

