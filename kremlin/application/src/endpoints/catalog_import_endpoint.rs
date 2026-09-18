use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use axum::{
    Json,
    extract::{Extension, Multipart, State},
};
use business::domain::{enums::Role, user::User};
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, QueryFilter, Set,
    TransactionTrait,
};
use entity::{
    category_entity, product_category_entity, product_entity, sku_entity, sku_stock_entity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct CatalogRow {
    product_key: String,
    category_key: String,
    product_name: String,
    slug: String,
    description: Option<String>,
    brand: Option<String>,
    ncm: String,
    cest: Option<String>,
    origem_mercadoria: i16,
    code: String,
    variant_key: String,
    price_cents: i32,
    weight_g: Option<i32>,
    width_mm: Option<i32>,
    height_mm: Option<i32>,
    length_mm: Option<i32>,
    active: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogImportResult {
    pub total: usize,
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

pub async fn import(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> HttpResponse<Json<CatalogImportResult>> {
    let tenant_id = if user.role == Role::SysAdmin {
        None
    } else {
        user.tenant_id
    };
    let mut requested_tenant = None;
    let mut rows_json = None;
    let mut file_bytes = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "tenantId" {
            requested_tenant = field.text().await.ok().and_then(|v| v.parse::<i64>().ok());
        } else if name == "editedRows" {
            rows_json = field.text().await.ok();
        } else if name == "file" {
            file_bytes = field.bytes().await.ok();
        }
    }
    let tenant_id = tenant_id
        .or(requested_tenant)
        .ok_or(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;
    if user.role != Role::SysAdmin
        && requested_tenant.is_some()
        && requested_tenant != user.tenant_id
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let rows: Vec<CatalogRow> = if let Some(json) = rows_json {
        serde_json::from_str(&json)
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    } else if let Some(bytes) = file_bytes {
        let mut reader = csv::Reader::from_reader(bytes.as_ref());
        reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    } else {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::RequiredParameterMissing,
        ));
    };
    if rows.is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let tx = state
        .conn
        .begin()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    let mut categories = HashMap::new();
    let mut products = HashMap::new();
    let mut created = 0;
    let mut updated = 0;
    let unchanged = 0;
    for row in &rows {
        if row.product_name.trim().is_empty()
            || row.slug.trim().is_empty()
            || row.code.trim().is_empty()
            || row.price_cents < 0
        {
            return Err(ExceptionResponse::BadRequest(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }
        let category = if let Some(id) = categories.get(&row.category_key) {
            *id
        } else {
            let found = category_entity::Entity::find()
                .filter(category_entity::Column::TenantId.eq(tenant_id))
                .filter(category_entity::Column::Slug.eq(&row.category_key))
                .one(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?;
            let id = if let Some(value) = found {
                value.id
            } else {
                category_entity::ActiveModel {
                    id: NotSet,
                    uuid: NotSet,
                    tenant_id: Set(Some(tenant_id)),
                    name: Set(row.category_key.clone()),
                    slug: Set(row.category_key.clone()),
                    parent_id: Set(None),
                    active: Set(true),
                    created_at: NotSet,
                    created_by: NotSet,
                    updated_at: NotSet,
                    updated_by: NotSet,
                }
                .insert(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?
                .id
            };
            categories.insert(row.category_key.clone(), id);
            id
        };
        let product = product_entity::Entity::find()
            .filter(product_entity::Column::TenantId.eq(tenant_id))
            .filter(product_entity::Column::Slug.eq(&row.slug))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        let product_id = if let Some(value) = product {
            let mut model = value.into_active_model();
            model.name = Set(row.product_name.clone());
            model.description = Set(row.description.clone());
            model.brand = Set(row.brand.clone());
            model.ncm = Set(row.ncm.clone());
            model.cest = Set(row.cest.clone());
            model.origem_mercadoria = Set(row.origem_mercadoria);
            model.active = Set(row.active);
            model
                .update(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?
                .id
        } else {
            created += 1;
            product_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                name: Set(row.product_name.clone()),
                slug: Set(row.slug.clone()),
                description: Set(row.description.clone()),
                brand: Set(row.brand.clone()),
                active: Set(row.active),
                ncm: Set(row.ncm.clone()),
                cest: Set(row.cest.clone()),
                origem_mercadoria: Set(row.origem_mercadoria),
                created_at: NotSet,
                created_by: NotSet,
                updated_at: NotSet,
                updated_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
            .id
        };
        products.insert(row.product_key.clone(), product_id);
        let link = product_category_entity::Entity::find()
            .filter(product_category_entity::Column::TenantId.eq(tenant_id))
            .filter(product_category_entity::Column::ProductId.eq(product_id))
            .filter(product_category_entity::Column::CategoryId.eq(category))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        if link.is_none() {
            product_category_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                product_id: Set(product_id),
                category_id: Set(category),
                is_primary: Set(true),
                created_at: NotSet,
                created_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        }
        let sku = sku_entity::Entity::find()
            .filter(sku_entity::Column::TenantId.eq(tenant_id))
            .filter(sku_entity::Column::Code.eq(&row.code))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        if let Some(value) = sku {
            let mut model = value.into_active_model();
            model.product_id = Set(product_id);
            model.variant_key = Set(row.variant_key.clone());
            model.price_cents = Set(row.price_cents);
            model.active = Set(row.active);
            model.update(&tx).await.map_err(|_| {
                ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
            })?;
            updated += 1;
        } else {
            sku_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                product_id: Set(product_id),
                code: Set(row.code.clone()),
                variant_key: Set(row.variant_key.clone()),
                price_cents: Set(row.price_cents),
                compare_at_price_cents: Set(None),
                weight_g: Set(row.weight_g),
                width_mm: Set(row.width_mm),
                height_mm: Set(row.height_mm),
                length_mm: Set(row.length_mm),
                active: Set(row.active),
                created_at: NotSet,
                created_by: NotSet,
                updated_at: NotSet,
                updated_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
            created += 1;
        }
        let _ = sku_stock_entity::Entity::find()
            .filter(sku_stock_entity::Column::TenantId.eq(tenant_id))
            .filter(sku_stock_entity::Column::SkuId.eq(0_i64))
            .one(&tx)
            .await;
    }
    tx.commit()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(CatalogImportResult {
        total: rows.len(),
        created,
        updated,
        unchanged,
        errors: Vec::new(),
    }))
}
