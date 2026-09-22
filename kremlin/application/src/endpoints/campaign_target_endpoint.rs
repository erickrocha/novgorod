use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::campaign_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CampaignTargetMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::campaign_target::{CampaignTarget, CampaignTargetEntityMapper};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::campaign_target_gateway::CampaignTargetGateway;
use business::sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::campaign_target_use_case::CampaignTargetUseCase;
use entity::campaign_target_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CAMPAIGN_TARGET_SORT_FIELDS: &[&str] = &[
    "id",
    "campaignId",
    "campaign_id",
    "targetType",
    "target_type",
    "targetId",
    "target_id",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/campaign-targets",
    tag = "Campaign",
    responses(
        (status = 200, description = "List of campaign targets", body = [CampaignTargetJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CampaignTargetJson>> {
    let usecase = CampaignTargetUseCase::new(CampaignTargetGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(CampaignTargetMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/campaign-targets/paged",
    tag = "Campaign",
    params(CampaignTargetPageQuery),
    responses(
        (status = 200, description = "Paged campaign targets", body = PagedResponse<CampaignTargetJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CampaignTargetPageQuery>,
) -> Json<PagedResponse<CampaignTargetJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), CAMPAIGN_TARGET_SORT_FIELDS, "id");
    let mut query = campaign_target_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(campaign_target_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(campaign_target_entity::Column::TenantId.eq(tid));
    }

    if let Some(cid) = params.campaign_id {
        query = query.filter(campaign_target_entity::Column::CampaignId.eq(cid));
    }

    if let Some(ref tt) = params.target_type {
        query = query.filter(campaign_target_entity::Column::TargetType.eq(tt));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "campaignid" | "campaign_id" => campaign_target_entity::Column::CampaignId,
        "targettype" | "target_type" => campaign_target_entity::Column::TargetType,
        "targetid" | "target_id" => campaign_target_entity::Column::TargetId,
        "createdat" | "created_at" => campaign_target_entity::Column::CreatedAt,
        _ => campaign_target_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(campaign_target_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(campaign_target_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CampaignTargetEntityMapper::from_models(rows);
    let items = CampaignTargetMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/campaign-targets/{id}",
    tag = "Campaign",
    params(("id" = i64, Path, description = "Campaign Target ID")),
    responses(
        (status = 200, description = "Campaign target found", body = CampaignTargetJson),
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
) -> HttpResponse<Json<CampaignTargetJson>> {
    let usecase = CampaignTargetUseCase::new(CampaignTargetGateway::new(state.conn.as_ref().clone()));
    let item = usecase
        .find_by_id(id)
        .await
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

    Ok(Json(CampaignTargetMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/campaign-targets",
    tag = "Campaign",
    request_body = CampaignTargetInputJson,
    responses(
        (status = 201, description = "Campaign target created", body = CampaignTargetJson),
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
    Json(input): Json<CampaignTargetInputJson>,
) -> HttpResponse<(StatusCode, Json<CampaignTargetJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.target_type.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = CampaignTarget {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        campaign_id: input.campaign_id,
        target_type: input.target_type.trim().to_string(),
        target_id: input.target_id,
        created_at: None,
        created_by: None,
    };

    let usecase = CampaignTargetUseCase::new(CampaignTargetGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((
        StatusCode::CREATED,
        Json(CampaignTargetMapper::json(saved)),
    ))
}

#[utoipa::path(
    put,
    path = "/campaign-targets/{id}",
    tag = "Campaign",
    params(("id" = i64, Path, description = "Campaign Target ID")),
    request_body = CampaignTargetInputJson,
    responses(
        (status = 200, description = "Campaign target updated", body = CampaignTargetJson),
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
    Json(input): Json<CampaignTargetInputJson>,
) -> HttpResponse<Json<CampaignTargetJson>> {
    let usecase = CampaignTargetUseCase::new(CampaignTargetGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
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

    let mut updated = existing;
    updated.campaign_id = input.campaign_id;
    updated.target_type = input.target_type.trim().to_string();
    updated.target_id = input.target_id;

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(CampaignTargetMapper::json(saved)))
}
