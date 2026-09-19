use crate::AppState;
use crate::commons::pagination::{NormalizedPagination, PageQuery, PagedResponse};
use crate::endpoints::json::catalog_json::*;
use axum::{
    Json,
    extract::{Extension, Query, State},
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use entity::{catalog_attribute_entity, catalog_attribute_value_entity, product_attribute_entity};

#[utoipa::path(
    get,
    path = "/catalog-attributes",
    tag = "CatalogAttribute",
    responses(
        (status = 200, description = "List of catalog attributes", body = [CatalogAttributeJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn attributes(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CatalogAttributeJson>> {
    let mut query = catalog_attribute_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(catalog_attribute_entity::Column::TenantId.eq(id));
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

#[utoipa::path(
    get,
    path = "/catalog-attribute-values",
    tag = "CatalogAttribute",
    responses(
        (status = 200, description = "List of catalog attribute values", body = [CatalogAttributeValueJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn attribute_values(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CatalogAttributeValueJson>> {
    let mut query = catalog_attribute_value_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(catalog_attribute_value_entity::Column::TenantId.eq(id));
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

#[utoipa::path(
    get,
    path = "/product-attributes",
    tag = "CatalogAttribute",
    responses(
        (status = 200, description = "List of product attributes", body = [ProductAttributeJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn product_attributes(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<ProductAttributeJson>> {
    let mut query = product_attribute_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_attribute_entity::Column::TenantId.eq(id));
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
    tag = "CatalogAttribute",
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
    tag = "CatalogAttribute",
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
    tag = "CatalogAttribute",
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
