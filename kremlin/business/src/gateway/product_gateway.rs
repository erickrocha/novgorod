use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product::{Product, ProductEntityMapper, ProductSearchQuery};
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

        if let Some(ref cat) = query.category {
            if cat != "all" {
                let escaped = cat.replace("'", "''");
                conditions.push(format!("EXISTS (SELECT 1 FROM product_category pc JOIN category c ON pc.category_id = c.id WHERE pc.product_id = p.id AND c.slug = '{}')", escaped));
            }
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
