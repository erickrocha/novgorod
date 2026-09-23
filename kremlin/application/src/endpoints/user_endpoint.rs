use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::change_password_request::ChangePasswordRequest;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::user_json::UserJson;
use crate::commons::pagination::PagedResponse;
use crate::infrastructure::mapper::{Mapper, UserMapper};
use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use crate::endpoints::json::customer_json::{CustomerJson, CustomerAddressJson};
use crate::infrastructure::mapper::CustomerMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::user_gateway::UserGateway;
use business::use_cases::user_use_case::UserUseCase;

#[utoipa::path(
    post,
    tag = "User",
    path = "/user",
    request_body = UserJson,
    responses(
        (status = 201, description = "User created", body = UserJson),
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
    Json(payload): Json<UserJson>,
) -> HttpResponse<(StatusCode, Json<UserJson>)> {
    let mut domain = UserMapper::domain(payload);

    match current_user.role {
        Role::SysAdmin => {
            if domain.role == Role::SysAdmin {
                domain.tenant_id = None;
                domain.first_login = false;
            } else if domain.role == Role::TenantOwner && domain.tenant_id.is_some() {
                domain.first_login = true;
            } else {
                return Err(ExceptionResponse::BadRequest(
                    locale,
                    ErrorKey::InvalidParameterValue,
                ));
            }
        }
        Role::TenantOwner => {
            // TenantOwner can only create users for their own tenant
            if let Some(tenant_id) = current_user.tenant_id {
                domain.tenant_id = Some(tenant_id);
                domain.role = Role::TenantOwner;
                domain.first_login = true;
            } else {
                return Err(ExceptionResponse::Forbidden(
                    locale,
                    ErrorKey::RequiredHeaderValueMissing,
                ));
            }
        }
        _ => {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::RequiredHeaderValueMissing,
            ));
        }
    }

    let use_case = UserUseCase::new(UserGateway::new(state.conn.as_ref().clone()));
    let user = use_case.create(domain).await.ok_or(ExceptionResponse::BadRequest(
        locale,
        ErrorKey::RequiredParameterMissing,
    ))?;
    Ok((StatusCode::CREATED, Json(UserMapper::json(user))))
}

