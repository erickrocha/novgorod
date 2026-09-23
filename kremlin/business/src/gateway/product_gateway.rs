use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product::{
    Product, ProductEntityMapper, ProductSearchQuery, WebStoreAttributeDto,
    WebStoreImageDto, WebStoreProductDetailDto, WebStoreSellerDto, WebStoreSkuAttributeValueDto,
    WebStoreSkuDto,
};
use entity::prelude::ProductEntity as ProductQuery;
use entity::product_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder, Statement, DbBackend, ConnectionTrait
};

pub struct ProductGateway {
    db: DbConn,
}

impl ProductGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_paged_by_cursor(&self,query: ProductSearchQuery) -> Result<(Vec<product_entity::Model>, Option<i64>), DbErr> {
        let mut conditions = Condition::all();

        if let Some(active) = query.active {
            conditions = conditions.add(product_entity::Column::Active.eq(active));
        }

        if let Some(ref q) = query.q {
            let pattern = format!("%{}%", q);
            conditions = conditions.add(
                Condition::any()
                    .add(product_entity::Column::Name.like(&pattern))
                    .add(product_entity::Column::Slug.like(&pattern))
                    .add(product_entity::Column::Brand.like(&pattern))
                    .add(product_entity::Column::Description.like(&pattern))
                    .add(product_entity::Column::Ncm.like(&pattern)),
            );
        }

        let select = tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(conditions);

        let mut paginator = select.cursor_by(product_entity::Column::Id);
        if let Some(cursor) = query.cursor {
            paginator.after(cursor);
        }

        let limit = query.limit.unwrap_or(25);
        let mut rows = paginator.first(limit + 1).all(&self.db).await?;
        
        let next_cursor = if rows.len() as u64 > limit {
            rows.pop();
            rows.last().map(|r| r.id)
        } else {
            None
        };

