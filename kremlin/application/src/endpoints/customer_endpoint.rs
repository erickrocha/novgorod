use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::customer_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ErrorResponseJson, ForbiddenErrorJson, InternalServerErrorJson,
    NotFoundErrorJson, UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CustomerMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::customer::CustomerEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::customer_gateway::CustomerGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::customer_use_case::CustomerUseCase;
use entity::{customer_entity, user_entity};

const CUSTOMER_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "email",
    "active",
    "createdAt",
    "created_at",
];

async fn validate_customer_user_link(
    state: &AppState,
    locale: Locale,
    user_id: Option<i64>,
    customer_tenant_id: Option<i64>,
    current_customer_id: Option<i64>,
) -> Result<(), ExceptionResponse> {
    let Some(user_id) = user_id else {
        return Ok(());
    };

    let user = user_entity::Entity::find_by_id(user_id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| {
            ExceptionResponse::InternalServerError(locale, ErrorKey::InvalidParameterValue)
        })?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if user.tenant_id != customer_tenant_id {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let mut linked_customer =
        customer_entity::Entity::find().filter(customer_entity::Column::UserId.eq(user_id));
    if let Some(current_customer_id) = current_customer_id {
        linked_customer =
            linked_customer.filter(customer_entity::Column::Id.ne(current_customer_id));
    }

    if linked_customer
        .one(state.conn.as_ref())
        .await
        .map_err(|_| {
            ExceptionResponse::InternalServerError(locale, ErrorKey::InvalidParameterValue)
        })?
        .is_some()
    {
        return Err(ExceptionResponse::Conflict(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/customers",
    tag = "Customer",
    responses(
        (status = 200, description = "List of customers", body = [CustomerJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 404, description = "Linked User not found", body = ErrorResponseJson),
        (status = 409, description = "User already linked to another Customer", body = ErrorResponseJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CustomerJson>> {
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let items = use_case.find_all().await;
    Json(CustomerMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/customers/paged",
    tag = "Customer",
    params(CustomerPageQuery),
    responses(
        (status = 200, description = "Paged customers", body = PagedResponse<CustomerJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 409, description = "User already linked to another Customer", body = ErrorResponseJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CustomerPageQuery>,
) -> Json<PagedResponse<CustomerJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CUSTOMER_SORT_FIELDS, "name");
    let mut query = customer_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(customer_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(customer_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(customer_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(customer_entity::Column::Name.like(&pattern))
                .add(customer_entity::Column::Email.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "email" => customer_entity::Column::Email,
        "active" => customer_entity::Column::Active,
        "createdat" | "created_at" => customer_entity::Column::CreatedAt,
        "id" => customer_entity::Column::Id,
        _ => customer_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(customer_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(customer_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CustomerEntityMapper::from_models(rows);
    let items = CustomerMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/customers/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer ID")),
    responses(
        (status = 200, description = "Customer found", body = CustomerJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(State(state): State<AppState>,Extension(locale): Extension<Locale>,Path(id): Path<i64>) -> HttpResponse<Json<CustomerJson>> {
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let item = use_case.find_by_id(id).await;
    match item {
        Some(item) => Ok(Json(CustomerMapper::json(item))),
        None => Err(ExceptionResponse::NotFound(locale,ErrorKey::InvalidParameterValue))
    }
}

#[utoipa::path(
    post,
    path = "/customers",
    tag = "Customer",
    request_body = CustomerJson,
    responses(
        (status = 201, description = "Customer created", body = CustomerJson),
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
    Json(mut payload): Json<CustomerJson>,
) -> HttpResponse<(StatusCode, Json<CustomerJson>)> {
    if current_user.role == Role::Customer {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    if matches!(current_user.role, Role::TenantOwner | Role::TenantUser) {
        let Some(tenant_id) = current_user.tenant_id else {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        };

        if payload
            .tenant_id
            .is_some_and(|requested| requested != tenant_id)
            || payload.user_id.is_some()
        {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }

        payload.tenant_id = Some(tenant_id);
    }

    validate_customer_user_link(&state, locale, payload.user_id, payload.tenant_id, None).await?;

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let customer = CustomerMapper::domain(payload);
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let saved = use_case.create(customer).await;

    match saved {
        Some(domain) => Ok((StatusCode::CREATED, Json(CustomerMapper::json(domain)))),
        None => Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        )),
    }
}

#[utoipa::path(
    put,
    path = "/customers/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer ID")),
    request_body = CustomerJson,
    responses(
        (status = 200, description = "Customer updated", body = CustomerJson),
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
    Json(mut payload): Json<CustomerJson>,
) -> HttpResponse<Json<CustomerJson>> {
    if current_user.role == Role::Customer {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let existing = customer_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| {
            ExceptionResponse::InternalServerError(locale, ErrorKey::InvalidParameterValue)
        })?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    let tenant_update_scope = match current_user.role {
        Role::SysAdmin => {
            validate_customer_user_link(
                &state,
                locale,
                payload.user_id,
                payload.tenant_id,
                Some(id),
            )
            .await?;
            None
        }
        Role::TenantOwner | Role::TenantUser => {
            let Some(tenant_id) = current_user.tenant_id else {
                return Err(ExceptionResponse::Forbidden(
                    locale,
                    ErrorKey::InvalidParameterValue,
                ));
            };

            if existing.tenant_id != Some(tenant_id)
                || payload
                    .tenant_id
                    .is_some_and(|requested| requested != tenant_id)
                || payload
                    .user_id
                    .is_some_and(|requested| Some(requested) != existing.user_id)
            {
                return Err(ExceptionResponse::Forbidden(
                    locale,
                    ErrorKey::InvalidParameterValue,
                ));
            }

            payload.tenant_id = existing.tenant_id;
            payload.user_id = existing.user_id;
            Some(tenant_id)
        }
        Role::Customer => unreachable!("Customer role was rejected above"),
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let customer = CustomerMapper::domain(payload);

    if let Some(tenant_id) = tenant_update_scope {
        let saved = use_case
            .update_in_tenant(id, tenant_id, customer)
            .await
            .map_err(|_| {
                ExceptionResponse::InternalServerError(locale, ErrorKey::InvalidParameterValue)
            })?;
        return saved.map(|saved| Json(CustomerMapper::json(saved))).ok_or(
            ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
        );
    }

    let saved = use_case.update(id, customer).await;

    match saved {
        Some(saved) => Ok(Json(CustomerMapper::json(saved))),
        None => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        )),
    }
}
