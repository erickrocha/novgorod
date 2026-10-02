use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::cart::{Cart, CartEntityMapper};
use entity::cart_entity;
use entity::prelude::CartEntity as CartQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, DbConn, DbErr,
    DeleteResult, EntityTrait, QueryFilter, Statement, TransactionTrait,
};
use std::collections::BTreeSet;

/// Longest wait for a row lock taken by a concurrent cart write while the purchased carts are
/// deleted (C-034, DES-CRT-03): an approved payment must not wait on a cart.
const CLEANUP_LOCK_TIMEOUT: &str = "2000ms";

pub struct CartGateway {
    db: DbConn,
}

impl CartGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    /// Deletes the carts a purchase consumed (C-034, SR-CRT-002, SR-CRT-003): with one order tenant
    /// the customer's carts of that tenant, with several (a marketplace purchase) the customer's
    /// carts without a tenant, none when there are no orders. Cart items go with the cart through
    /// the cascade foreign key. Explicit SQL: it must not depend on the request task-local scope,
    /// which a webhook does not have (SR-CRT-007). Returns the number of carts deleted.
    pub async fn delete_purchased<C: ConnectionTrait>(
        db: &C,
        customer_id: i64,
        tenants: &[i64],
    ) -> Result<u64, DbErr> {
        let distinct: BTreeSet<i64> = tenants.iter().copied().collect();
        let mut tenants = distinct.iter();
        let statement = match (tenants.next(), tenants.next()) {
            (None, _) => return Ok(0),
            (Some(tenant), None) => Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM cart WHERE customer_id = $1 AND tenant_id = $2",
                [customer_id.into(), (*tenant).into()],
            ),
            (Some(_), Some(_)) => Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM cart WHERE customer_id = $1 AND tenant_id IS NULL",
                [customer_id.into()],
            ),
        };
        Ok(db.execute_raw(statement).await?.rows_affected())
    }

    /// Best-effort cleanup inside the transaction that marks the purchase paid (SR-CRT-001,
    /// SR-CRT-004, SR-CRT-006): it runs in a savepoint and never fails the caller. A failure
    /// (including a lock timeout) rolls back to the savepoint, is logged and leaves the carts.
    pub async fn purge_purchased(
        tx: &DatabaseTransaction,
        purchase_id: i64,
        customer_id: i64,
        tenants: &[i64],
    ) {
        let savepoint = match tx.begin().await {
            Ok(savepoint) => savepoint,
            Err(e) => {
                log::warn!("cart cleanup skipped, purchase_id={purchase_id}: {e}");
                return;
            }
        };
        match Self::delete_with_bounded_wait(&savepoint, customer_id, tenants).await {
            Ok(deleted) => match savepoint.commit().await {
                Ok(()) => log::info!(
                    "cart cleanup done, purchase_id={purchase_id} carts_deleted={deleted}"
                ),
                Err(e) => log::warn!("cart cleanup not released, purchase_id={purchase_id}: {e}"),
            },
            Err(e) => {
                log::warn!("cart cleanup failed, purchase_id={purchase_id}: {e}");
                // ROLLBACK TO SAVEPOINT also undoes the lock_timeout set below
                if let Err(e) = savepoint.rollback().await {
                    log::error!("cart cleanup rollback failed, purchase_id={purchase_id}: {e}");
                }
            }
        }
    }

    async fn delete_with_bounded_wait(
        savepoint: &DatabaseTransaction,
        customer_id: i64,
        tenants: &[i64],
    ) -> Result<u64, DbErr> {
        let previous: String = savepoint
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                "SELECT current_setting('lock_timeout') AS value".to_string(),
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("lock_timeout not readable".into()))?
            .try_get("", "value")?;
        Self::set_lock_timeout(savepoint, CLEANUP_LOCK_TIMEOUT).await?;
        let deleted = Self::delete_purchased(savepoint, customer_id, tenants).await?;
        // SET LOCAL lasts until the end of the transaction and releasing the savepoint does not
        // undo it: put the previous value back so the rest of the transaction is not limited
        Self::set_lock_timeout(savepoint, &previous).await?;
        Ok(deleted)
    }

    async fn set_lock_timeout(db: &DatabaseTransaction, value: &str) -> Result<(), DbErr> {
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT set_config('lock_timeout', $1, true)",
            [value.into()],
        ))
        .await?;
        Ok(())
    }
}

#[async_trait]
impl Gateway<Cart, cart_entity::Model, cart_entity::ActiveModel> for CartGateway {
    async fn persist(&self, entity: Cart) -> Result<cart_entity::ActiveModel, DbErr> {
        let active_model = CartEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CartQuery::delete_many().filter(cart_entity::Column::Id.eq(id)),
            cart_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .filter(cart_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .filter(cart_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .all(&self.db)
            .await
    }
}