        Ok((rows, next_cursor))
    }

    pub async fn find_webstore_products(&self, tenant_id: Option<i64>, query: ProductSearchQuery) -> Result<(Vec<crate::domain::product::WebStoreProductDto>, Option<i64>), DbErr> {
        // Build conditions
        let mut conditions = vec!["p.active = true".to_string()];
        
        if let Some(t_id) = tenant_id {
            conditions.push(format!("p.tenant_id = {}", t_id));
        }

        if let Some(ref q) = query.q {
            let escaped = q.replace("'", "''");
            conditions.push(format!("(p.name ILIKE '%{}%' OR p.description ILIKE '%{}%' OR p.brand ILIKE '%{}%')", escaped, escaped, escaped));
        }

        if let Some(ref cat) = query.category
            && cat != "all" {
                let escaped = cat.replace("'", "''");
                conditions.push(format!("EXISTS (SELECT 1 FROM product_category pc JOIN category c ON pc.category_id = c.id WHERE pc.product_id = p.id AND c.slug = '{}')", escaped));
            }

        // Basic cursor pagination using p.id
        // We will default to sorting by p.id DESC if not specified.
        // For custom sorts (price-asc), cursor pagination gets very complex with raw SQL unless we use offset.
        // For simplicity, we will use offset if sort_by is provided, or just id-based cursor if not.
        // Actually, we can just use offset always for the webstore if cursor is used as an offset.
        // Wait, if cursor is an ID, it's hard to sort by price.
        // Let's assume cursor is an offset for webstore if we need sorting.
        let offset = query.cursor.unwrap_or(0);
        let limit = query.limit.unwrap_or(12);

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let mut order_clause = "ORDER BY p.id DESC".to_string();
        if let Some(ref sort) = query.sort_by {
            if sort == "price-asc" {
                order_clause = "ORDER BY min_price ASC NULLS LAST, p.id DESC".to_string();
            } else if sort == "price-desc" {
                order_clause = "ORDER BY min_price DESC NULLS LAST, p.id DESC".to_string();
            } else if sort == "rating" {
                order_clause = "ORDER BY p.id DESC".to_string(); // Mock rating sort
            } else if sort == "newest" {
                order_clause = "ORDER BY p.created_at DESC NULLS LAST, p.id DESC".to_string();
            }
        }

        let raw_sql = format!(
            r#"
            SELECT 
                p.id, p.uuid, p.name, p.slug, p.description, p.brand,
                (SELECT MIN(s.price_cents) FROM sku s WHERE s.product_id = p.id AND s.active = true) as min_price,
                (SELECT MAX(s.compare_at_price_cents) FROM sku s WHERE s.product_id = p.id AND s.active = true) as compare_price,
                (SELECT pi.object_key FROM product_image pi WHERE pi.product_id = p.id AND pi.is_primary = true LIMIT 1) as primary_image_url,
                (SELECT pi.alt_text FROM product_image pi WHERE pi.product_id = p.id AND pi.is_primary = true LIMIT 1) as primary_image_alt
            FROM product p
            {}
            {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, order_clause, limit + 1, offset
        );

        let query_res = self.db.query_all_raw(Statement::from_string(DbBackend::Postgres, raw_sql)).await?;
        
        let mut products = Vec::new();
        for row in query_res {
            let id: i64 = row.try_get("", "id")?;
            let uuid: uuid::Uuid = row.try_get("", "uuid")?;
            let uuid_str = crate::commons::functions::uuid_to_string(uuid);
            let name: String = row.try_get("", "name")?;
            let slug: String = row.try_get("", "slug")?;
            let description: Option<String> = row.try_get("", "description")?;
            let brand: Option<String> = row.try_get("", "brand")?;
            let price_cents: Option<i32> = row.try_get("", "min_price")?;
            let compare_at_price_cents: Option<i32> = row.try_get("", "compare_price")?;
            let primary_image_url: Option<String> = row.try_get("", "primary_image_url")?;
            let primary_image_alt: Option<String> = row.try_get("", "primary_image_alt")?;

            products.push(crate::domain::product::WebStoreProductDto {
                id,
                uuid: uuid_str,
                name,
                slug,
                description,
                brand,
                price_cents,
                compare_at_price_cents,
                primary_image_url,
                primary_image_alt,
                category_slugs: vec![], // we can fetch these if needed
                rating: Some(4.5),
                review_count: Some(12),
                is_featured: false,
                is_new: false,
            });
        }

        let next_cursor = if products.len() as u64 > limit {
            products.pop();
            Some(offset + limit as i64)
        } else {
            None
        };

        Ok((products, next_cursor))
    }

    pub async fn find_webstore_product_detail(&self, slug_or_id: &str) -> Result<Option<WebStoreProductDetailDto>, DbErr> {
        let product_res = if let Ok(id) = slug_or_id.parse::<i64>() {
            product_entity::Entity::find()
                .filter(product_entity::Column::Id.eq(id))
                .filter(product_entity::Column::Active.eq(true))
                .one(&self.db)
                .await?
        } else {
            product_entity::Entity::find()
                .filter(product_entity::Column::Slug.eq(slug_or_id))
                .filter(product_entity::Column::Active.eq(true))
                .one(&self.db)
                .await?
        };

        let product = match product_res {
            Some(p) => p,
            None => return Ok(None),
        };
        let product_id = product.id;

        // Fetch seller / tenant
        let seller = if let Some(tenant_id) = product.tenant_id {
            let tenant_res = entity::tenant_entity::Entity::find_by_id(tenant_id).one(&self.db).await?;
            if let Some(t) = tenant_res {
                WebStoreSellerDto {
                    id: t.id,
                    business_name: t.business_name,
                    company_name: t.company_name,
                    email: t.email,
                    phone: t.phone,
                    web_site: t.web_site,
                    locality: t.locality,
                    administrative_area: t.administrative_area,
                    postal_code: t.postal_code,
                    country_code: t.country_code,
                }
            } else {
                WebStoreSellerDto {
                    id: tenant_id,
                    business_name: "Novgorod Vinhos".to_string(),
                    company_name: None,
                    email: None,
                    phone: None,
                    web_site: None,
                    locality: None,
                    administrative_area: None,
                    postal_code: None,
                    country_code: None,
                }
            }
        } else {
            WebStoreSellerDto {
                id: 0,
                business_name: "Novgorod Vinhos".to_string(),
                company_name: None,
                email: None,
                phone: None,
                web_site: None,
                locality: None,
                administrative_area: None,
                postal_code: None,
                country_code: None,
            }
        };

        // Fetch all product images
        let images_models = entity::product_image_entity::Entity::find()
            .filter(entity::product_image_entity::Column::ProductId.eq(product_id))
            .filter(entity::product_image_entity::Column::StorageStatus.eq("available"))
            .order_by_asc(entity::product_image_entity::Column::SortOrder)
            .order_by_asc(entity::product_image_entity::Column::Id)
            .all(&self.db)
            .await?;

        let images = images_models.into_iter().map(|img| WebStoreImageDto {
            id: img.id,
            object_key: img.object_key,
            alt_text: img.alt_text,
            sort_order: img.sort_order,
            is_primary: img.is_primary,
            width_px: img.width_px,
            height_px: img.height_px,
        }).collect();

        // Fetch SKUs
        let skus_models = entity::sku_entity::Entity::find()
            .filter(entity::sku_entity::Column::ProductId.eq(product_id))
            .filter(entity::sku_entity::Column::Active.eq(true))
            .order_by_asc(entity::sku_entity::Column::Id)
            .all(&self.db)
            .await?;

        // Query SKU attribute values for this product
        let raw_sku_attrs_sql = format!(
            r#"
            SELECT 
                sav.sku_id, sav.attribute_id, ca.name as attr_name, cav.value as attr_val
            FROM sku_attribute_value sav
            JOIN catalog_attribute ca ON ca.id = sav.attribute_id
            JOIN catalog_attribute_value cav ON cav.id = sav.attribute_value_id
            WHERE sav.product_id = {}
            ORDER BY ca.id ASC
            "#,
            product_id
        );
        let sku_attr_rows = self.db.query_all_raw(Statement::from_string(DbBackend::Postgres, raw_sku_attrs_sql)).await?;
        let mut sku_attrs_map: std::collections::HashMap<i64, Vec<WebStoreSkuAttributeValueDto>> = std::collections::HashMap::new();
        for row in sku_attr_rows {
            let s_id: i64 = row.try_get("", "sku_id")?;
            let a_id: i64 = row.try_get("", "attribute_id")?;
            let a_name: String = row.try_get("", "attr_name")?;
            let a_val: String = row.try_get("", "attr_val")?;
            sku_attrs_map.entry(s_id).or_default().push(WebStoreSkuAttributeValueDto {
                attribute_id: a_id,
                name: a_name,
                value: a_val,
            });
        }

        // Query stock for skus
        let raw_stock_sql = format!(
            r#"
            SELECT sku_id, COALESCE(SUM(quantity), 0) as total_stock
            FROM sku_stock
            WHERE sku_id IN (SELECT id FROM sku WHERE product_id = {})
            GROUP BY sku_id
            "#,
            product_id
        );
        let stock_rows = self.db.query_all_raw(Statement::from_string(DbBackend::Postgres, raw_stock_sql)).await?;
        let mut stock_map: std::collections::HashMap<i64, i32> = std::collections::HashMap::new();
        for row in stock_rows {
            let s_id: i64 = row.try_get("", "sku_id")?;
            let s_qty: i64 = row.try_get("", "total_stock")?;
            stock_map.insert(s_id, s_qty as i32);
        }

        let skus = skus_models.into_iter().map(|s| {
            let stock = stock_map.get(&s.id).copied().unwrap_or(0);
            let attributes = sku_attrs_map.remove(&s.id).unwrap_or_default();
            WebStoreSkuDto {
                id: s.id,
                uuid: crate::commons::functions::uuid_to_string(s.uuid),
                code: s.code,
                variant_key: s.variant_key,
                price_cents: s.price_cents,
                compare_at_price_cents: s.compare_at_price_cents,
                weight_g: s.weight_g,
                width_mm: s.width_mm,
                height_mm: s.height_mm,
                length_mm: s.length_mm,
                active: s.active,
                stock,
                attributes,
            }
        }).collect();

        // Fetch Product attributes
        let raw_prod_attrs_sql = format!(
            r#"
            SELECT pa.id, pa.attribute_id, ca.name, ca.display_type, pa.required, pa.sort_order
            FROM product_attribute pa
            JOIN catalog_attribute ca ON ca.id = pa.attribute_id
            WHERE pa.product_id = {}
            ORDER BY pa.sort_order ASC, pa.id ASC
            "#,
            product_id
        );
        let prod_attr_rows = self.db.query_all_raw(Statement::from_string(DbBackend::Postgres, raw_prod_attrs_sql)).await?;
        let mut attributes = Vec::new();
        for r in prod_attr_rows {
            let id: i64 = r.try_get("", "id")?;
            let attribute_id: i64 = r.try_get("", "attribute_id")?;
            let name: String = r.try_get("", "name")?;
            let display_type: String = r.try_get("", "display_type")?;
            let required: bool = r.try_get("", "required")?;
            let sort_order: i32 = r.try_get("", "sort_order")?;
            attributes.push(WebStoreAttributeDto {
                id,
                attribute_id,
                name,
                display_type,
                required,
                sort_order,
            });
        }

        // Fetch Category slugs
        let raw_cat_sql = format!(
            r#"
            SELECT c.slug
            FROM category c
            JOIN product_category pc ON pc.category_id = c.id
            WHERE pc.product_id = {}
            "#,
            product_id
        );
        let cat_rows = self.db.query_all_raw(Statement::from_string(DbBackend::Postgres, raw_cat_sql)).await?;
        let category_slugs = cat_rows.into_iter().filter_map(|r| r.try_get("", "slug").ok()).collect();

        Ok(Some(WebStoreProductDetailDto {
            id: product.id,
            uuid: crate::commons::functions::uuid_to_string(product.uuid),
            name: product.name,
            slug: product.slug,
            description: product.description,
            brand: product.brand,
            active: product.active,
            ncm: product.ncm,
            cest: product.cest,
            origem_mercadoria: product.origem_mercadoria,
            seller,
            images,
            skus,
            attributes,
            category_slugs,
            rating: Some(4.8),
            review_count: Some(24),
        }))
    }
}

#[async_trait]
impl Gateway<Product, product_entity::Model, product_entity::ActiveModel> for ProductGateway {
    async fn persist(&self, entity: Product) -> Result<product_entity::ActiveModel, DbErr> {
        let active_model = ProductEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ProductQuery::delete_many().filter(product_entity::Column::Id.eq(id)),
            product_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .order_by_asc(product_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
