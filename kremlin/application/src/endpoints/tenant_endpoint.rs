use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::commons::pagination::PagedResponse;
use crate::endpoints::json::tenant_json::TenantJson;
use crate::infrastructure::mapper::{Mapper, TenantMapper};
use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::tenant_gateway::TenantGateway;
use business::use_cases::tenant_use_case::TenantUseCase;

fn can_access_tenant(user: &User, tenant_id: i64) -> bool {
    user.tenant_id == Some(tenant_id) || (user.role == Role::SysAdmin && user.tenant_id.is_none())
}

#[utoipa::path(
    post,
    tag = "Tenant",
    path = "/tenant",
    request_body = TenantJson,
    responses(
        (status = 201, description = "Tenant created", body = TenantJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<TenantJson>,
) -> HttpResponse<(StatusCode, Json<TenantJson>)> {
    if current_user.role != Role::SysAdmin || current_user.tenant_id.is_some() {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let domain = TenantMapper::domain(payload);
    let use_case = TenantUseCase::new(TenantGateway::new(state.conn.as_ref().clone()));
    match use_case.create(domain).await {
        Ok(tenant) => Ok((StatusCode::CREATED, Json(TenantMapper::json(tenant)))),
        Err(_) => Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::TenantCreatedFailed,
        )),
    }
}

#[utoipa::path(
    get,
    tag = "Tenant",
    path = "/tenant/{id}",
    params(
        ("id" = i64, Path, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Tenant found", body = TenantJson),
        (status = 404, description = "Tenant not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<TenantJson>> {
    if !can_access_tenant(&current_user, id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::TenantNotFound,
        ));
    }
    let use_case = TenantUseCase::new(TenantGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_id(id).await {
        Ok(tenant) => Ok(Json(TenantMapper::json(tenant))),
        Err(_) => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::TenantNotFound,
        )),
    }
}

#[utoipa::path(
    get,
    tag = "Tenant",
    path = "/tenant/uuid/{uuid}",
    params(
        ("uuid" = String, Path, description = "Tenant UUID")
    ),
    responses(
        (status = 200, description = "Tenant found", body = TenantJson),
        (status = 404, description = "Tenant not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_uuid(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(uuid): Path<String>,
) -> HttpResponse<Json<TenantJson>> {
    let use_case = TenantUseCase::new(TenantGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_uuid(uuid).await {
        Ok(tenant) if can_access_tenant(&current_user, tenant.id.unwrap_or_default()) => {
            Ok(Json(TenantMapper::json(tenant)))
        }
        Ok(_) => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::TenantNotFound,
        )),
        Err(_) => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::TenantNotFound,
        )),
    }
}

#[utoipa::path(
    get,
    tag = "Tenant",
    path = "/tenant",
    responses(
        (status = 200, description = "List of tenants", body = Vec<TenantJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    state: State<AppState>,
    Extension(current_user): Extension<User>,
) -> HttpResponse<Json<Vec<TenantJson>>> {
    let use_case = TenantUseCase::new(TenantGateway::new(state.conn.as_ref().clone()));
    if let Some(tenant_id) = current_user.tenant_id {
        return match use_case.find_by_id(tenant_id).await {
            Ok(tenant) => Ok(Json(vec![TenantMapper::json(tenant)])),
            Err(_) => Ok(Json(Vec::new())),
        };
    }
    if current_user.role != Role::SysAdmin {
        return Ok(Json(Vec::new()));
    }
    match use_case.find_all().await {
        Ok(tenants) => Ok(Json(TenantMapper::json_vec(tenants))),
        Err(_) => Ok(Json(Vec::new())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TenantPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "country_code")]
    pub country_code: Option<String>,
}

impl TenantPageQuery {
    pub fn to_page_query(&self) -> crate::commons::pagination::PageQuery {
        crate::commons::pagination::PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const TENANT_SORT_FIELDS: &[&str] = &[
    "id",
    "businessName",
    "business_name",
    "companyName",
    "company_name",
    "taxId",
    "tax_id",
    "email",
    "countryCode",
    "country_code",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    tag = "Tenant",
    path = "/tenant/paged",
    params(TenantPageQuery),
    responses(
        (status = 200, description = "Paged tenants", body = PagedResponse<TenantJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    state: State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<TenantPageQuery>,
) -> HttpResponse<Json<crate::commons::pagination::PagedResponse<TenantJson>>> {
    use business::commons::entity_mapper::EntityMapper;
    use business::domain::tenant::TenantEntityMapper;
    use business::sea_orm::{
        ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    };
    use entity::tenant_entity;

    let norm = crate::commons::pagination::NormalizedPagination::new(
        &params.to_page_query(),
        TENANT_SORT_FIELDS,
        "id",
    );

    let mut query = tenant_entity::Entity::find();

    if let Some(tenant_id) = current_user.tenant_id {
        query = query.filter(tenant_entity::Column::Id.eq(tenant_id));
    } else if current_user.role != Role::SysAdmin {
        return Ok(Json(crate::commons::pagination::PagedResponse::empty(
            norm.page,
            norm.page_size,
        )));
    }

    if let Some(ref cc) = params.country_code {
        let trimmed = cc.trim().to_ascii_uppercase();
        if !trimmed.is_empty() {
            query = query.filter(tenant_entity::Column::CountryCode.eq(trimmed));
        }
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(tenant_entity::Column::BusinessName.like(&pattern))
                .add(tenant_entity::Column::CompanyName.like(&pattern))
                .add(tenant_entity::Column::TaxId.like(&pattern))
                .add(tenant_entity::Column::Email.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "businessname" | "business_name" => tenant_entity::Column::BusinessName,
        "companyname" | "company_name" => tenant_entity::Column::CompanyName,
        "taxid" | "tax_id" => tenant_entity::Column::TaxId,
        "email" => tenant_entity::Column::Email,
        "countrycode" | "country_code" => tenant_entity::Column::CountryCode,
        "createdat" | "created_at" => tenant_entity::Column::CreatedAt,
        _ => tenant_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(tenant_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(tenant_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let models = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domain_tenants = TenantEntityMapper::from_models(models);
    let items = TenantMapper::json_vec(domain_tenants);

    Ok(Json(crate::commons::pagination::PagedResponse::new(
        items,
        total,
        norm.page,
        norm.page_size,
    )))
}

#[utoipa::path(
    put,
    tag = "Tenant",
    path = "/tenant/{id}",
    params(
        ("id" = i32, Path, description = "Tenant ID")
    ),
    request_body = TenantJson,
    responses(
        (status = 200, description = "Tenant updated", body = TenantJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 404, description = "Tenant not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
    Json(payload): Json<TenantJson>,
) -> HttpResponse<Json<TenantJson>> {
    if !can_access_tenant(&current_user, id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::TenantNotFound,
        ));
    }
    let domain = TenantMapper::domain(payload);
    let use_case = TenantUseCase::new(TenantGateway::new(state.conn.as_ref().clone()));
    match use_case.update(id, domain).await {
        Ok(tenant) => Ok(Json(TenantMapper::json(tenant))),
        Err(err) => {
            if err.message.contains("not found") {
                Err(ExceptionResponse::NotFound(
                    locale,
                    ErrorKey::TenantNotFound,
                ))
            } else {
                Err(ExceptionResponse::BadRequest(
                    locale,
                    ErrorKey::TenantUpdateFailed,
                ))
            }
        }
    }
}