#[utoipa::path(
    get,
    tag = "User",
    path = "/user",
    responses(
        (status = 200, description = "List of users", body = Vec<UserJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> HttpResponse<Json<Vec<UserJson>>> {
    let use_case = UserUseCase::new(UserGateway::new(state.conn.as_ref().clone()));

    let users = match current_user.role {
        Role::SysAdmin => use_case.find_all().await,
        Role::TenantOwner => {
            if let Some(tenant_id) = current_user.tenant_id {
                use_case.find_all_by_tenant_id(tenant_id).await
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    Ok(Json(UserMapper::json_vec(users)))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct UserPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub role: Option<String>,
    pub enabled: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl UserPageQuery {
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

const USER_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "email",
    "role",
    "enabled",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    tag = "User",
    path = "/user/paged",
    params(UserPageQuery),
    responses(
        (status = 200, description = "Paged users", body = PagedResponse<UserJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<UserPageQuery>,
) -> HttpResponse<Json<PagedResponse<UserJson>>> {
    use business::commons::entity_mapper::EntityMapper;
    use business::domain::user::UserEntityMapper;
    use business::sea_orm::{
        ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    };
    use entity::user_entity;

    let norm = crate::commons::pagination::NormalizedPagination::new(
        &params.to_page_query(),
        USER_SORT_FIELDS,
        "id",
    );

    let mut query = user_entity::Entity::find();

    match current_user.role {
        Role::SysAdmin => {
            if let Some(tid) = params.tenant_id {
                query = query.filter(user_entity::Column::TenantId.eq(tid));
            }
        }
        Role::TenantOwner => {
            if let Some(tid) = current_user.tenant_id {
                query = query.filter(user_entity::Column::TenantId.eq(tid));
            } else {
                return Ok(Json(crate::commons::pagination::PagedResponse::empty(
                    norm.page,
                    norm.page_size,
                )));
            }
        }
        _ => {
            return Ok(Json(crate::commons::pagination::PagedResponse::empty(
                norm.page,
                norm.page_size,
            )));
        }
    }

    if let Some(ref role) = params.role
        && !role.trim().is_empty() {
            query = query.filter(user_entity::Column::Role.eq(role.trim()));
        }

    if let Some(enabled) = params.enabled {
        query = query.filter(user_entity::Column::Enabled.eq(enabled));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(user_entity::Column::Name.like(&pattern))
                .add(user_entity::Column::Email.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "name" => user_entity::Column::Name,
        "email" => user_entity::Column::Email,
        "role" => user_entity::Column::Role,
        "enabled" => user_entity::Column::Enabled,
        "createdat" | "created_at" => user_entity::Column::CreatedAt,
        _ => user_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(user_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(user_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let models = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domain_users = UserEntityMapper::from_models(models);
    let items = UserMapper::json_vec(domain_users);

    Ok(Json(crate::commons::pagination::PagedResponse::new(
        items,
        total,
        norm.page,
        norm.page_size,
    )))
}

#[utoipa::path(
    get,
    tag = "User",
    path = "/user/{id}",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = UserJson),
        (status = 404, description = "User not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<UserJson>> {
    let use_case = UserUseCase::new(UserGateway::new(state.conn.as_ref().clone()));
    let user = use_case.find_by_id(id).await.ok_or(ExceptionResponse::NotFound(
        locale,
        ErrorKey::RequiredParameterMissing,
    ))?;
    if current_user.role == Role::TenantOwner && user.tenant_id != current_user.tenant_id {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::RequiredHeaderValueMissing,
        ));
    }
    Ok(Json(UserMapper::json(user)))
}

#[utoipa::path(
    put,
    tag = "User",
    path = "/user/{id}",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body = UserJson,
    responses(
        (status = 200, description = "User updated", body = UserJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 404, description = "User not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
    Json(payload): Json<UserJson>,
) -> HttpResponse<Json<UserJson>> {
    let mut domain = UserMapper::domain(payload);

    let use_case = UserUseCase::new(UserGateway::new(state.conn.as_ref().clone()));

    if current_user.id == Some(id) && !domain.enabled {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    // Check permissions: SysAdmin or self-update allowed, otherwise TenantOwner restricted to same tenant
    if current_user.role != Role::SysAdmin && current_user.id != Some(id) {
        if current_user.role == Role::TenantOwner {
            let existing = use_case.find_by_id(id).await.ok_or(
                ExceptionResponse::NotFound(locale, ErrorKey::RequiredParameterMissing)
            )?;
            if existing.tenant_id != current_user.tenant_id {
                return Err(ExceptionResponse::Forbidden(
                    locale,
                    ErrorKey::RequiredHeaderValueMissing,
                ));
            }
            domain.tenant_id = current_user.tenant_id;
            domain.role = Role::TenantOwner;
        } else {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::RequiredHeaderValueMissing,
            ));
        }
    }

    if current_user.role == Role::SysAdmin {
        if domain.role == Role::SysAdmin {
            domain.tenant_id = None;
        } else if domain.role != Role::TenantOwner || domain.tenant_id.is_none() {
            return Err(ExceptionResponse::BadRequest(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }
    } else if current_user.role == Role::TenantOwner {
        domain.tenant_id = current_user.tenant_id;
        domain.role = Role::TenantOwner;
    }

    let user = use_case.update(id, domain).await.ok_or(ExceptionResponse::BadRequest(
        locale,
        ErrorKey::RequiredParameterMissing,
    ))?;
    Ok(Json(UserMapper::json(user)))
}

#[utoipa::path(
    put,
    tag = "User",
    path = "/user/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<ChangePasswordRequest>,
) -> HttpResponse<StatusCode> {
    let use_case = UserUseCase::new(UserGateway::new(state.conn.as_ref().clone()));

    let user_id = current_user.id.unwrap();

    use_case
        .change_password(user_id, payload.current_password, payload.new_password)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidCurrentPassword,
        ))?;

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub user: UserJson,
    pub customer: Option<CustomerJson>,
    pub addresses: Vec<CustomerAddressJson>,
}

#[utoipa::path(
    get,
    tag = "User",
    path = "/me",
    responses(
        (status = 200, description = "Current user info", body = MeResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> HttpResponse<Json<MeResponse>> {
    use business::sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
    use entity::{customer_entity, customer_address_entity};
    use business::domain::customer::CustomerEntityMapper;
    use business::commons::entity_mapper::EntityMapper;
    
    let mut response = MeResponse {
        user: UserMapper::json(current_user.clone()),
        customer: None,
        addresses: vec![],
    };

    if let Some(user_id) = current_user.id {
        let customer_model = customer_entity::Entity::find()
            .filter(customer_entity::Column::UserId.eq(user_id))
            .one(state.conn.as_ref())
            .await
            .unwrap_or(None);

        if let Some(model) = customer_model {
            let customer_domain = CustomerEntityMapper::from_model(model);
            let customer_id = customer_domain.id.unwrap_or(0);
            response.customer = Some(CustomerMapper::json(customer_domain));

            let address_models = customer_address_entity::Entity::find()
                .filter(customer_address_entity::Column::CustomerId.eq(customer_id))
                .all(state.conn.as_ref())
                .await
                .unwrap_or_default();
            
            // Just map them manually since there might not be a mapper
            response.addresses = address_models.into_iter().map(|a| CustomerAddressJson {
                id: a.id,
                uuid: business::commons::functions::uuid_to_string(a.uuid),
                tenant_id: a.tenant_id,
                customer_id: a.customer_id,
                label: a.label,
                recipient: a.recipient,
                cep: a.cep,
                logradouro: a.logradouro,
                numero: a.numero,
                complemento: a.complemento,
                bairro: a.bairro,
                cidade: a.cidade,
                uf: a.uf,
                is_default: a.is_default,
                created_at: Some(a.created_at),
                updated_at: Some(a.updated_at),
            }).collect();
        }
    }

    Ok(Json(response))
}
