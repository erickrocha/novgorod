use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::person_json::*;
use crate::infrastructure::mapper::{Mapper, PersonAddressMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::person_address::{PersonAddress, PersonAddressEntityMapper};
use business::domain::user::User;
use business::gateway::person_address_gateway::PersonAddressGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::person_address_use_case::PersonAddressUseCase;
use entity::person_address_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const PERSON_ADDRESS_SORT_FIELDS: &[&str] = &[
    "id",
    "locality",
    "administrative_area",
    "administrativeArea",
    "postal_code",
    "postalCode",
    "country_code",
    "countryCode",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/person-addresses",
    tag = "PersonAddress",
    responses(
        (status = 200, description = "List of person addresses", body = [PersonAddressJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<PersonAddressJson>> {
    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(PersonAddressMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/person-addresses/paged",
    tag = "PersonAddress",
    params(PersonAddressPageQuery),
    responses(
        (status = 200, description = "Paged person addresses", body = PagedResponse<PersonAddressJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<PersonAddressPageQuery>,
) -> Json<PagedResponse<PersonAddressJson>> {
    let norm = NormalizedPagination::new(
        &params.to_page_query(),
        PERSON_ADDRESS_SORT_FIELDS,
        "id",
    );
    let mut query = person_address_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(person_address_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(person_address_entity::Column::TenantId.eq(tid));
    }

    if let Some(person_id) = params.person_id {
        query = query.filter(person_address_entity::Column::PersonId.eq(person_id));
    }

    if let Some(ref country_code) = params.country_code {
        query = query.filter(person_address_entity::Column::CountryCode.eq(country_code.clone()));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(person_address_entity::Column::AddressLine1.like(&pattern))
                .add(person_address_entity::Column::AddressLine2.like(&pattern))
                .add(person_address_entity::Column::Locality.like(&pattern))
                .add(person_address_entity::Column::AdministrativeArea.like(&pattern))
                .add(person_address_entity::Column::PostalCode.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "locality" => person_address_entity::Column::Locality,
        "administrative_area" | "administrativearea" => {
            person_address_entity::Column::AdministrativeArea
        }
        "postal_code" | "postalcode" => person_address_entity::Column::PostalCode,
        "country_code" | "countrycode" => person_address_entity::Column::CountryCode,
        "createdat" | "created_at" => person_address_entity::Column::CreatedAt,
        _ => person_address_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(person_address_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(person_address_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = PersonAddressEntityMapper::from_models(rows);
    let items = PersonAddressMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/person-addresses/{id}",
    tag = "PersonAddress",
    params(("id" = i64, Path, description = "Person Address ID")),
    responses(
        (status = 200, description = "Person address found", body = PersonAddressJson),
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
) -> HttpResponse<Json<PersonAddressJson>> {
    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(PersonAddressMapper::json(item)))
}

#[utoipa::path(
    get,
    path = "/person-addresses/by-person/{person_id}",
    tag = "PersonAddress",
    params(("person_id" = i64, Path, description = "Person ID")),
    responses(
        (status = 200, description = "List of addresses for person", body = [PersonAddressJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn by_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Path(person_id): Path<i64>,
) -> Json<Vec<PersonAddressJson>> {
    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_by_person_id(person_id).await;
    let filtered = if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            items.into_iter().filter(|a| a.tenant_id == Some(id)).collect()
        } else {
            Vec::new()
        }
    } else {
        items
    };
    Json(PersonAddressMapper::json_vec(filtered))
}

#[utoipa::path(
    post,
    path = "/person-addresses",
    tag = "PersonAddress",
    request_body = PersonAddressInputJson,
    responses(
        (status = 201, description = "Person address created", body = PersonAddressJson),
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
    Json(input): Json<PersonAddressInputJson>,
) -> HttpResponse<(StatusCode, Json<PersonAddressJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    let domain = PersonAddress {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        person_id: input.person_id,
        address_line1: input.address_line1,
        address_line2: input.address_line2,
        locality: input.locality,
        administrative_area: input.administrative_area,
        postal_code: input.postal_code,
        country_code: input.country_code,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((
        StatusCode::CREATED,
        Json(PersonAddressMapper::json(saved)),
    ))
}

#[utoipa::path(
    put,
    path = "/person-addresses/{id}",
    tag = "PersonAddress",
    params(("id" = i64, Path, description = "Person Address ID")),
    request_body = PersonAddressInputJson,
    responses(
        (status = 200, description = "Person address updated", body = PersonAddressJson),
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
    Json(input): Json<PersonAddressInputJson>,
) -> HttpResponse<Json<PersonAddressJson>> {
    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
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
    updated.person_id = input.person_id;
    updated.address_line1 = input.address_line1;
    updated.address_line2 = input.address_line2;
    updated.locality = input.locality;
    updated.administrative_area = input.administrative_area;
    updated.postal_code = input.postal_code;
    updated.country_code = input.country_code;

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(PersonAddressMapper::json(saved)))
}

#[utoipa::path(
    delete,
    path = "/person-addresses/{id}",
    tag = "PersonAddress",
    params(("id" = i64, Path, description = "Person Address ID")),
    responses(
        (status = 204, description = "Person address deleted"),
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
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<StatusCode> {
    let usecase = PersonAddressUseCase::new(PersonAddressGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, existing.tenant_id) != existing.tenant_id
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    usecase
        .delete_by_id(id)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(StatusCode::NO_CONTENT)
}
