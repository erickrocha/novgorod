use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PageQuery, PagedResponse};
use crate::endpoints::json::catalog_json::*;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::{product_entity, sku_entity};

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

#[utoipa::path(
    get,
    path = "/skus",
    tag = "Sku",
    responses(
        (status = 200, description = "List of SKUs", body = [SkuJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn skus(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<SkuJson>> {
    let mut query = sku_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(sku_entity::Column::Code)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| SkuJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                product_id: x.product_id,
                code: x.code,
                variant_key: x.variant_key,
                price_cents: x.price_cents,
                compare_at_price_cents: x.compare_at_price_cents,
                weight_g: x.weight_g,
                width_mm: x.width_mm,
                height_mm: x.height_mm,
                length_mm: x.length_mm,
                active: x.active,
            })
            .collect(),
    )
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct SkuPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "product_id")]
    pub product_id: Option<i64>,
    pub active: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl SkuPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const SKU_SORT_FIELDS: &[&str] = &[
    "id",
    "code",
    "variantKey",
    "variant_key",
    "priceCents",
    "price_cents",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/skus/paged",
    tag = "Sku",
    params(SkuPageQuery),
    responses(
        (status = 200, description = "Paged SKUs", body = PagedResponse<SkuJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn skus_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<SkuPageQuery>,
) -> Json<PagedResponse<SkuJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), SKU_SORT_FIELDS, "code");
    let mut query = sku_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(sku_entity::Column::TenantId.eq(tid));
    }

    if let Some(pid) = params.product_id {
        query = query.filter(sku_entity::Column::ProductId.eq(pid));
    }

    if let Some(active) = params.active {
        query = query.filter(sku_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(sku_entity::Column::Code.like(&pattern))
                .add(sku_entity::Column::VariantKey.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "variantkey" | "variant_key" => sku_entity::Column::VariantKey,
        "pricecents" | "price_cents" => sku_entity::Column::PriceCents,
        "active" => sku_entity::Column::Active,
        "createdat" | "created_at" => sku_entity::Column::CreatedAt,
        "id" => sku_entity::Column::Id,
        _ => sku_entity::Column::Code,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(sku_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(sku_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
        .map(|x| SkuJson {
            id: x.id,
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            product_id: x.product_id,
            code: x.code,
            variant_key: x.variant_key,
            price_cents: x.price_cents,
            compare_at_price_cents: x.compare_at_price_cents,
            weight_g: x.weight_g,
            width_mm: x.width_mm,
            height_mm: x.height_mm,
            length_mm: x.length_mm,
            active: x.active,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    post,
    path = "/skus",
    tag = "Sku",
    request_body = SkuInputJson,
    responses(
        (status = 201, description = "SKU created", body = SkuJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_sku(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<SkuInputJson>,
) -> HttpResponse<(StatusCode, Json<SkuJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.code.trim().is_empty() || input.variant_key.trim().is_empty() || input.price_cents < 0
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let product = product_entity::Entity::find_by_id(input.product_id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.as_ref().map(|p| p.tenant_id) != Some(Some(tenant_id)) {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = sku_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        product_id: Set(input.product_id),
        code: Set(input.code),
        variant_key: Set(input.variant_key),
        price_cents: Set(input.price_cents),
        compare_at_price_cents: Set(input.compare_at_price_cents),
        weight_g: Set(input.weight_g),
        width_mm: Set(input.width_mm),
        height_mm: Set(input.height_mm),
        length_mm: Set(input.length_mm),
        active: Set(input.active),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };
    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok((
        StatusCode::CREATED,
        Json(SkuJson {
            id: saved.id,
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            product_id: saved.product_id,
            code: saved.code,
            variant_key: saved.variant_key,
            price_cents: saved.price_cents,
            compare_at_price_cents: saved.compare_at_price_cents,
            weight_g: saved.weight_g,
            width_mm: saved.width_mm,
            height_mm: saved.height_mm,
            length_mm: saved.length_mm,
            active: saved.active,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/skus/{id}",
    tag = "Sku",
    params(("id" = i64, Path)),
    request_body = SkuInputJson,
    responses(
        (status = 200, description = "SKU updated", body = SkuJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_sku(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<SkuInputJson>,
) -> HttpResponse<Json<SkuJson>> {
    let existing = sku_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;
    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.price_cents < 0
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let product = product_entity::Entity::find_by_id(input.product_id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.as_ref().map(|p| p.tenant_id) != Some(existing.tenant_id) {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.product_id = Set(input.product_id);
    model.code = Set(input.code);
    model.variant_key = Set(input.variant_key);
    model.price_cents = Set(input.price_cents);
    model.compare_at_price_cents = Set(input.compare_at_price_cents);
    model.weight_g = Set(input.weight_g);
    model.width_mm = Set(input.width_mm);
    model.height_mm = Set(input.height_mm);
    model.length_mm = Set(input.length_mm);
    model.active = Set(input.active);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(SkuJson {
        id: saved.id,
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        product_id: saved.product_id,
        code: saved.code,
        variant_key: saved.variant_key,
        price_cents: saved.price_cents,
        compare_at_price_cents: saved.compare_at_price_cents,
        weight_g: saved.weight_g,
        width_mm: saved.width_mm,
        height_mm: saved.height_mm,
        length_mm: saved.length_mm,
        active: saved.active,
    }))
}
