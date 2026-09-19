use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::coupon_redemption::{CouponRedemption, CouponRedemptionEntityMapper};
use entity::coupon_redemption_entity;
use entity::prelude::CouponRedemptionEntity as CouponRedemptionQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct CouponRedemptionGateway {
    db: DbConn,
}

impl CouponRedemptionGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_coupon_id(
        &self,
        coupon_id: i64,
    ) -> Result<Vec<coupon_redemption_entity::Model>, DbErr> {
        tenant_select(
            CouponRedemptionQuery::find(),
            coupon_redemption_entity::Column::TenantId,
        )
        .filter(coupon_redemption_entity::Column::CouponId.eq(coupon_id))
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<CouponRedemption, coupon_redemption_entity::Model, coupon_redemption_entity::ActiveModel>
    for CouponRedemptionGateway
{
    async fn persist(
        &self,
        entity: CouponRedemption,
    ) -> Result<coupon_redemption_entity::ActiveModel, DbErr> {
        let active_model = CouponRedemptionEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CouponRedemptionQuery::delete_many()
                .filter(coupon_redemption_entity::Column::Id.eq(id)),
            coupon_redemption_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<coupon_redemption_entity::Model>, DbErr> {
        tenant_select(
            CouponRedemptionQuery::find(),
            coupon_redemption_entity::Column::TenantId,
        )
        .filter(coupon_redemption_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<coupon_redemption_entity::Model>, DbErr> {
        tenant_select(
            CouponRedemptionQuery::find(),
            coupon_redemption_entity::Column::TenantId,
        )
        .filter(coupon_redemption_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<coupon_redemption_entity::Model>, DbErr> {
        tenant_select(
            CouponRedemptionQuery::find(),
            coupon_redemption_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
