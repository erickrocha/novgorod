use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::city_json::CityJson;
use crate::endpoints::json::error_response_json::{
    ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson, UnauthorizedErrorJson,
};
use crate::commons::pagination::PagedResponse;
use crate::infrastructure::mapper::{CityMapper, Mapper};
use axum::Json;
use axum::extract::Extension;
use axum::extract::Multipart;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use business::domain::city::City;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::city_gateway::CityGateway;
use business::gateway::province_gateway::ProvinceGateway;
use business::use_cases::city_use_case::CityUseCase;
use business::use_cases::province_use_case::ProvinceUseCase;
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
    let list = use_case.find_all().await;
    Ok(Json(CityMapper::json_vec(list)))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CityPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "province_id")]
    pub province_id: Option<i64>,
}

impl CityPageQuery {
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

const CITY_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "provinceId",
    "province_id",
    "ibgeCode",
    "ibge_code",
];

#[utoipa::path(
    get,
    tag = "City",
    path = "/cities/paged",
    params(CityPageQuery),
    responses(
        (status = 200, description = "Paged cities", body = PagedResponse<CityJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    state: State<AppState>,
    Query(params): Query<CityPageQuery>,
) -> HttpResponse<Json<crate::commons::pagination::PagedResponse<CityJson>>> {
    use business::commons::entity_mapper::EntityMapper;
    use business::domain::city::CityEntityMapper;
    use business::sea_orm::{
        ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    };
    use entity::city_entity;

    let norm = crate::commons::pagination::NormalizedPagination::new(
        &params.to_page_query(),
        CITY_SORT_FIELDS,
        "name",
    );

    let mut query = city_entity::Entity::find();

    if let Some(pid) = params.province_id {
        query = query.filter(city_entity::Column::ProvinceId.eq(pid));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(city_entity::Column::Name.like(&pattern))
                .add(city_entity::Column::IbgeCode.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "provinceid" | "province_id" => city_entity::Column::ProvinceId,
        "ibgecode" | "ibge_code" => city_entity::Column::IbgeCode,
        "id" => city_entity::Column::Id,
        _ => city_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(city_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(city_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let models = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domain_cities = CityEntityMapper::from_models(models);
    let items = CityMapper::json_vec(domain_cities);

    Ok(Json(crate::commons::pagination::PagedResponse::new(
        items,
        total,
        norm.page,
        norm.page_size,
    )))
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
    let list = use_case.find_by_province_id(province_id).await;
    Ok(Json(CityMapper::json_vec(list)))
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
    let res = use_case.find_by_id(id).await.ok_or(ExceptionResponse::NotFound(
        locale,
        ErrorKey::RequiredParameterMissing,
    ))?;
    Ok(Json(CityMapper::json(res)))
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
    let saved = use_case
        .save(CityMapper::domain(payload))
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok((StatusCode::CREATED, Json(CityMapper::json(saved))))
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
    let saved = use_case
        .save(CityMapper::domain(payload))
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(CityMapper::json(saved)))
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
    let province_use_case = ProvinceUseCase::new(ProvinceGateway::new(state.conn.as_ref().clone()));
    let city_use_case = CityUseCase::new(CityGateway::new(state.conn.as_ref().clone()));
    let province_by_code: HashMap<String, business::domain::province::Province> = province_use_case
        .find_all()
        .await
        .into_iter()
        .filter_map(|value| value.ibge_code.clone().map(|code| (code, value)))
        .collect();
    let mut city_by_code: HashMap<String, City> = city_use_case
        .find_all()
        .await
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
            Some(value) => {
                city_by_code.insert(row.ibge_code.trim().to_string(), value);
                if is_update {
                    report.updated += 1;
                } else {
                    report.inserted += 1;
                }
            }
            None => {
                report.skipped += 1;
                report
                    .errors
                    .push(format!("row {}: database error", index + 2));
            }
        }
    }
    Ok(Json(report))
}
