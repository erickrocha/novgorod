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
use business::domain::campaign::{Campaign, CampaignEntityMapper};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::campaign_gateway::CampaignGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::campaign_use_case::CampaignUseCase;
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
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CampaignJson>> {
    let usecase = CampaignUseCase::new(CampaignGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(CampaignMapper::json_vec(items))
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
    let usecase = CampaignUseCase::new(CampaignGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(CampaignMapper::json(item)))
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

    let domain = Campaign {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        name: input.name.trim().to_string(),
        campaign_type: input.campaign_type.trim().to_string(),
        value: input.value,
        scope: input.scope.trim().to_string(),
        starts_at: input.starts_at,
        ends_at: input.ends_at,
        active: input.active.unwrap_or(true),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = CampaignUseCase::new(CampaignGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((StatusCode::CREATED, Json(CampaignMapper::json(saved))))
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
    let usecase = CampaignUseCase::new(CampaignGateway::new(state.conn.as_ref().clone()));
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
    updated.name = input.name.trim().to_string();
    updated.campaign_type = input.campaign_type.trim().to_string();
    updated.value = input.value;
    updated.scope = input.scope.trim().to_string();
    updated.starts_at = input.starts_at;
    updated.ends_at = input.ends_at;
    if let Some(act) = input.active {
        updated.active = act;
    }

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(CampaignMapper::json(saved)))
}
