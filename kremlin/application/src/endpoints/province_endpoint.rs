use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::commons::pagination::PagedResponse;
use crate::endpoints::json::province_json::ProvinceJson;
use crate::infrastructure::mapper::{Mapper, ProvinceMapper};
use axum::Json;
use axum::extract::Extension;
use axum::extract::Multipart;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::province_gateway::ProvinceGateway;
use business::use_cases::province_use_case::ProvinceUseCase;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProvinceQueryParams {
    /// ISO 3166-1 alpha-2 country code.
    #[serde(alias = "country_code")]
    pub country_code: String,
}

fn normalize_country_code(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_uppercase();
    (normalized.len() == 2 && normalized.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then_some(normalized)
}

#[utoipa::path(
    get,
    tag = "Province",
    path = "/province",
    params(ProvinceQueryParams),
    responses(
        (status = 200, description = "List of provinces", body = Vec<ProvinceJson>),
        (status = 400, description = "Missing or invalid country code", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Query(params): Query<ProvinceQueryParams>,
) -> HttpResponse<Json<Vec<ProvinceJson>>> {
    let country_code = normalize_country_code(&params.country_code).ok_or(
        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue),
    )?;

    let use_case = ProvinceUseCase::new(ProvinceGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_country_code(country_code).await {
        Ok(list) => Ok(Json(ProvinceMapper::json_vec(list))),
        Err(_) => Ok(Json(Vec::new())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProvincePageQuery {
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

impl ProvincePageQuery {
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

const PROVINCE_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "acronym",
    "countryCode",
    "country_code",
    "ibgeCode",
    "ibge_code",
];

#[utoipa::path(
    get,
    tag = "Province",
    path = "/province/paged",
    params(ProvincePageQuery),
    responses(
        (status = 200, description = "Paged provinces", body = PagedResponse<ProvinceJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    state: State<AppState>,
    Query(params): Query<ProvincePageQuery>,
) -> HttpResponse<Json<crate::commons::pagination::PagedResponse<ProvinceJson>>> {
    use business::commons::entity_mapper::EntityMapper;
    use business::domain::province::ProvinceEntityMapper;
    use business::sea_orm::{
        ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    };
    use entity::province_entity;

    let norm = crate::commons::pagination::NormalizedPagination::new(
        &params.to_page_query(),
        PROVINCE_SORT_FIELDS,
        "name",
    );

    let mut query = province_entity::Entity::find();

    if let Some(ref cc) = params.country_code {
        let trimmed = cc.trim().to_ascii_uppercase();
        if !trimmed.is_empty() {
            query = query.filter(province_entity::Column::CountryCode.eq(trimmed));
        }
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(province_entity::Column::Name.like(&pattern))
                .add(province_entity::Column::Acronym.like(&pattern))
                .add(province_entity::Column::IbgeCode.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "acronym" => province_entity::Column::Acronym,
        "countrycode" | "country_code" => province_entity::Column::CountryCode,
        "ibgecode" | "ibge_code" => province_entity::Column::IbgeCode,
        "id" => province_entity::Column::Id,
        _ => province_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(province_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(province_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let models = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domain_provinces = ProvinceEntityMapper::from_models(models);
    let items = ProvinceMapper::json_vec(domain_provinces);

    Ok(Json(crate::commons::pagination::PagedResponse::new(
        items,
        total,
        norm.page,
        norm.page_size,
    )))
}

#[utoipa::path(
    get,
    tag = "Province",
    path = "/province/{id}",
    params(
        ("id" = i32, Path, description = "Province ID")
    ),
    responses(
        (status = 200, description = "Province found", body = ProvinceJson),
        (status = 404, description = "Province not found", body = NotFoundErrorJson),
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
) -> HttpResponse<Json<ProvinceJson>> {
    let use_case = ProvinceUseCase::new(ProvinceGateway::new(state.conn.as_ref().clone()));
    match use_case.find_by_id(id).await {
        Ok(res) => Ok(Json(ProvinceMapper::json(res))),
        Err(_) => Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::RequiredParameterMissing,
        )),
    }
}

fn require_sysadmin(user: &User, locale: Locale) -> Result<(), ExceptionResponse> {
    (user.role == Role::SysAdmin)
        .then_some(())
        .ok_or(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ))
}

#[utoipa::path(post, tag = "Province", path = "/province", request_body = ProvinceJson, responses((status = 201, body = ProvinceJson)))]
pub async fn add(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(payload): Json<ProvinceJson>,
) -> HttpResponse<(StatusCode, Json<ProvinceJson>)> {
    require_sysadmin(&user, locale)?;
    let use_case = ProvinceUseCase::new(ProvinceGateway::new(state.conn.as_ref().clone()));
    use_case
        .save(ProvinceMapper::domain(payload))
        .await
        .map(|value| (StatusCode::CREATED, Json(ProvinceMapper::json(value))))
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))
}

#[utoipa::path(put, tag = "Province", path = "/province/{id}", params(("id" = i64, Path)), request_body = ProvinceJson, responses((status = 200, body = ProvinceJson)))]
pub async fn update(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(mut payload): Json<ProvinceJson>,
) -> HttpResponse<Json<ProvinceJson>> {
    require_sysadmin(&user, locale)?;
    payload.id = Some(id);
    let use_case = ProvinceUseCase::new(ProvinceGateway::new(state.conn.as_ref().clone()));
    use_case
        .save(ProvinceMapper::domain(payload))
        .await
        .map(|value| Json(ProvinceMapper::json(value)))
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
struct ProvinceCsv {
    ibge_code: String,
    acronym: String,
    name: String,
    country_code: String,
}

#[utoipa::path(post, tag = "Province", path = "/province/import", request_body(content_type = "multipart/form-data", content = String), responses((status = 200, body = ImportReport)))]
pub async fn import_csv(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> HttpResponse<Json<ImportReport>> {
    require_sysadmin(&user, locale)?;
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
    let gateway = ProvinceGateway::new(state.conn.as_ref().clone());
    let use_case = ProvinceUseCase::new(gateway);
    let mut report = ImportReport {
        inserted: 0,
        updated: 0,
        skipped: 0,
        errors: Vec::new(),
    };
    for (index, row) in reader.deserialize::<ProvinceCsv>().enumerate() {
        let row = match row {
            Ok(row) => row,
            Err(error) => {
                report.skipped += 1;
                report.errors.push(format!("row {}: {error}", index + 2));
                continue;
            }
        };
        let code = row.ibge_code.trim().to_string();
        if code.is_empty()
            || row.acronym.trim().is_empty()
            || row.name.trim().is_empty()
            || row.country_code.trim().len() != 2
        {
            report.skipped += 1;
            report
                .errors
                .push(format!("row {}: invalid required value", index + 2));
            continue;
        }
        let existing = use_case
            .find_all()
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
            .into_iter()
            .find(|value| value.ibge_code.as_deref() == Some(code.as_str()));
        let is_update = existing.is_some();
        let payload = ProvinceJson {
            id: existing.as_ref().and_then(|v| v.id),
            uuid: existing.and_then(|v| v.uuid),
            acronym: row.acronym.trim().to_ascii_uppercase(),
            name: row.name.trim().to_string(),
            country_code: row.country_code.trim().to_ascii_uppercase(),
            ibge_code: Some(code),
        };
        if use_case
            .save(ProvinceMapper::domain(payload))
            .await
            .is_err()
        {
            report.skipped += 1;
            report
                .errors
                .push(format!("row {}: database error", index + 2));
        } else if is_update {
            report.updated += 1;
        } else {
            report.inserted += 1;
        }
    }
    Ok(Json(report))
}

#[cfg(test)]
mod tests {
    use super::normalize_country_code;

    #[test]
    fn country_code_is_trimmed_and_uppercased() {
        assert_eq!(normalize_country_code(" br "), Some("BR".to_string()));
        assert_eq!(normalize_country_code("Us"), Some("US".to_string()));
    }

    #[test]
    fn country_code_rejects_missing_or_malformed_values() {
        for invalid in ["", "B", "BRA", "B1", "éR"] {
            assert_eq!(normalize_country_code(invalid), None);
        }
    }
}
