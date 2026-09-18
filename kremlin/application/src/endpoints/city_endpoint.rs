use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::city_json::CityJson;
use crate::endpoints::json::error_response_json::{
    ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson, UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CityMapper, Mapper};
use axum::Json;
use axum::extract::Extension;
use axum::extract::Multipart;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use business::domain::city::City;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::city_gateway::CityGateway;
use business::use_cases::city_use_case::CityUseCase;
use serde::Serialize;
use std::collections::HashMap;

#[utoipa::path(
    get,
    tag = "City",
    path = "/city",
    responses(
        (status = 200, description = "List of all cities", body = Vec<CityJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(state: State<AppState>) -> HttpResponse<Json<Vec<CityJson>>> {
    let use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    match use_case.find_all().await {
        Ok(list) => Ok(Json(CityMapper::json_vec(list))),
        Err(_) => Ok(Json(Vec::new())),
    }
}

#[utoipa::path(
    get,
    tag = "City",
    path = "/city/by-province/{province_id}",
    params(
        ("province_id" = i64, Path, description = "Province ID")
    ),
    responses(
        (status = 200, description = "List of cities for the specified province", body = Vec<CityJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_province(
    state: State<AppState>,
    Path(province_id): Path<i32>,
) -> HttpResponse<Json<Vec<CityJson>>> {
    let use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_province_id(province_id).await {
        Ok(list) => Ok(Json(CityMapper::json_vec(list))),
        Err(_) => Ok(Json(Vec::new())),
    }
}

#[utoipa::path(
    get,
    tag = "City",
    path = "/city/{id}",
    params(
        ("id" = i64, Path, description = "City ID")
    ),
    responses(
        (status = 200, description = "City found", body = CityJson),
        (status = 404, description = "City not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<CityJson>> {
    let use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_id(id).await {
        Ok(res) => Ok(Json(CityMapper::json(res))),
        Err(_) => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::RequiredParameterMissing,
        )),
    }
}

#[utoipa::path(post, tag = "City", path = "/city", request_body = CityJson, responses((status = 201, body = CityJson)))]
pub async fn add(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(payload): Json<CityJson>,
) -> HttpResponse<(StatusCode, Json<CityJson>)> {
    if user.role != Role::SysAdmin {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    use_case
        .save(CityMapper::domain(payload))
        .await
        .map(|value| (StatusCode::CREATED, Json(CityMapper::json(value))))
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))
}

#[utoipa::path(put, tag = "City", path = "/city/{id}", params(("id" = i64, Path)), request_body = CityJson, responses((status = 200, body = CityJson)))]
pub async fn update(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(mut payload): Json<CityJson>,
) -> HttpResponse<Json<CityJson>> {
    if user.role != Role::SysAdmin {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    payload.id = Some(id);
    let use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    use_case
        .save(CityMapper::domain(payload))
        .await
        .map(|value| Json(CityMapper::json(value)))
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImportReport {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CityCsv {
    ibge_code: String,
    province_ibge_code: String,
    name: String,
}

#[utoipa::path(post, tag = "City", path = "/city/import", request_body(content_type = "multipart/form-data", content = String), responses((status = 200, body = ImportReport)))]
pub async fn import_csv(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> HttpResponse<Json<ImportReport>> {
    if user.role != Role::SysAdmin {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut bytes = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    {
        if field.name() == Some("file") {
            bytes = Some(field.bytes().await.map_err(|_| {
                ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
            })?);
        }
    }
    let Some(bytes) = bytes else {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::RequiredParameterMissing,
        ));
    };
    let mut reader = csv::Reader::from_reader(bytes.as_ref());
    let province_use_case = business::use_cases::province_use_case::ProvinceUseCase::new(
        business::gateway::province_gateway::ProvinceGateway::new(state.conn.as_ref().clone()),
    );
    let city_use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    let province_by_code: HashMap<String, business::domain::province::Province> = province_use_case
        .find_all()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.ibge_code.clone().map(|code| (code, value)))
        .collect();
    let mut city_by_code: HashMap<String, City> = city_use_case
        .find_all()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.ibge_code.clone().map(|code| (code, value)))
        .collect();
    let mut report = ImportReport {
        inserted: 0,
        updated: 0,
        skipped: 0,
        errors: Vec::new(),
    };
    for (index, row) in reader.deserialize::<CityCsv>().enumerate() {
        let row = match row {
            Ok(row) => row,
            Err(error) => {
                report.skipped += 1;
                report.errors.push(format!("row {}: {error}", index + 2));
                continue;
            }
        };
        let province = match province_by_code.get(row.province_ibge_code.trim()) {
            Some(value) => value,
            None => {
                report.skipped += 1;
                report
                    .errors
                    .push(format!("row {}: province not found", index + 2));
                continue;
            }
        };
        if row.ibge_code.trim().is_empty() || row.name.trim().is_empty() {
            report.skipped += 1;
            report
                .errors
                .push(format!("row {}: invalid required value", index + 2));
            continue;
        }
        let existing = city_by_code.get(row.ibge_code.trim()).cloned();
        let is_update = existing.is_some();
        let payload = CityJson {
            id: existing.as_ref().and_then(|v| v.id),
            uuid: existing.and_then(|v| v.uuid),
            province_id: province.id.unwrap_or_default(),
            name: row.name.trim().to_string(),
            ibge_code: Some(row.ibge_code.trim().to_string()),
        };
        match city_use_case.save(CityMapper::domain(payload)).await {
            Ok(value) => {
                city_by_code.insert(row.ibge_code.trim().to_string(), value);
                if is_update {
                    report.updated += 1;
                } else {
                    report.inserted += 1;
                }
            }
            Err(_) => {
                report.skipped += 1;
                report
                    .errors
                    .push(format!("row {}: database error", index + 2));
            }
        }
    }
    Ok(Json(report))
}
