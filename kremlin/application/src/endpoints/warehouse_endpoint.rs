use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::warehouse_json::*;
use crate::infrastructure::mapper::{Mapper, WarehouseMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::domain::warehouse::{Warehouse, WarehouseEntityMapper};
use business::gateway::warehouse_gateway::WarehouseGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::warehouse_use_case::WarehouseUseCase;
use entity::warehouse_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser | Role::Customer => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const WAREHOUSE_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "originCep",
    "origin_cep",
    "city",
    "uf",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/warehouses",
    tag = "Warehouse",
    responses(
        (status = 200, description = "List of warehouses", body = [WarehouseJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<WarehouseJson>> {
    let mut query = warehouse_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(warehouse_entity::Column::TenantId.eq(id));
        } else {
            return Json(vec![]);
        }
    }
    let rows = query
        .order_by_asc(warehouse_entity::Column::Name)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    let domains = WarehouseEntityMapper::from_models(rows);
    Json(WarehouseMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/warehouses/paged",
    tag = "Warehouse",
    params(WarehousePageQuery),
    responses(
        (status = 200, description = "Paged warehouses", body = PagedResponse<WarehouseJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<WarehousePageQuery>,
) -> Json<PagedResponse<WarehouseJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), WAREHOUSE_SORT_FIELDS, "name");
    let mut query = warehouse_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(warehouse_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    }

    if let Some(ref uf) = params.uf {
        query = query.filter(warehouse_entity::Column::Uf.eq(uf));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(warehouse_entity::Column::Name.like(&pattern))
                .add(warehouse_entity::Column::OriginCep.like(&pattern))
                .add(warehouse_entity::Column::City.like(&pattern))
                .add(warehouse_entity::Column::Uf.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "origincep" | "origin_cep" => warehouse_entity::Column::OriginCep,
        "city" => warehouse_entity::Column::City,
        "uf" => warehouse_entity::Column::Uf,
        "createdat" | "created_at" => warehouse_entity::Column::CreatedAt,
        "id" => warehouse_entity::Column::Id,
        _ => warehouse_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(warehouse_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(warehouse_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = WarehouseEntityMapper::from_models(rows);
    let items = WarehouseMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/warehouses/{id}",
    tag = "Warehouse",
    params(("id" = i64, Path, description = "Warehouse ID")),
    responses(
        (status = 200, description = "Warehouse found", body = WarehouseJson),
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
) -> HttpResponse<Json<WarehouseJson>> {
    let usecase = WarehouseUseCase::new(WarehouseGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(WarehouseMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/warehouses",
    tag = "Warehouse",
    request_body = WarehouseInputJson,
    responses(
        (status = 201, description = "Warehouse created", body = WarehouseJson),
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
    Extension(current_user): Extension<User>,
    Json(input): Json<WarehouseInputJson>,
) -> HttpResponse<Json<WarehouseJson>> {
    let tenant_id = tenant_for_write(&current_user, input.tenant_id);

    if input.name.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let origin_cep = input.origin_cep.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
    if origin_cep.len() != 8 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = Warehouse {
        id: None,
        uuid: None,
        tenant_id,
        name: input.name.trim().to_string(),
        origin_cep,
        street: input.street.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        number: input.number.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        complement: input.complement.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        district: input.district.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        city: input.city.trim().to_string(),
        uf: input.uf.trim().to_uppercase(),
        is_default: input.is_default,
        active: input.active,
        created_at: None,
        created_by: Some(current_user.email),
        updated_at: None,
        updated_by: None,
    };

    let usecase = WarehouseUseCase::new(WarehouseGateway::new(state.conn.as_ref().clone()));
    let created = usecase.create(domain).await.ok_or(ExceptionResponse::BadRequest(
        locale,
        ErrorKey::InvalidParameterValue,
    ))?;

    Ok(Json(WarehouseMapper::json(created)))
}

#[utoipa::path(
    put,
    path = "/warehouses/{id}",
    tag = "Warehouse",
    params(("id" = i64, Path, description = "Warehouse ID")),
    request_body = WarehouseInputJson,
    responses(
        (status = 200, description = "Warehouse updated", body = WarehouseJson),
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
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<WarehouseInputJson>,
) -> HttpResponse<Json<WarehouseJson>> {
    let usecase = WarehouseUseCase::new(WarehouseGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&current_user, existing.tenant_id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let origin_cep = input.origin_cep.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
    if origin_cep.len() != 8 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let tenant_id = tenant_for_write(&current_user, input.tenant_id.or(existing.tenant_id));

    let domain = Warehouse {
        id: Some(id),
        uuid: existing.uuid,
        tenant_id,
        name: input.name.trim().to_string(),
        origin_cep,
        street: input.street.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        number: input.number.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        complement: input.complement.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        district: input.district.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        city: input.city.trim().to_string(),
        uf: input.uf.trim().to_uppercase(),
        is_default: input.is_default,
        active: input.active,
        created_at: existing.created_at,
        created_by: existing.created_by,
        updated_at: None,
        updated_by: Some(current_user.email),
    };

    let updated = usecase
        .update(id, domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(Json(WarehouseMapper::json(updated)))
}

#[utoipa::path(
    delete,
    path = "/warehouses/{id}",
    tag = "Warehouse",
    params(("id" = i64, Path, description = "Warehouse ID")),
    responses(
        (status = 204, description = "Warehouse deleted"),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<StatusCode> {
    let usecase = WarehouseUseCase::new(WarehouseGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&current_user, existing.tenant_id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    usecase
        .delete_by_id(id)
        .await
        .ok_or(ExceptionResponse::InternalServerError(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(StatusCode::NO_CONTENT)
}
