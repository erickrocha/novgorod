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
use entity::category_entity;

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
    path = "/categories",
    tag = "Category",
    responses(
        (status = 200, description = "List of categories", body = [CategoryJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CategoryJson>> {
    let mut query = category_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(category_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(category_entity::Column::Name)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CategoryJson {
                id: Some(x.id),
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                parent_id: x.parent_id,
                active: x.active,
            })
            .collect(),
    )
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CategoryPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "parent_id")]
    pub parent_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CategoryPageQuery {
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

const CATEGORY_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "slug",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/categories/paged",
    tag = "Category",
    params(CategoryPageQuery),
    responses(
        (status = 200, description = "Paged categories", body = PagedResponse<CategoryJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn categories_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CategoryPageQuery>,
) -> Json<PagedResponse<CategoryJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CATEGORY_SORT_FIELDS, "name");
    let mut query = category_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(category_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(category_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(category_entity::Column::Active.eq(active));
    }

    if let Some(parent_id) = params.parent_id {
        query = query.filter(category_entity::Column::ParentId.eq(parent_id));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(category_entity::Column::Name.like(&pattern))
                .add(category_entity::Column::Slug.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "slug" => category_entity::Column::Slug,
        "active" => category_entity::Column::Active,
        "createdat" | "created_at" => category_entity::Column::CreatedAt,
        "id" => category_entity::Column::Id,
        _ => category_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(category_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(category_entity::Column::Id)
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
        .map(|x| CategoryJson {
            id: Some(x.id),
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            name: x.name,
            slug: x.slug,
            parent_id: x.parent_id,
            active: x.active,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    post,
    path = "/categories",
    tag = "Category",
    request_body = CategoryInputJson,
    responses(
        (status = 201, description = "Category created", body = CategoryJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_category(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<CategoryInputJson>,
) -> HttpResponse<(StatusCode, Json<CategoryJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.name.trim().is_empty() || input.slug.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = category_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        name: Set(input.name),
        slug: Set(input.slug),
        parent_id: Set(input.parent_id),
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
        Json(CategoryJson {
            id: Some(saved.id),
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            name: saved.name,
            slug: saved.slug,
            parent_id: saved.parent_id,
            active: saved.active,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/categories/{id}",
    tag = "Category",
    params(("id" = i64, Path)),
    request_body = CategoryInputJson,
    responses(
        (status = 200, description = "Category updated", body = CategoryJson)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_category(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<CategoryInputJson>,
) -> HttpResponse<Json<CategoryJson>> {
    let existing = category_entity::Entity::find_by_id(id)
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
    if input.name.trim().is_empty() || input.slug.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.name = Set(input.name);
    model.slug = Set(input.slug);
    model.parent_id = Set(input.parent_id);
    model.active = Set(input.active);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(CategoryJson {
        id: Some(saved.id),
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        name: saved.name,
        slug: saved.slug,
        parent_id: saved.parent_id,
        active: saved.active,
    }))
}
