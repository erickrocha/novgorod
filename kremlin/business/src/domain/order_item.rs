use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::order_item_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub sku_id: i64,
    pub sku_code: String,
    pub product_name: String,
    pub attributes_desc: Option<String>,
    pub quantity: i32,
    pub unit_price_cents: i32,
    pub discount_cents: i32,
    pub ncm: String,
    pub cfop: String,
    pub csosn: String,
    pub icms_rate_bp: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct OrderItemEntityMapper {}

impl EntityMapper<OrderItem, Model, ActiveModel> for OrderItemEntityMapper {
    fn build_active_model(d: OrderItem) -> ActiveModel {
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
            order_id: Set(d.order_id),
            sku_id: Set(d.sku_id),
            sku_code: Set(d.sku_code),
            product_name: Set(d.product_name),
            attributes_desc: Set(d.attributes_desc),
            quantity: Set(d.quantity),
            unit_price_cents: Set(d.unit_price_cents),
            discount_cents: Set(d.discount_cents),
            ncm: Set(d.ncm),
            cfop: Set(d.cfop),
            csosn: Set(d.csosn),
            icms_rate_bp: Set(d.icms_rate_bp),
            tax_cents: Set(d.tax_cents),
            total_cents: Set(d.total_cents),
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

    fn from_model(e: Model) -> OrderItem {
        OrderItem {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            order_id: e.order_id,
            sku_id: e.sku_id,
            sku_code: e.sku_code,
            product_name: e.product_name,
            attributes_desc: e.attributes_desc,
            quantity: e.quantity,
            unit_price_cents: e.unit_price_cents,
            discount_cents: e.discount_cents,
            ncm: e.ncm,
            cfop: e.cfop,
            csosn: e.csosn,
            icms_rate_bp: e.icms_rate_bp,
            tax_cents: e.tax_cents,
            total_cents: e.total_cents,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> OrderItem {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => OrderItem {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                order_id: e.order_id.take().unwrap_or_default(),
                sku_id: e.sku_id.take().unwrap_or_default(),
                sku_code: e.sku_code.take().unwrap_or_default(),
                product_name: e.product_name.take().unwrap_or_default(),
                attributes_desc: e.attributes_desc.take().flatten(),
                quantity: e.quantity.take().unwrap_or_default(),
                unit_price_cents: e.unit_price_cents.take().unwrap_or_default(),
                discount_cents: e.discount_cents.take().unwrap_or_default(),
                ncm: e.ncm.take().unwrap_or_default(),
                cfop: e.cfop.take().unwrap_or_default(),
                csosn: e.csosn.take().unwrap_or_default(),
                icms_rate_bp: e.icms_rate_bp.take().unwrap_or_default(),
                tax_cents: e.tax_cents.take().unwrap_or_default(),
                total_cents: e.total_cents.take().unwrap_or_default(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

