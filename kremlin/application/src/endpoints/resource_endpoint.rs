use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, InternalServerErrorJson, UnauthorizedErrorJson,
};
use crate::endpoints::json::person_json::{
    AvatarPresignRequest, AvatarPresignResponse, PersonInputJson, PersonJson, ResourceProfileJson,
};
use crate::infrastructure::mapper::{Mapper, PersonMapper, UserMapper};
use axum::extract::{Extension, State};
use axum::Json;
use business::commons::entity_mapper::EntityMapper;
use business::domain::person::{Person, PersonEntityMapper};
use business::domain::user::User;
use business::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, QueryFilter, Set};
use business::use_cases::user_use_case::UserUseCase;
use entity::person_entity;

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[utoipa::path(
    get,
    path = "/resource/profile",
    tag = "Resource",
    responses(
        (status = 200, description = "Current authenticated user profile and person details", body = ResourceProfileJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
) -> HttpResponse<Json<ResourceProfileJson>> {
    let user_id = user.id.ok_or(ExceptionResponse::Unauthorized(
        locale,
        ErrorKey::BadCredentials,
    ))?;

    let existing_person = person_entity::Entity::find()
        .filter(person_entity::Column::UserId.eq(user_id))
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let person_domain = match existing_person {
        Some(model) => PersonEntityMapper::from_model(model),
        None => {
            let person_to_save = UserUseCase::build_person_for_user(&user).unwrap_or_else(|| Person {
                id: None,
                uuid: None,
                tenant_id: user.tenant_id,
                user_id,
                first_name: user
                    .name
                    .clone()
                    .unwrap_or_else(|| user.email.split('@').next().unwrap_or("User").to_string()),
                surname: None,
                date_of_birth: None,
                gender: None,
                avatar: None,
                phone: None,
                email: None,
                created_at: None,
                created_by: user.created_by.clone(),
                updated_at: None,
                updated_by: None,
            });

            let am = PersonEntityMapper::build_active_model(person_to_save);
            let saved = am
                .insert(state.conn.as_ref())
                .await
                .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
            PersonEntityMapper::from_model(saved)
        }
    };

    let person_json = PersonMapper::json_with_storage(person_domain, &state.storage);
    let user_json = UserMapper::json(user);

    Ok(Json(ResourceProfileJson {
        user: user_json,
        person: Some(person_json),
    }))
}

#[utoipa::path(
    put,
    path = "/resource/profile",
    tag = "Resource",
    request_body = PersonInputJson,
    responses(
        (status = 200, description = "Profile updated successfully", body = PersonJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<PersonInputJson>,
) -> HttpResponse<Json<PersonJson>> {
    let user_id = user.id.ok_or(ExceptionResponse::Unauthorized(
        locale,
        ErrorKey::BadCredentials,
    ))?;

    if input.first_name.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let existing_person = person_entity::Entity::find()
        .filter(person_entity::Column::UserId.eq(user_id))
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let saved = match existing_person {
        Some(existing) => {
            let mut model = existing.into_active_model();
            model.first_name = Set(input.first_name.trim().to_string());
            model.surname = Set(input.surname.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
            model.date_of_birth = Set(input.date_of_birth);
            model.gender = Set(input.gender.map(|g| g.trim().to_string()).filter(|g| !g.is_empty()));
            if let Some(avatar) = input.avatar {
                model.avatar = Set(Some(avatar));
            }
            if let Some(phone) = input.phone {
                model.phone = Set(Some(phone));
            }
            if let Some(email) = input.email {
                model.email = Set(Some(email.trim().to_lowercase()));
            }
            model
                .update(state.conn.as_ref())
                .await
                .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
        }
        None => {
            let model = person_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(user.tenant_id),
                user_id: Set(user_id),
                first_name: Set(input.first_name.trim().to_string()),
                surname: Set(input.surname.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())),
                date_of_birth: Set(input.date_of_birth),
                gender: Set(input.gender.map(|g| g.trim().to_string()).filter(|g| !g.is_empty())),
                avatar: Set(input.avatar),
                phone: Set(input.phone),
                email: Set(input.email.map(|e| e.trim().to_lowercase())),
                created_at: NotSet,
                created_by: Set(user.created_by.clone()),
                updated_at: NotSet,
                updated_by: NotSet,
            };
            model
                .insert(state.conn.as_ref())
                .await
                .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
        }
    };

    let domain = PersonEntityMapper::from_model(saved);
    Ok(Json(PersonMapper::json_with_storage(domain, &state.storage)))
}

#[utoipa::path(
    post,
    path = "/resource/avatar/presign",
    tag = "Resource",
    request_body = AvatarPresignRequest,
    responses(
        (status = 200, description = "Presigned S3 upload URL for avatar generated", body = AvatarPresignResponse),
        (status = 400, description = "Invalid request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn presign_avatar(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(body): Json<AvatarPresignRequest>,
) -> HttpResponse<Json<AvatarPresignResponse>> {
    let user_id = user.id.ok_or(ExceptionResponse::Unauthorized(
        locale,
        ErrorKey::BadCredentials,
    ))?;

    if body.original_filename.trim().is_empty() || body.mime_type.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let tenant_id = user.tenant_id.unwrap_or(0);
    let clean_filename = sanitize_filename(&body.original_filename);
    let unique_id = uuid::Uuid::new_v4().to_string();
    let object_key = format!("tenants/{}/avatars/{}-{}-{}", tenant_id, user_id, unique_id, clean_filename);

    let upload_url = state
        .storage
        .generate_presigned_upload_url(&object_key, &body.mime_type, 900)
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let cdn_url = Some(state.storage.get_cdn_url(&object_key));

    Ok(Json(AvatarPresignResponse {
        upload_url,
        object_key,
        cdn_url,
    }))
}
