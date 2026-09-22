use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PageQuery, PagedResponse};
use crate::endpoints::json::catalog_json::*;
use crate::infrastructure::mapper::{Mapper, SkuMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::product_gateway::ProductGateway;
use business::gateway::sku_gateway::SkuGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::product_use_case::ProductUseCase;
use business::use_cases::sku_use_case::SkuUseCase;
use entity::sku_entity;

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
    Extension(_current_user): Extension<User>,
) -> Json<Vec<SkuJson>> {
    let usecase = SkuUseCase::new(SkuGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(SkuMapper::json_vec(items))
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
    let product_usecase = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));
    let product = product_usecase
        .find_by_id(input.product_id)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.tenant_id != Some(tenant_id) {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let sku = SkuMapper::domain(SkuJson {
        id: 0,
        uuid: String::new(),
        tenant_id: Some(tenant_id),
        product_id: input.product_id,
        code: input.code,
        variant_key: input.variant_key,
        price_cents: input.price_cents,
        compare_at_price_cents: input.compare_at_price_cents,
        weight_g: input.weight_g,
        width_mm: input.width_mm,
        height_mm: input.height_mm,
        length_mm: input.length_mm,
        active: input.active,
    });

    let usecase = SkuUseCase::new(SkuGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(sku)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok((StatusCode::CREATED, Json(SkuMapper::json(saved))))
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
    let usecase = SkuUseCase::new(SkuGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;
    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.price_cents < 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let product_usecase = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));
    let product = product_usecase
        .find_by_id(input.product_id)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.tenant_id != existing.tenant_id {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let sku = SkuMapper::domain(SkuJson {
        id,
        uuid: existing.uuid.unwrap_or_default(),
        tenant_id: existing.tenant_id,
        product_id: input.product_id,
        code: input.code,
        variant_key: input.variant_key,
        price_cents: input.price_cents,
        compare_at_price_cents: input.compare_at_price_cents,
        weight_g: input.weight_g,
        width_mm: input.width_mm,
        height_mm: input.height_mm,
        length_mm: input.length_mm,
        active: input.active,
    });

    let saved = usecase
        .update(id, sku)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(Json(SkuMapper::json(saved)))
}
