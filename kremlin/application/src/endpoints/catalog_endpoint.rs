use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PageQuery, PagedResponse};
use crate::endpoints::json::catalog_json::*;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::{
    catalog_attribute_entity, catalog_attribute_value_entity, category_entity,
    product_attribute_entity, product_entity, sku_entity,
};

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

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CategoryPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "parent_id")]
    pub parent_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CategoryPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const CATEGORY_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "slug",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/categories/paged",
    tag = "Catalog",
    params(CategoryPageQuery),
    responses(
        (status = 200, description = "Paged categories", body = PagedResponse<CategoryJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn categories_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CategoryPageQuery>,
) -> Json<PagedResponse<CategoryJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CATEGORY_SORT_FIELDS, "name");
    let mut query = category_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(category_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(category_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(category_entity::Column::Active.eq(active));
    }

    if let Some(parent_id) = params.parent_id {
        query = query.filter(category_entity::Column::ParentId.eq(parent_id));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(category_entity::Column::Name.like(&pattern))
                .add(category_entity::Column::Slug.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "slug" => category_entity::Column::Slug,
        "active" => category_entity::Column::Active,
        "createdat" | "created_at" => category_entity::Column::CreatedAt,
        "id" => category_entity::Column::Id,
        _ => category_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(category_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(category_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
        .map(|x| CategoryJson {
            id: Some(x.id),
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            name: x.name,
            slug: x.slug,
            parent_id: x.parent_id,
            active: x.active,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAttributePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CatalogAttributePageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const CATALOG_ATTRIBUTE_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "displayType",
    "display_type",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/catalog-attributes/paged",
    tag = "Catalog",
    params(CatalogAttributePageQuery),
    responses(
        (status = 200, description = "Paged catalog attributes", body = PagedResponse<CatalogAttributeJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn attributes_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CatalogAttributePageQuery>,
) -> Json<PagedResponse<CatalogAttributeJson>> {
    let norm = NormalizedPagination::new(
        &params.to_page_query(),
        CATALOG_ATTRIBUTE_SORT_FIELDS,
        "name",
    );
    let mut query = catalog_attribute_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(catalog_attribute_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(catalog_attribute_entity::Column::TenantId.eq(tid));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(catalog_attribute_entity::Column::Name.like(&pattern))
                .add(catalog_attribute_entity::Column::DisplayType.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "displaytype" | "display_type" => catalog_attribute_entity::Column::DisplayType,
        "createdat" | "created_at" => catalog_attribute_entity::Column::CreatedAt,
        "id" => catalog_attribute_entity::Column::Id,
        _ => catalog_attribute_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(catalog_attribute_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(catalog_attribute_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
        .map(|x| CatalogAttributeJson {
            id: x.id,
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            name: x.name,
            display_type: x.display_type,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAttributeValuePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "attribute_id")]
    pub attribute_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CatalogAttributeValuePageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const CATALOG_ATTRIBUTE_VALUE_SORT_FIELDS: &[&str] = &[
    "id",
    "value",
    "attributeId",
    "attribute_id",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/catalog-attribute-values/paged",
    tag = "Catalog",
    params(CatalogAttributeValuePageQuery),
    responses(
        (status = 200, description = "Paged catalog attribute values", body = PagedResponse<CatalogAttributeValueJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn attribute_values_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CatalogAttributeValuePageQuery>,
) -> Json<PagedResponse<CatalogAttributeValueJson>> {
    let norm = NormalizedPagination::new(
        &params.to_page_query(),
        CATALOG_ATTRIBUTE_VALUE_SORT_FIELDS,
        "id",
    );
    let mut query = catalog_attribute_value_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(catalog_attribute_value_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(catalog_attribute_value_entity::Column::TenantId.eq(tid));
    }

    if let Some(aid) = params.attribute_id {
        query = query.filter(catalog_attribute_value_entity::Column::AttributeId.eq(aid));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(catalog_attribute_value_entity::Column::Value.like(&pattern));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "value" => catalog_attribute_value_entity::Column::Value,
        "attributeid" | "attribute_id" => catalog_attribute_value_entity::Column::AttributeId,
        "createdat" | "created_at" => catalog_attribute_value_entity::Column::CreatedAt,
        _ => catalog_attribute_value_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(catalog_attribute_value_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(catalog_attribute_value_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
        .map(|x| CatalogAttributeValueJson {
            id: x.id,
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            attribute_id: x.attribute_id,
            value: x.value,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProductPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    pub brand: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl ProductPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const PRODUCT_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "slug",
    "brand",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/products/paged",
    tag = "Catalog",
    params(ProductPageQuery),
    responses(
        (status = 200, description = "Paged products", body = PagedResponse<ProductJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn products_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<ProductPageQuery>,
) -> Json<PagedResponse<ProductJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), PRODUCT_SORT_FIELDS, "name");
    let mut query = product_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(product_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(product_entity::Column::Active.eq(active));
    }

    if let Some(ref brand) = params.brand {
        if !brand.trim().is_empty() {
            query = query.filter(product_entity::Column::Brand.eq(brand.trim()));
        }
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(product_entity::Column::Name.like(&pattern))
                .add(product_entity::Column::Slug.like(&pattern))
                .add(product_entity::Column::Brand.like(&pattern))
                .add(product_entity::Column::Description.like(&pattern))
                .add(product_entity::Column::Ncm.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "slug" => product_entity::Column::Slug,
        "brand" => product_entity::Column::Brand,
        "active" => product_entity::Column::Active,
        "createdat" | "created_at" => product_entity::Column::CreatedAt,
        "id" => product_entity::Column::Id,
        _ => product_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(product_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(product_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
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
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProductAttributePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "product_id")]
    pub product_id: Option<i64>,
    #[serde(alias = "attribute_id")]
    pub attribute_id: Option<i64>,
    pub required: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl ProductAttributePageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const PRODUCT_ATTRIBUTE_SORT_FIELDS: &[&str] = &[
    "id",
    "productId",
    "product_id",
    "attributeId",
    "attribute_id",
    "required",
    "sortOrder",
    "sort_order",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/product-attributes/paged",
    tag = "Catalog",
    params(ProductAttributePageQuery),
    responses(
        (status = 200, description = "Paged product attributes", body = PagedResponse<ProductAttributeJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn product_attributes_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<ProductAttributePageQuery>,
) -> Json<PagedResponse<ProductAttributeJson>> {
    let norm = NormalizedPagination::new(
        &params.to_page_query(),
        PRODUCT_ATTRIBUTE_SORT_FIELDS,
        "sortOrder",
    );
    let mut query = product_attribute_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_attribute_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(product_attribute_entity::Column::TenantId.eq(tid));
    }

    if let Some(pid) = params.product_id {
        query = query.filter(product_attribute_entity::Column::ProductId.eq(pid));
    }

    if let Some(aid) = params.attribute_id {
        query = query.filter(product_attribute_entity::Column::AttributeId.eq(aid));
    }

    if let Some(req) = params.required {
        query = query.filter(product_attribute_entity::Column::Required.eq(req));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "productid" | "product_id" => product_attribute_entity::Column::ProductId,
        "attributeid" | "attribute_id" => product_attribute_entity::Column::AttributeId,
        "required" => product_attribute_entity::Column::Required,
        "createdat" | "created_at" => product_attribute_entity::Column::CreatedAt,
        "id" => product_attribute_entity::Column::Id,
        _ => product_attribute_entity::Column::SortOrder,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(product_attribute_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(product_attribute_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
        .map(|x| ProductAttributeJson {
            id: x.id,
            uuid: x.uuid.to_string(),
            tenant_id: x.tenant_id,
            product_id: x.product_id,
            attribute_id: x.attribute_id,
            required: x.required,
            sort_order: x.sort_order,
        })
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct SkuPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "product_id")]
    pub product_id: Option<i64>,
    pub active: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl SkuPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

const SKU_SORT_FIELDS: &[&str] = &[
    "id",
    "code",
    "variantKey",
    "variant_key",
    "priceCents",
    "price_cents",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/skus/paged",
    tag = "Catalog",
    params(SkuPageQuery),
    responses(
        (status = 200, description = "Paged SKUs", body = PagedResponse<SkuJson>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn skus_paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<SkuPageQuery>,
) -> Json<PagedResponse<SkuJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), SKU_SORT_FIELDS, "code");
    let mut query = sku_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(sku_entity::Column::TenantId.eq(tid));
    }

    if let Some(pid) = params.product_id {
        query = query.filter(sku_entity::Column::ProductId.eq(pid));
    }

    if let Some(active) = params.active {
        query = query.filter(sku_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(sku_entity::Column::Code.like(&pattern))
                .add(sku_entity::Column::VariantKey.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "variantkey" | "variant_key" => sku_entity::Column::VariantKey,
        "pricecents" | "price_cents" => sku_entity::Column::PriceCents,
        "active" => sku_entity::Column::Active,
        "createdat" | "created_at" => sku_entity::Column::CreatedAt,
        "id" => sku_entity::Column::Id,
        _ => sku_entity::Column::Code,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(sku_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(sku_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let items = rows
        .into_iter()
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
        .collect();

    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
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
