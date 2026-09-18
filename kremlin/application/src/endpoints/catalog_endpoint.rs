use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::catalog_json::*;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, QueryFilter, QueryOrder,
    Set,
};
use entity::{category_entity, product_entity, sku_entity};

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

#[utoipa::path(get, path = "/categories", tag = "Catalog", responses((status = 200, body = [CategoryJson])))]
pub async fn categories(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CategoryJson>> {
    let mut query = category_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(category_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(category_entity::Column::Name)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CategoryJson {
                id: Some(x.id),
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                parent_id: x.parent_id,
                active: x.active,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/catalog-attributes", tag = "Catalog", responses((status = 200, body = [CatalogAttributeJson])))]
pub async fn attributes(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CatalogAttributeJson>> {
    let mut query = entity::catalog_attribute_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(entity::catalog_attribute_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CatalogAttributeJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                display_type: x.display_type,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/catalog-attribute-values", tag = "Catalog", responses((status = 200, body = [CatalogAttributeValueJson])))]
pub async fn attribute_values(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CatalogAttributeValueJson>> {
    let mut query = entity::catalog_attribute_value_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(entity::catalog_attribute_value_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CatalogAttributeValueJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                attribute_id: x.attribute_id,
                value: x.value,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/products", tag = "Catalog", responses((status = 200, body = [ProductJson])))]
pub async fn products(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<ProductJson>> {
    let mut query = product_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(product_entity::Column::Name)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| ProductJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                description: x.description,
                brand: x.brand,
                active: x.active,
                ncm: x.ncm,
                cest: x.cest,
                origem_mercadoria: x.origem_mercadoria,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/product-attributes", tag = "Catalog", responses((status = 200, body = [ProductAttributeJson])))]
pub async fn product_attributes(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<ProductAttributeJson>> {
    let mut query = entity::product_attribute_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(entity::product_attribute_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| ProductAttributeJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                product_id: x.product_id,
                attribute_id: x.attribute_id,
                required: x.required,
                sort_order: x.sort_order,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/skus", tag = "Catalog", responses((status = 200, body = [SkuJson])))]
pub async fn skus(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<SkuJson>> {
    let mut query = sku_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(sku_entity::Column::Code)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| SkuJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                product_id: x.product_id,
                code: x.code,
                variant_key: x.variant_key,
                price_cents: x.price_cents,
                compare_at_price_cents: x.compare_at_price_cents,
                weight_g: x.weight_g,
                width_mm: x.width_mm,
                height_mm: x.height_mm,
                length_mm: x.length_mm,
                active: x.active,
            })
            .collect(),
    )
}

#[utoipa::path(post, path = "/categories", tag = "Catalog", request_body = CategoryInputJson, responses((status = 201, body = CategoryJson)), security(("bearer_auth" = [])))]
pub async fn add_category(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<CategoryInputJson>,
) -> HttpResponse<(StatusCode, Json<CategoryJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.name.trim().is_empty() || input.slug.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = category_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        name: Set(input.name),
        slug: Set(input.slug),
        parent_id: Set(input.parent_id),
        active: Set(input.active),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };
    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok((
        StatusCode::CREATED,
        Json(CategoryJson {
            id: Some(saved.id),
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            name: saved.name,
            slug: saved.slug,
            parent_id: saved.parent_id,
            active: saved.active,
        }),
    ))
}

#[utoipa::path(put, path = "/categories/{id}", tag = "Catalog", params(("id" = i64, Path)), request_body = CategoryInputJson, responses((status = 200, body = CategoryJson)), security(("bearer_auth" = [])))]
pub async fn update_category(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<CategoryInputJson>,
) -> HttpResponse<Json<CategoryJson>> {
    let existing = category_entity::Entity::find_by_id(id)
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
    if input.name.trim().is_empty() || input.slug.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.name = Set(input.name);
    model.slug = Set(input.slug);
    model.parent_id = Set(input.parent_id);
    model.active = Set(input.active);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(CategoryJson {
        id: Some(saved.id),
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        name: saved.name,
        slug: saved.slug,
        parent_id: saved.parent_id,
        active: saved.active,
    }))
}

#[utoipa::path(post, path = "/products", tag = "Catalog", request_body = ProductInputJson, responses((status = 201, body = ProductJson)), security(("bearer_auth" = [])))]
pub async fn add_product(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<ProductInputJson>,
) -> HttpResponse<(StatusCode, Json<ProductJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.name.trim().is_empty() || input.slug.trim().is_empty() || input.ncm.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = product_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        name: Set(input.name),
        slug: Set(input.slug),
        description: Set(input.description),
        brand: Set(input.brand),
        active: Set(input.active),
        ncm: Set(input.ncm),
        cest: Set(input.cest),
        origem_mercadoria: Set(input.origem_mercadoria),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };
    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok((
        StatusCode::CREATED,
        Json(ProductJson {
            id: saved.id,
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            name: saved.name,
            slug: saved.slug,
            description: saved.description,
            brand: saved.brand,
            active: saved.active,
            ncm: saved.ncm,
            cest: saved.cest,
            origem_mercadoria: saved.origem_mercadoria,
        }),
    ))
}

#[utoipa::path(put, path = "/products/{id}", tag = "Catalog", params(("id" = i64, Path)), request_body = ProductInputJson, responses((status = 200, body = ProductJson)), security(("bearer_auth" = [])))]
pub async fn update_product(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<ProductInputJson>,
) -> HttpResponse<Json<ProductJson>> {
    let existing = product_entity::Entity::find_by_id(id)
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
    if input.name.trim().is_empty() || input.slug.trim().is_empty() || input.ncm.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.name = Set(input.name);
    model.slug = Set(input.slug);
    model.description = Set(input.description);
    model.brand = Set(input.brand);
    model.active = Set(input.active);
    model.ncm = Set(input.ncm);
    model.cest = Set(input.cest);
    model.origem_mercadoria = Set(input.origem_mercadoria);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(ProductJson {
        id: saved.id,
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        name: saved.name,
        slug: saved.slug,
        description: saved.description,
        brand: saved.brand,
        active: saved.active,
        ncm: saved.ncm,
        cest: saved.cest,
        origem_mercadoria: saved.origem_mercadoria,
    }))
}

#[utoipa::path(post, path = "/skus", tag = "Catalog", request_body = SkuInputJson, responses((status = 201, body = SkuJson)), security(("bearer_auth" = [])))]
pub async fn add_sku(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Json(input): Json<SkuInputJson>,
) -> HttpResponse<(StatusCode, Json<SkuJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;
    if input.code.trim().is_empty() || input.variant_key.trim().is_empty() || input.price_cents < 0
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let product = product_entity::Entity::find_by_id(input.product_id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.as_ref().map(|p| p.tenant_id) != Some(Some(tenant_id)) {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let model = sku_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        product_id: Set(input.product_id),
        code: Set(input.code),
        variant_key: Set(input.variant_key),
        price_cents: Set(input.price_cents),
        compare_at_price_cents: Set(input.compare_at_price_cents),
        weight_g: Set(input.weight_g),
        width_mm: Set(input.width_mm),
        height_mm: Set(input.height_mm),
        length_mm: Set(input.length_mm),
        active: Set(input.active),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };
    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok((
        StatusCode::CREATED,
        Json(SkuJson {
            id: saved.id,
            uuid: saved.uuid.to_string(),
            tenant_id: saved.tenant_id,
            product_id: saved.product_id,
            code: saved.code,
            variant_key: saved.variant_key,
            price_cents: saved.price_cents,
            compare_at_price_cents: saved.compare_at_price_cents,
            weight_g: saved.weight_g,
            width_mm: saved.width_mm,
            height_mm: saved.height_mm,
            length_mm: saved.length_mm,
            active: saved.active,
        }),
    ))
}

#[utoipa::path(put, path = "/skus/{id}", tag = "Catalog", params(("id" = i64, Path)), request_body = SkuInputJson, responses((status = 200, body = SkuJson)), security(("bearer_auth" = [])))]
pub async fn update_sku(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<SkuInputJson>,
) -> HttpResponse<Json<SkuJson>> {
    let existing = sku_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;
    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.price_cents < 0
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let product = product_entity::Entity::find_by_id(input.product_id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    if product.as_ref().map(|p| p.tenant_id) != Some(existing.tenant_id) {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let mut model = existing.into_active_model();
    model.product_id = Set(input.product_id);
    model.code = Set(input.code);
    model.variant_key = Set(input.variant_key);
    model.price_cents = Set(input.price_cents);
    model.compare_at_price_cents = Set(input.compare_at_price_cents);
    model.weight_g = Set(input.weight_g);
    model.width_mm = Set(input.width_mm);
    model.height_mm = Set(input.height_mm);
    model.length_mm = Set(input.length_mm);
    model.active = Set(input.active);
    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
    Ok(Json(SkuJson {
        id: saved.id,
        uuid: saved.uuid.to_string(),
        tenant_id: saved.tenant_id,
        product_id: saved.product_id,
        code: saved.code,
        variant_key: saved.variant_key,
        price_cents: saved.price_cents,
        compare_at_price_cents: saved.compare_at_price_cents,
        weight_g: saved.weight_g,
        width_mm: saved.width_mm,
        height_mm: saved.height_mm,
        length_mm: saved.length_mm,
        active: saved.active,
    }))
}
