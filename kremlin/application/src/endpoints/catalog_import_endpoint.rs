use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use axum::{
    Json,
    extract::{Extension, Multipart, State},
};
use business::domain::{enums::Role, user::User};
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, QueryFilter, Set,
    TransactionTrait,
};
use entity::{
    catalog_attribute_entity, catalog_attribute_value_entity, category_entity,
    product_attribute_entity, product_category_entity, product_entity, sku_attribute_value_entity,
    sku_entity, sku_stock_entity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn deserialize_empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + std::str::FromStr,
    T::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper<T> {
        Val(T),
        Str(String),
        None,
    }

    match Option::<Helper<T>>::deserialize(deserializer)? {
        None | Some(Helper::None) => Ok(None),
        Some(Helper::Val(v)) => Ok(Some(v)),
        Some(Helper::Str(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<T>()
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}

fn deserialize_price_cents<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PriceHelper {
        Int(i32),
        Str(String),
    }

    match PriceHelper::deserialize(deserializer)? {
        PriceHelper::Int(i) => Ok(i),
        PriceHelper::Str(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(0)
            } else {
                trimmed.parse::<i32>().map_err(serde::de::Error::custom)
            }
        }
    }
}

fn deserialize_origem_lenient<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OrigemHelper {
        Int(i16),
        Str(String),
    }

    match Option::<OrigemHelper>::deserialize(deserializer)? {
        None => Ok(0),
        Some(OrigemHelper::Int(i)) => Ok(i),
        Some(OrigemHelper::Str(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(0)
            } else {
                trimmed.parse::<i16>().map_err(serde::de::Error::custom)
            }
        }
    }
}

fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolHelper {
        Bool(bool),
        Int(i64),
        Str(String),
    }

    match Option::<BoolHelper>::deserialize(deserializer)? {
        None => Ok(true),
        Some(BoolHelper::Bool(b)) => Ok(b),
        Some(BoolHelper::Int(i)) => Ok(i != 0),
        Some(BoolHelper::Str(s)) => {
            let trimmed = s.trim().to_lowercase();
            match trimmed.as_str() {
                "" | "true" | "1" | "t" | "yes" | "sim" => Ok(true),
                "false" | "0" | "f" | "no" | "nao" | "não" => Ok(false),
                _ => trimmed.parse::<bool>().map_err(serde::de::Error::custom),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct CatalogRow {
    #[serde(default)]
    product_key: String,
    #[serde(default)]
    category_key: String,
    #[serde(default)]
    category_name: Option<String>,
    #[serde(default)]
    product_name: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    brand: Option<String>,
    #[serde(default)]
    ncm: String,
    #[serde(default)]
    cest: Option<String>,
    #[serde(default, deserialize_with = "deserialize_origem_lenient")]
    origem_mercadoria: i16,
    #[serde(default)]
    code: String,
    #[serde(default)]
    variant_key: String,
    #[serde(default, deserialize_with = "deserialize_price_cents")]
    price_cents: i32,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    compare_at_price_cents: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    stock_quantity: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    weight_g: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    width_mm: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    height_mm: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_empty_as_none")]
    length_mm: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    active: bool,
    // Wine catalog attributes
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    grape: Option<String>,
    #[serde(default)]
    vintage: Option<String>,
    #[serde(default)]
    alcohol: Option<String>,
    #[serde(default)]
    volume: Option<String>,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogImportResult {
    pub total: usize,
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

pub async fn import(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> HttpResponse<Json<CatalogImportResult>> {
    let mut requested_tenant = None;
    let mut rows_json = None;
    let mut file_bytes = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "tenantId" {
            requested_tenant = field
                .text()
                .await
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .filter(|&id| id > 0);
        } else if name == "editedRows" {
            rows_json = field.text().await.ok();
        } else if name == "file" {
            file_bytes = field.bytes().await.ok();
        }
    }

    let tenant_id = match user.role {
        Role::SysAdmin => requested_tenant
            .or(user.tenant_id)
            .ok_or(ExceptionResponse::BadRequest(
                locale,
                ErrorKey::RequiredParameterMissing,
            ))?,
        Role::TenantOwner => user.tenant_id.ok_or(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?,
        Role::TenantUser => {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }
        Role::Customer => {
            return Err(ExceptionResponse::Forbidden(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }
    };
    let rows: Vec<CatalogRow> = if let Some(json) = rows_json {
        serde_json::from_str(&json)
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    } else if let Some(bytes) = file_bytes {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(bytes.as_ref());
        reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
    } else {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::RequiredParameterMissing,
        ));
    };
    if rows.is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }
    let tx = state
        .conn
        .begin()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let mut categories: HashMap<String, i64> = HashMap::new();
    let mut products: HashMap<String, i64> = HashMap::new();
    let mut attribute_cache: HashMap<String, i64> = HashMap::new();
    let mut prod_attr_cache: HashMap<(i64, i64), i64> = HashMap::new();
    let mut value_cache: HashMap<(i64, String), i64> = HashMap::new();

    let mut created = 0;
    let mut updated = 0;
    let unchanged = 0;

    for row in &rows {
        if row.product_name.trim().is_empty()
            || row.slug.trim().is_empty()
            || row.code.trim().is_empty()
            || row.price_cents < 0
        {
            return Err(ExceptionResponse::BadRequest(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }

        let category_slug = if row.category_key.trim().is_empty() {
            "geral".to_string()
        } else {
            row.category_key.trim().to_lowercase().replace(' ', "-")
        };

        let category_display_name = row
            .category_name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if !row.category_key.trim().is_empty() {
                    row.category_key.trim()
                } else {
                    "Geral"
                }
            });

        let category_id = if let Some(&id) = categories.get(&category_slug) {
            id
        } else {
            let found = category_entity::Entity::find()
                .filter(category_entity::Column::TenantId.eq(tenant_id))
                .filter(category_entity::Column::Slug.eq(&category_slug))
                .one(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?;
            let id = if let Some(value) = found {
                value.id
            } else {
                category_entity::ActiveModel {
                    id: NotSet,
                    uuid: NotSet,
                    tenant_id: Set(Some(tenant_id)),
                    name: Set(category_display_name.to_string()),
                    slug: Set(category_slug.clone()),
                    parent_id: Set(None),
                    active: Set(true),
                    created_at: NotSet,
                    created_by: NotSet,
                    updated_at: NotSet,
                    updated_by: NotSet,
                }
                .insert(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?
                .id
            };
            categories.insert(category_slug.clone(), id);
            categories.insert(row.category_key.clone(), id);
            id
        };

        let product = product_entity::Entity::find()
            .filter(product_entity::Column::TenantId.eq(tenant_id))
            .filter(product_entity::Column::Slug.eq(&row.slug))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        let product_id = if let Some(value) = product {
            let mut model = value.into_active_model();
            model.name = Set(row.product_name.clone());
            model.description = Set(row.description.clone());
            model.brand = Set(row.brand.clone());
            model.ncm = Set(row.ncm.clone());
            model.cest = Set(row.cest.clone());
            model.origem_mercadoria = Set(row.origem_mercadoria);
            model.active = Set(row.active);
            model
                .update(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?
                .id
        } else {
            product_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                name: Set(row.product_name.clone()),
                slug: Set(row.slug.clone()),
                description: Set(row.description.clone()),
                brand: Set(row.brand.clone()),
                active: Set(row.active),
                ncm: Set(row.ncm.clone()),
                cest: Set(row.cest.clone()),
                origem_mercadoria: Set(row.origem_mercadoria),
                created_at: NotSet,
                created_by: NotSet,
                updated_at: NotSet,
                updated_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?
            .id
        };
        products.insert(row.product_key.clone(), product_id);

        let link = product_category_entity::Entity::find()
            .filter(product_category_entity::Column::TenantId.eq(tenant_id))
            .filter(product_category_entity::Column::ProductId.eq(product_id))
            .filter(product_category_entity::Column::CategoryId.eq(category_id))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        if link.is_none() {
            product_category_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                product_id: Set(product_id),
                category_id: Set(category_id),
                is_primary: Set(true),
                created_at: NotSet,
                created_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        }

        let sku = sku_entity::Entity::find()
            .filter(sku_entity::Column::TenantId.eq(tenant_id))
            .filter(sku_entity::Column::Code.eq(&row.code))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

        let sku_id = if let Some(value) = sku {
            let mut model = value.into_active_model();
            model.product_id = Set(product_id);
            model.variant_key = Set(row.variant_key.clone());
            model.price_cents = Set(row.price_cents);
            model.compare_at_price_cents = Set(row.compare_at_price_cents);
            model.weight_g = Set(row.weight_g);
            model.width_mm = Set(row.width_mm);
            model.height_mm = Set(row.height_mm);
            model.length_mm = Set(row.length_mm);
            model.active = Set(row.active);
            let updated_model = model.update(&tx).await.map_err(|_| {
                ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
            })?;
            updated += 1;
            updated_model.id
        } else {
            let inserted = sku_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                product_id: Set(product_id),
                code: Set(row.code.clone()),
                variant_key: Set(row.variant_key.clone()),
                price_cents: Set(row.price_cents),
                compare_at_price_cents: Set(row.compare_at_price_cents),
                weight_g: Set(row.weight_g),
                width_mm: Set(row.width_mm),
                height_mm: Set(row.height_mm),
                length_mm: Set(row.length_mm),
                active: Set(row.active),
                created_at: NotSet,
                created_by: NotSet,
                updated_at: NotSet,
                updated_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
            created += 1;
            inserted.id
        };

        // Populate SKU Inventory Stock
        let existing_stock = sku_stock_entity::Entity::find()
            .filter(sku_stock_entity::Column::TenantId.eq(tenant_id))
            .filter(sku_stock_entity::Column::SkuId.eq(sku_id))
            .one(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        let stock_qty = row.stock_quantity.unwrap_or(36);
        if let Some(stock_model) = existing_stock {
            if row.stock_quantity.is_some() {
                let mut active_stock = stock_model.into_active_model();
                active_stock.quantity = Set(stock_qty);
                active_stock.update(&tx).await.map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?;
            }
        } else {
            sku_stock_entity::ActiveModel {
                id: NotSet,
                uuid: NotSet,
                tenant_id: Set(Some(tenant_id)),
                sku_id: Set(sku_id),
                quantity: Set(stock_qty),
                reserved: Set(0),
                created_at: NotSet,
                created_by: NotSet,
                updated_at: NotSet,
                updated_by: NotSet,
            }
            .insert(&tx)
            .await
            .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;
        }

        // Ingest Wine Attributes
        let mut attr_pairs: Vec<(&'static str, Option<&str>)> = Vec::new();
        if let Some(ref c) = row.country {
            attr_pairs.push(("País", Some(c.as_str())));
        }
        if let Some(ref r) = row.region {
            attr_pairs.push(("Região", Some(r.as_str())));
        }
        if let Some(ref g) = row.grape {
            attr_pairs.push(("Uva", Some(g.as_str())));
        }
        if let Some(ref v) = row.vintage {
            attr_pairs.push(("Safra", Some(v.as_str())));
        }
        if let Some(ref a) = row.alcohol {
            attr_pairs.push(("Teor Alcoólico", Some(a.as_str())));
        }
        let vol = row.volume.as_deref().or(if !row.variant_key.is_empty() {
            Some(row.variant_key.as_str())
        } else {
            None
        });
        if let Some(v) = vol {
            attr_pairs.push(("Volume", Some(v)));
        }
        if let Some(ref c) = row.color {
            attr_pairs.push(("Cor", Some(c.as_str())));
        }

        for (sort_order, (attr_name, attr_val_opt)) in attr_pairs.into_iter().enumerate() {
            let Some(val_raw) = attr_val_opt else { continue };
            let val = val_raw.trim();
            if val.is_empty() {
                continue;
            }

            // 1. Catalog Attribute
            let attr_id = if let Some(&id) = attribute_cache.get(attr_name) {
                id
            } else {
                let existing = catalog_attribute_entity::Entity::find()
                    .filter(catalog_attribute_entity::Column::TenantId.eq(tenant_id))
                    .filter(catalog_attribute_entity::Column::Name.eq(attr_name))
                    .one(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?;
                let id = if let Some(a) = existing {
                    a.id
                } else {
                    catalog_attribute_entity::ActiveModel {
                        id: NotSet,
                        uuid: NotSet,
                        tenant_id: Set(Some(tenant_id)),
                        name: Set(attr_name.to_string()),
                        display_type: Set("select".to_string()),
                        created_at: NotSet,
                        created_by: NotSet,
                        updated_at: NotSet,
                        updated_by: NotSet,
                    }
                    .insert(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?
                    .id
                };
                attribute_cache.insert(attr_name.to_string(), id);
                id
            };

            // 2. Product Attribute Link
            let prod_attr_key = (product_id, attr_id);
            let prod_attr_id = if let Some(&id) = prod_attr_cache.get(&prod_attr_key) {
                id
            } else {
                let existing = product_attribute_entity::Entity::find()
                    .filter(product_attribute_entity::Column::TenantId.eq(tenant_id))
                    .filter(product_attribute_entity::Column::ProductId.eq(product_id))
                    .filter(product_attribute_entity::Column::AttributeId.eq(attr_id))
                    .one(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?;
                let id = if let Some(pa) = existing {
                    pa.id
                } else {
                    product_attribute_entity::ActiveModel {
                        id: NotSet,
                        uuid: NotSet,
                        tenant_id: Set(Some(tenant_id)),
                        product_id: Set(product_id),
                        attribute_id: Set(attr_id),
                        required: Set(false),
                        sort_order: Set(sort_order as i32),
                        created_at: NotSet,
                        created_by: NotSet,
                    }
                    .insert(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?
                    .id
                };
                prod_attr_cache.insert(prod_attr_key, id);
                id
            };

            // 3. Catalog Attribute Value
            let val_key = (attr_id, val.to_string());
            let val_id = if let Some(&id) = value_cache.get(&val_key) {
                id
            } else {
                let existing = catalog_attribute_value_entity::Entity::find()
                    .filter(catalog_attribute_value_entity::Column::TenantId.eq(tenant_id))
                    .filter(catalog_attribute_value_entity::Column::AttributeId.eq(attr_id))
                    .filter(catalog_attribute_value_entity::Column::Value.eq(val))
                    .one(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?;
                let id = if let Some(av) = existing {
                    av.id
                } else {
                    catalog_attribute_value_entity::ActiveModel {
                        id: NotSet,
                        uuid: NotSet,
                        tenant_id: Set(Some(tenant_id)),
                        attribute_id: Set(attr_id),
                        value: Set(val.to_string()),
                        created_at: NotSet,
                        created_by: NotSet,
                        updated_at: NotSet,
                        updated_by: NotSet,
                    }
                    .insert(&tx)
                    .await
                    .map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?
                    .id
                };
                value_cache.insert(val_key, id);
                id
            };

            // 4. SKU Attribute Value Link
            let existing_sku_attr = sku_attribute_value_entity::Entity::find()
                .filter(sku_attribute_value_entity::Column::TenantId.eq(tenant_id))
                .filter(sku_attribute_value_entity::Column::SkuId.eq(sku_id))
                .filter(sku_attribute_value_entity::Column::AttributeId.eq(attr_id))
                .one(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?;
            if let Some(sku_attr) = existing_sku_attr {
                if sku_attr.attribute_value_id != val_id
                    || sku_attr.product_attribute_id != prod_attr_id
                {
                    let mut model = sku_attr.into_active_model();
                    model.attribute_value_id = Set(val_id);
                    model.product_attribute_id = Set(prod_attr_id);
                    model.update(&tx).await.map_err(|_| {
                        ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                    })?;
                }
            } else {
                sku_attribute_value_entity::ActiveModel {
                    id: NotSet,
                    uuid: NotSet,
                    tenant_id: Set(Some(tenant_id)),
                    product_id: Set(product_id),
                    sku_id: Set(sku_id),
                    product_attribute_id: Set(prod_attr_id),
                    attribute_id: Set(attr_id),
                    attribute_value_id: Set(val_id),
                    created_at: NotSet,
                    created_by: NotSet,
                    updated_at: NotSet,
                    updated_by: NotSet,
                }
                .insert(&tx)
                .await
                .map_err(|_| {
                    ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue)
                })?;
            }
        }
    }

    tx.commit()
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(CatalogImportResult {
        total: rows.len(),
        created,
        updated,
        unchanged,
        errors: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_wine_catalog_csv_deserialization_50_wines() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let csv_path = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("data/wines/wine_catalog.csv");

        let content = std::fs::read(&csv_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", csv_path.display(), e));

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(content.as_slice());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("CSV deserialization of 50 wines should succeed without error");

        assert_eq!(rows.len(), 50, "Catalog CSV must contain exactly 50 wines");

        let mut unique_slugs = HashSet::new();
        let mut unique_codes = HashSet::new();
        let mut categories = HashSet::new();
        let mut countries = HashSet::new();

        for row in &rows {
            assert!(!row.product_name.trim().is_empty(), "Product name must not be empty");
            assert!(!row.slug.trim().is_empty(), "Slug must not be empty");
            assert!(!row.code.trim().is_empty(), "Code must not be empty");
            assert!(row.price_cents > 0, "Price cents must be positive for {}", row.product_name);
            assert_eq!(row.ncm, "22042100", "NCM must be 22042100 for {}", row.product_name);
            assert_eq!(row.cest.as_deref(), Some("0301200"), "CEST must be 0301200 for {}", row.product_name);
            assert!(row.brand.is_some(), "Brand must be present for {}", row.product_name);
            assert!(row.country.is_some(), "Country must be present for {}", row.product_name);
            assert!(row.region.is_some(), "Region must be present for {}", row.product_name);
            assert!(row.grape.is_some(), "Grape must be present for {}", row.product_name);
            assert!(row.vintage.is_some(), "Vintage must be present for {}", row.product_name);
            assert!(row.alcohol.is_some(), "Alcohol must be present for {}", row.product_name);
            assert!(row.stock_quantity.unwrap_or(0) > 0, "Stock must be positive for {}", row.product_name);
            assert!(row.active, "Product must be active for {}", row.product_name);

            if row.country.as_deref() == Some("Brasil") {
                assert_eq!(row.origem_mercadoria, 0, "Brazilian wines must have origem 0: {}", row.product_name);
            } else {
                assert_eq!(row.origem_mercadoria, 1, "Imported wines must have origem 1: {}", row.product_name);
            }

            unique_slugs.insert(row.slug.clone());
            unique_codes.insert(row.code.clone());
            categories.insert(row.category_key.clone());
            countries.insert(row.country.clone().unwrap());
        }

        assert_eq!(unique_slugs.len(), 50, "All 50 wine slugs must be unique");
        assert_eq!(unique_codes.len(), 50, "All 50 SKU codes must be unique");
        assert!(categories.len() >= 4, "Must cover at least 4 wine categories, found: {:?}", categories);
        assert!(countries.contains("Brasil"));
        assert!(countries.contains("Argentina"));
        assert!(countries.contains("Chile"));
        assert!(countries.contains("França"));
        assert!(countries.contains("Itália"));
        assert!(countries.contains("Portugal"));
        assert!(countries.contains("Espanha"));
    }

    #[test]
    fn test_lenient_csv_deserialization_with_empty_fields() {
        let csv_data = "product_key,category_key,category_name,product_name,slug,description,brand,ncm,cest,origem_mercadoria,code,variant_key,price_cents,compare_at_price_cents,stock_quantity,weight_g,width_mm,height_mm,length_mm,active,country,region,grape,vintage,alcohol,volume,color\n\
        pk-1,vinhos-tintos,Vinhos Tintos,Vinho Teste,vinho-teste,,,22042100,,0,SKU-001,750ml,9900,,,,,,,true,Brasil,,,,,,Tinto\n";

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("Should deserialize leniently with empty fields");

        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.product_name, "Vinho Teste");
        assert_eq!(row.price_cents, 9900);
        assert_eq!(row.compare_at_price_cents, None);
        assert_eq!(row.stock_quantity, None);
        assert_eq!(row.weight_g, None);
        assert_eq!(row.description, None);
        assert_eq!(row.cest, None);
        assert!(row.active);
        assert_eq!(row.country.as_deref(), Some("Brasil"));
        assert_eq!(row.color.as_deref(), Some("Tinto"));
    }

    #[test]
    fn test_lenient_boolean_and_numeric_parsing() {
        let csv_data = "product_name,slug,code,price_cents,active,origem_mercadoria\n\
        W1,w-1,C-1,1000,1,0\n\
        W2,w-2,C-2,2000,0,1\n\
        W3,w-3,C-3,3000,true,0\n\
        W4,w-4,C-4,4000,false,1\n";

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("Should deserialize lenient booleans");

        assert_eq!(rows.len(), 4);
        assert!(rows[0].active);
        assert!(!rows[1].active);
        assert!(rows[2].active);
        assert!(!rows[3].active);
    }
}

