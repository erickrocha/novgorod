use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::campaign_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CampaignMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::campaign::CampaignEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::campaign_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CAMPAIGN_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "campaignType",
    "campaign_type",
    "startsAt",
    "starts_at",
    "endsAt",
    "ends_at",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/campaigns",
    tag = "Campaign",
    responses(
        (status = 200, description = "List of campaigns", body = [CampaignJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CampaignJson>> {
    let mut query = campaign_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(campaign_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    let domains = CampaignEntityMapper::from_models(r);
    Json(CampaignMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/campaigns/paged",
    tag = "Campaign",
    params(CampaignPageQuery),
    responses(
        (status = 200, description = "Paged campaigns", body = PagedResponse<CampaignJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CampaignPageQuery>,
) -> Json<PagedResponse<CampaignJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CAMPAIGN_SORT_FIELDS, "name");
    let mut query = campaign_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(campaign_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(campaign_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(campaign_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(campaign_entity::Column::Name.like(&pattern))
                .add(campaign_entity::Column::CampaignType.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "campaigntype" | "campaign_type" => campaign_entity::Column::CampaignType,
        "startsat" | "starts_at" => campaign_entity::Column::StartsAt,
        "endsat" | "ends_at" => campaign_entity::Column::EndsAt,
        "active" => campaign_entity::Column::Active,
        "createdat" | "created_at" => campaign_entity::Column::CreatedAt,
        _ => campaign_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(campaign_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(campaign_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CampaignEntityMapper::from_models(rows);
    let items = CampaignMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/campaigns/{id}",
    tag = "Campaign",
    params(("id" = i64, Path, description = "Campaign ID")),
    responses(
        (status = 200, description = "Campaign found", body = CampaignJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<CampaignJson>> {
    let item = campaign_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&current_user, item.tenant_id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = CampaignEntityMapper::from_model(item);
    Ok(Json(CampaignMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/campaigns",
    tag = "Campaign",
    request_body = CampaignInputJson,
    responses(
        (status = 201, description = "Campaign created", body = CampaignJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<CampaignInputJson>,
) -> HttpResponse<(StatusCode, Json<CampaignJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.name.trim().is_empty() || input.campaign_type.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let model = campaign_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        name: Set(input.name.trim().to_string()),
        campaign_type: Set(input.campaign_type.trim().to_string()),
        value: Set(input.value),
        scope: Set(input.scope.trim().to_string()),
        starts_at: Set(input.starts_at),
        ends_at: Set(input.ends_at),
        active: Set(input.active.unwrap_or(true)),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CampaignEntityMapper::from_model(saved);
    Ok((StatusCode::CREATED, Json(CampaignMapper::json(domain))))
}

#[utoipa::path(
    put,
    path = "/campaigns/{id}",
    tag = "Campaign",
    params(("id" = i64, Path, description = "Campaign ID")),
    request_body = CampaignInputJson,
    responses(
        (status = 200, description = "Campaign updated", body = CampaignJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<CampaignInputJson>,
) -> HttpResponse<Json<CampaignJson>> {
    let existing = campaign_entity::Entity::find_by_id(id)
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

    let mut model = existing.into_active_model();
    model.name = Set(input.name.trim().to_string());
    model.campaign_type = Set(input.campaign_type.trim().to_string());
    model.value = Set(input.value);
    model.scope = Set(input.scope.trim().to_string());
    model.starts_at = Set(input.starts_at);
    model.ends_at = Set(input.ends_at);
    if let Some(act) = input.active {
        model.active = Set(act);
    }

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CampaignEntityMapper::from_model(saved);
    Ok(Json(CampaignMapper::json(domain)))
}
