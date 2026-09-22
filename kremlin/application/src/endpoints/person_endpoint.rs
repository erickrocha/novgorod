use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::person_json::*;
use crate::infrastructure::mapper::{Mapper, PersonMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::person::{Person, PersonEntityMapper};
use business::domain::user::User;
use business::gateway::person_gateway::PersonGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::person_use_case::PersonUseCase;
use entity::person_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const PERSON_SORT_FIELDS: &[&str] = &[
    "id",
    "first_name",
    "firstName",
    "surname",
    "email",
    "gender",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/persons",
    tag = "Person",
    responses(
        (status = 200, description = "List of persons", body = [PersonJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<PersonJson>> {
    let usecase = PersonUseCase::new(PersonGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(PersonMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/persons/paged",
    tag = "Person",
    params(PersonPageQuery),
    responses(
        (status = 200, description = "Paged persons", body = PagedResponse<PersonJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<PersonPageQuery>,
) -> Json<PagedResponse<PersonJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), PERSON_SORT_FIELDS, "surname");
    let mut query = person_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(person_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(person_entity::Column::TenantId.eq(tid));
    }

    if let Some(user_id) = params.user_id {
        query = query.filter(person_entity::Column::UserId.eq(user_id));
    }

    if let Some(ref gender) = params.gender {
        query = query.filter(person_entity::Column::Gender.eq(gender.clone()));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(person_entity::Column::FirstName.like(&pattern))
                .add(person_entity::Column::Surname.like(&pattern))
                .add(person_entity::Column::Email.like(&pattern))
                .add(person_entity::Column::Phone.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "first_name" | "firstname" => person_entity::Column::FirstName,
        "email" => person_entity::Column::Email,
        "gender" => person_entity::Column::Gender,
        "createdat" | "created_at" => person_entity::Column::CreatedAt,
        "id" => person_entity::Column::Id,
        _ => person_entity::Column::Surname,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(person_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(person_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = PersonEntityMapper::from_models(rows);
    let items = PersonMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/persons/{id}",
    tag = "Person",
    params(("id" = i64, Path, description = "Person ID")),
    responses(
        (status = 200, description = "Person found", body = PersonJson),
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
) -> HttpResponse<Json<PersonJson>> {
    let usecase = PersonUseCase::new(PersonGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(PersonMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/persons",
    tag = "Person",
    request_body = PersonInputJson,
    responses(
        (status = 201, description = "Person created", body = PersonJson),
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
    Json(input): Json<PersonInputJson>,
) -> HttpResponse<(StatusCode, Json<PersonJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.first_name.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = Person {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        user_id: input.user_id,
        first_name: input.first_name.trim().to_string(),
        surname: input.surname.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        date_of_birth: input.date_of_birth,
        gender: input.gender.map(|g| g.trim().to_string()).filter(|g| !g.is_empty()),
        avatar: input.avatar,
        phone: input.phone,
        email: input.email.map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty()),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = PersonUseCase::new(PersonGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((StatusCode::CREATED, Json(PersonMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/persons/{id}",
    tag = "Person",
    params(("id" = i64, Path, description = "Person ID")),
    request_body = PersonInputJson,
    responses(
        (status = 200, description = "Person updated", body = PersonJson),
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
    Json(input): Json<PersonInputJson>,
) -> HttpResponse<Json<PersonJson>> {
    let usecase = PersonUseCase::new(PersonGateway::new(state.conn.as_ref().clone()));
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

    if input.first_name.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let mut updated = existing;
    updated.user_id = input.user_id;
    updated.first_name = input.first_name.trim().to_string();
    updated.surname = input.surname.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    updated.date_of_birth = input.date_of_birth;
    updated.gender = input.gender.map(|g| g.trim().to_string()).filter(|g| !g.is_empty());
    if let Some(avatar) = input.avatar {
        updated.avatar = Some(avatar);
    }
    if let Some(phone) = input.phone {
        updated.phone = Some(phone);
    }
    if let Some(email) = input.email {
        updated.email = Some(email.trim().to_lowercase());
    }

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(PersonMapper::json(saved)))
}

#[utoipa::path(
    delete,
    path = "/persons/{id}",
    tag = "Person",
    params(("id" = i64, Path, description = "Person ID")),
    responses(
        (status = 204, description = "Person deleted"),
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
    let usecase = PersonUseCase::new(PersonGateway::new(state.conn.as_ref().clone()));
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
