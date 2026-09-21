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
use entity::product_entity;

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
    path = "/products",
    tag = "Product",
    responses(
        (status = 200, description = "List of products", body = [ProductJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn products(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<ProductJson>> {
    let mut query = product_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(product_entity::Column::Name)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| ProductJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                description: x.description,
                brand: x.brand,
                active: x.active,
                ncm: x.ncm,
                cest: x.cest,
                origem_mercadoria: x.origem_mercadoria,
            })
            .collect(),
    )
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProductPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    pub brand: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl ProductPageQuery {
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

const PRODUCT_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "slug",
    "brand",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/products/paged",
    tag = "Product",
    params(ProductPageQuery),
    responses(
        (status = 200, description = "Paged products", body = PagedResponse<ProductJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn products_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<ProductPageQuery>,
) -> Json<PagedResponse<ProductJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), PRODUCT_SORT_FIELDS, "name");
    let mut query = product_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(product_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(product_entity::Column::Active.eq(active));
    }

    if let Some(ref brand) = params.brand
        && !brand.trim().is_empty() {
            query = query.filter(product_entity::Column::Brand.eq(brand.trim()));
        }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(product_entity::Column::Name.like(&pattern))
                .add(product_entity::Column::Slug.like(&pattern))
                .add(product_entity::Column::Brand.like(&pattern))
                .add(product_entity::Column::Description.like(&pattern))
                .add(product_entity::Column::Ncm.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "slug" => product_entity::Column::Slug,
        "brand" => product_entity::Column::Brand,
        "active" => product_entity::Column::Active,
        "createdat" | "created_at" => product_entity::Column::CreatedAt,
        "id" => product_entity::Column::Id,
        _ => product_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(product_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(product_entity::Column::Id)
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
        .map(|x| ProductJson {
            id: x.id,
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            name: x.name,
            slug: x.slug,
            description: x.description,
            brand: x.brand,
            active: x.active,
            ncm: x.ncm,
            cest: x.cest,
            origem_mercadoria: x.origem_mercadoria,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    post,
    path = "/products",
    tag = "Product",
    request_body = ProductInputJson,
    responses(
        (status = 201, description = "Product created", body = ProductJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_product(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<ProductInputJson>,
) -> HttpResponse<(StatusCode, Json<ProductJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.name.trim().is_empty() || input.slug.trim().is_empty() || input.ncm.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = product_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        name: Set(input.name),
        slug: Set(input.slug),
        description: Set(input.description),
        brand: Set(input.brand),
        active: Set(input.active),
        ncm: Set(input.ncm),
        cest: Set(input.cest),
        origem_mercadoria: Set(input.origem_mercadoria),
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
        Json(ProductJson {
            id: saved.id,
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            name: saved.name,
            slug: saved.slug,
            description: saved.description,
            brand: saved.brand,
            active: saved.active,
            ncm: saved.ncm,
            cest: saved.cest,
            origem_mercadoria: saved.origem_mercadoria,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/products/{id}",
    tag = "Product",
    params(("id" = i64, Path)),
    request_body = ProductInputJson,
    responses(
        (status = 200, description = "Product updated", body = ProductJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_product(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<ProductInputJson>,
) -> HttpResponse<Json<ProductJson>> {
    let existing = product_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;
    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    if input.name.trim().is_empty() || input.slug.trim().is_empty() || input.ncm.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.name = Set(input.name);
    model.slug = Set(input.slug);
    model.description = Set(input.description);
    model.brand = Set(input.brand);
    model.active = Set(input.active);
    model.ncm = Set(input.ncm);
    model.cest = Set(input.cest);
    model.origem_mercadoria = Set(input.origem_mercadoria);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(ProductJson {
        id: saved.id,
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        name: saved.name,
        slug: saved.slug,
        description: saved.description,
        brand: saved.brand,
        active: saved.active,
        ncm: saved.ncm,
        cest: saved.cest,
        origem_mercadoria: saved.origem_mercadoria,
    }))
}
