use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::customer_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CustomerAddressMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::customer_address::CustomerAddressEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::customer_address_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CUSTOMER_ADDRESS_SORT_FIELDS: &[&str] = &[
    "id",
    "recipient",
    "cidade",
    "uf",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/customer-addresses",
    tag = "Customer",
    responses(
        (status = 200, description = "List of customer addresses", body = [CustomerAddressJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CustomerAddressJson>> {
    let mut query = customer_address_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(customer_address_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    let domains = CustomerAddressEntityMapper::from_models(r);
    Json(CustomerAddressMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/customer-addresses/paged",
    tag = "Customer",
    params(CustomerAddressPageQuery),
    responses(
        (status = 200, description = "Paged customer addresses", body = PagedResponse<CustomerAddressJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CustomerAddressPageQuery>,
) -> Json<PagedResponse<CustomerAddressJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), CUSTOMER_ADDRESS_SORT_FIELDS, "id");
    let mut query = customer_address_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(customer_address_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(customer_address_entity::Column::TenantId.eq(tid));
    }

    if let Some(cid) = params.customer_id {
        query = query.filter(customer_address_entity::Column::CustomerId.eq(cid));
    }

    if let Some(ref uf) = params.uf {
        query = query.filter(customer_address_entity::Column::Uf.eq(uf));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(customer_address_entity::Column::Recipient.like(&pattern))
                .add(customer_address_entity::Column::Cidade.like(&pattern))
                .add(customer_address_entity::Column::Logradouro.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "recipient" => customer_address_entity::Column::Recipient,
        "cidade" => customer_address_entity::Column::Cidade,
        "uf" => customer_address_entity::Column::Uf,
        "createdat" | "created_at" => customer_address_entity::Column::CreatedAt,
        _ => customer_address_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(customer_address_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(customer_address_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CustomerAddressEntityMapper::from_models(rows);
    let items = CustomerAddressMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/customer-addresses/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer Address ID")),
    responses(
        (status = 200, description = "Customer address found", body = CustomerAddressJson),
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
) -> HttpResponse<Json<CustomerAddressJson>> {
    let item = customer_address_entity::Entity::find_by_id(id)
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

    let domain = CustomerAddressEntityMapper::from_model(item);
    Ok(Json(CustomerAddressMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/customer-addresses",
    tag = "Customer",
    request_body = CustomerAddressInputJson,
    responses(
        (status = 201, description = "Customer address created", body = CustomerAddressJson),
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
    Json(input): Json<CustomerAddressInputJson>,
) -> HttpResponse<(StatusCode, Json<CustomerAddressJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.recipient.trim().is_empty()
        || input.cep.trim().is_empty()
        || input.logradouro.trim().is_empty()
        || input.numero.trim().is_empty()
        || input.cidade.trim().is_empty()
        || input.uf.trim().is_empty()
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let model = customer_address_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        customer_id: Set(input.customer_id),
        label: Set(input.label),
        recipient: Set(input.recipient.trim().to_string()),
        cep: Set(input.cep.trim().to_string()),
        logradouro: Set(input.logradouro.trim().to_string()),
        numero: Set(input.numero.trim().to_string()),
        complemento: Set(input.complemento),
        bairro: Set(input.bairro.trim().to_string()),
        cidade: Set(input.cidade.trim().to_string()),
        uf: Set(input.uf.trim().to_uppercase()),
        is_default: Set(input.is_default.unwrap_or(false)),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CustomerAddressEntityMapper::from_model(saved);
    Ok((
        StatusCode::CREATED,
        Json(CustomerAddressMapper::json(domain)),
    ))
}

#[utoipa::path(
    put,
    path = "/customer-addresses/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer Address ID")),
    request_body = CustomerAddressInputJson,
    responses(
        (status = 200, description = "Customer address updated", body = CustomerAddressJson),
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
    Json(input): Json<CustomerAddressInputJson>,
) -> HttpResponse<Json<CustomerAddressJson>> {
    let existing = customer_address_entity::Entity::find_by_id(id)
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
    model.customer_id = Set(input.customer_id);
    model.label = Set(input.label);
    model.recipient = Set(input.recipient.trim().to_string());
    model.cep = Set(input.cep.trim().to_string());
    model.logradouro = Set(input.logradouro.trim().to_string());
    model.numero = Set(input.numero.trim().to_string());
    model.complemento = Set(input.complemento);
    model.bairro = Set(input.bairro.trim().to_string());
    model.cidade = Set(input.cidade.trim().to_string());
    model.uf = Set(input.uf.trim().to_uppercase());
    if let Some(def) = input.is_default {
        model.is_default = Set(def);
    }

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CustomerAddressEntityMapper::from_model(saved);
    Ok(Json(CustomerAddressMapper::json(domain)))
}
