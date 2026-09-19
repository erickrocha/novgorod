use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::tax_rule_json::*;
use crate::infrastructure::mapper::{Mapper, TaxRuleMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::tax_rule::TaxRuleEntityMapper;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::tax_rule_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const TAX_RULE_SORT_FIELDS: &[&str] = &[
    "id",
    "ufOrigem",
    "uf_origem",
    "ufDestino",
    "uf_destino",
    "cfop",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/tax-rules",
    tag = "TaxRule",
    responses(
        (status = 200, description = "List of tax rules", body = [TaxRuleJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<TaxRuleJson>> {
    let mut query = tax_rule_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(tax_rule_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    let domains = TaxRuleEntityMapper::from_models(r);
    Json(TaxRuleMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/tax-rules/paged",
    tag = "TaxRule",
    params(TaxRulePageQuery),
    responses(
        (status = 200, description = "Paged tax rules", body = PagedResponse<TaxRuleJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<TaxRulePageQuery>,
) -> Json<PagedResponse<TaxRuleJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), TAX_RULE_SORT_FIELDS, "id");
    let mut query = tax_rule_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(tax_rule_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(tax_rule_entity::Column::TenantId.eq(tid));
    }

    if let Some(ref orig) = params.uf_origem {
        query = query.filter(tax_rule_entity::Column::UfOrigem.eq(orig));
    }

    if let Some(ref dest) = params.uf_destino {
        query = query.filter(tax_rule_entity::Column::UfDestino.eq(dest));
    }

    if let Some(active) = params.active {
        query = query.filter(tax_rule_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(tax_rule_entity::Column::UfOrigem.like(&pattern))
                .add(tax_rule_entity::Column::UfDestino.like(&pattern))
                .add(tax_rule_entity::Column::Cfop.like(&pattern))
                .add(tax_rule_entity::Column::Regime.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "uforigem" | "uf_origem" => tax_rule_entity::Column::UfOrigem,
        "ufdestino" | "uf_destino" => tax_rule_entity::Column::UfDestino,
        "cfop" => tax_rule_entity::Column::Cfop,
        "active" => tax_rule_entity::Column::Active,
        "createdat" | "created_at" => tax_rule_entity::Column::CreatedAt,
        _ => tax_rule_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(tax_rule_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(tax_rule_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = TaxRuleEntityMapper::from_models(rows);
    let items = TaxRuleMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/tax-rules/{id}",
    tag = "TaxRule",
    params(("id" = i64, Path, description = "Tax Rule ID")),
    responses(
        (status = 200, description = "Tax rule found", body = TaxRuleJson),
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
) -> HttpResponse<Json<TaxRuleJson>> {
    let item = tax_rule_entity::Entity::find_by_id(id)
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

    let domain = TaxRuleEntityMapper::from_model(item);
    Ok(Json(TaxRuleMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/tax-rules",
    tag = "TaxRule",
    request_body = TaxRuleInputJson,
    responses(
        (status = 201, description = "Tax rule created", body = TaxRuleJson),
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
    Json(input): Json<TaxRuleInputJson>,
) -> HttpResponse<(StatusCode, Json<TaxRuleJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.uf_origem.trim().is_empty()
        || input.uf_destino.trim().is_empty()
        || input.regime.trim().is_empty()
        || input.cfop.trim().is_empty()
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let model = tax_rule_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        uf_origem: Set(input.uf_origem.trim().to_uppercase()),
        uf_destino: Set(input.uf_destino.trim().to_uppercase()),
        ncm_prefix: Set(input.ncm_prefix),
        regime: Set(input.regime),
        csosn: Set(input.csosn),
        cfop: Set(input.cfop),
        icms_rate_bp: Set(input.icms_rate_bp),
        active: Set(input.active.unwrap_or(true)),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = TaxRuleEntityMapper::from_model(saved);
    Ok((StatusCode::CREATED, Json(TaxRuleMapper::json(domain))))
}

#[utoipa::path(
    put,
    path = "/tax-rules/{id}",
    tag = "TaxRule",
    params(("id" = i64, Path, description = "Tax Rule ID")),
    request_body = TaxRuleInputJson,
    responses(
        (status = 200, description = "Tax rule updated", body = TaxRuleJson),
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
    Json(input): Json<TaxRuleInputJson>,
) -> HttpResponse<Json<TaxRuleJson>> {
    let existing = tax_rule_entity::Entity::find_by_id(id)
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
    model.uf_origem = Set(input.uf_origem.trim().to_uppercase());
    model.uf_destino = Set(input.uf_destino.trim().to_uppercase());
    model.ncm_prefix = Set(input.ncm_prefix);
    model.regime = Set(input.regime);
    model.csosn = Set(input.csosn);
    model.cfop = Set(input.cfop);
    model.icms_rate_bp = Set(input.icms_rate_bp);
    if let Some(act) = input.active {
        model.active = Set(act);
    }

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = TaxRuleEntityMapper::from_model(saved);
    Ok(Json(TaxRuleMapper::json(domain)))
}
