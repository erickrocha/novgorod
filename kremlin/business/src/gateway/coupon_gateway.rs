use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::coupon::{Coupon, CouponEntityMapper};
use entity::coupon_entity;
use entity::prelude::CouponEntity as CouponQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CouponGateway {
    db: DbConn,
}

impl CouponGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_code(&self, code: String) -> Result<Option<coupon_entity::Model>, DbErr> {
        tenant_select(CouponQuery::find(), coupon_entity::Column::TenantId)
            .filter(coupon_entity::Column::Code.eq(code))
            .one(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<Coupon, coupon_entity::Model, coupon_entity::ActiveModel> for CouponGateway {
    async fn persist(&self, entity: Coupon) -> Result<coupon_entity::ActiveModel, DbErr> {
        let active_model = CouponEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CouponQuery::delete_many().filter(coupon_entity::Column::Id.eq(id)),
            coupon_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<coupon_entity::Model>, DbErr> {
        tenant_select(CouponQuery::find(), coupon_entity::Column::TenantId)
            .filter(coupon_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<coupon_entity::Model>, DbErr> {
        tenant_select(CouponQuery::find(), coupon_entity::Column::TenantId)
            .filter(coupon_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<coupon_entity::Model>, DbErr> {
        tenant_select(CouponQuery::find(), coupon_entity::Column::TenantId)
            .order_by_asc(coupon_entity::Column::Code)
            .all(&self.db)
            .await
    }
}
