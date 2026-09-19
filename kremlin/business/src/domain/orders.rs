use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::orders_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orders {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub number: String,
    pub customer_id: i64,
    pub status: String,
    pub payment_status: String,
    pub subtotal_cents: i32,
    pub discount_cents: i32,
    pub shipping_cents: i32,
    pub tax_total_cents: i32,
    pub total_cents: i32,
    pub coupon_id: Option<i64>,
    pub coupon_code: Option<String>,
    pub ship_recipient: String,
    pub ship_cep: String,
    pub ship_logradouro: String,
    pub ship_numero: String,
    pub ship_complemento: Option<String>,
    pub ship_bairro: String,
    pub ship_cidade: String,
    pub ship_uf: String,
    pub placed_at: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct OrdersEntityMapper {}

impl EntityMapper<Orders, Model, ActiveModel> for OrdersEntityMapper {
    fn build_active_model(d: Orders) -> ActiveModel {
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
            number: Set(d.number),
            customer_id: Set(d.customer_id),
            status: Set(d.status),
            payment_status: Set(d.payment_status),
            subtotal_cents: Set(d.subtotal_cents),
            discount_cents: Set(d.discount_cents),
            shipping_cents: Set(d.shipping_cents),
            tax_total_cents: Set(d.tax_total_cents),
            total_cents: Set(d.total_cents),
            coupon_id: Set(d.coupon_id),
            coupon_code: Set(d.coupon_code),
            ship_recipient: Set(d.ship_recipient),
            ship_cep: Set(d.ship_cep),
            ship_logradouro: Set(d.ship_logradouro),
            ship_numero: Set(d.ship_numero),
            ship_complemento: Set(d.ship_complemento),
            ship_bairro: Set(d.ship_bairro),
            ship_cidade: Set(d.ship_cidade),
            ship_uf: Set(d.ship_uf),
            placed_at: Set(d.placed_at),
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

    fn from_model(e: Model) -> Orders {
        Orders {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            number: e.number,
            customer_id: e.customer_id,
            status: e.status,
            payment_status: e.payment_status,
            subtotal_cents: e.subtotal_cents,
            discount_cents: e.discount_cents,
            shipping_cents: e.shipping_cents,
            tax_total_cents: e.tax_total_cents,
            total_cents: e.total_cents,
            coupon_id: e.coupon_id,
            coupon_code: e.coupon_code,
            ship_recipient: e.ship_recipient,
            ship_cep: e.ship_cep,
            ship_logradouro: e.ship_logradouro,
            ship_numero: e.ship_numero,
            ship_complemento: e.ship_complemento,
            ship_bairro: e.ship_bairro,
            ship_cidade: e.ship_cidade,
            ship_uf: e.ship_uf,
            placed_at: e.placed_at,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Orders {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Orders {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                number: e.number.take().unwrap_or_default(),
                customer_id: e.customer_id.take().unwrap_or_default(),
                status: e.status.take().unwrap_or_default(),
                payment_status: e.payment_status.take().unwrap_or_default(),
                subtotal_cents: e.subtotal_cents.take().unwrap_or_default(),
                discount_cents: e.discount_cents.take().unwrap_or_default(),
                shipping_cents: e.shipping_cents.take().unwrap_or_default(),
                tax_total_cents: e.tax_total_cents.take().unwrap_or_default(),
                total_cents: e.total_cents.take().unwrap_or_default(),
                coupon_id: e.coupon_id.take().flatten(),
                coupon_code: e.coupon_code.take().flatten(),
                ship_recipient: e.ship_recipient.take().unwrap_or_default(),
                ship_cep: e.ship_cep.take().unwrap_or_default(),
                ship_logradouro: e.ship_logradouro.take().unwrap_or_default(),
                ship_numero: e.ship_numero.take().unwrap_or_default(),
                ship_complemento: e.ship_complemento.take().flatten(),
                ship_bairro: e.ship_bairro.take().unwrap_or_default(),
                ship_cidade: e.ship_cidade.take().unwrap_or_default(),
                ship_uf: e.ship_uf.take().unwrap_or_default(),
                placed_at: e.placed_at.take().unwrap_or_default(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}
