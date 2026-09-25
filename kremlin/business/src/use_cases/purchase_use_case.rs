use crate::domain::{enums::Role, marketplace::*, user::User};
use crate::use_cases::checkout_quote_use_case::{CheckoutQuoteUseCase, QuoteResult};
use crate::gateway::{
    order_address_gateway::OrderAddressGateway, order_item_gateway::OrderItemGateway,
    order_status_history_gateway::OrderStatusHistoryGateway, orders_gateway::OrdersGateway,
    payment_allocation_gateway::PaymentAllocationGateway, payment_gateway::PaymentGateway,
    purchase_gateway::PurchaseGateway,
};
use entity::{
    order_address_entity, order_item_entity, order_status_history_entity, orders_entity,
    payment_allocation_entity, payment_entity, purchase_entity, coupon_redemption_entity,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Set,
    Statement, TransactionTrait,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub struct PurchaseUseCase {
    gateway: PurchaseGateway,
}
pub type CreatePurchaseUseCase = PurchaseUseCase;

pub struct CheckoutPurchaseInput {
    pub quote_id: i64,
    pub email: String,
    pub phone: String,
}

impl PurchaseUseCase {
    pub fn new(gateway: PurchaseGateway) -> Self {
        Self { gateway }
    }

    pub async fn access(&self, user: &User) -> Result<PurchaseAccess, PurchaseError> {
        match user.role {
            Role::SysAdmin => Ok(PurchaseAccess::Admin),
            Role::TenantOwner | Role::TenantUser => user
                .tenant_id
                .map(PurchaseAccess::Seller)
                .ok_or(PurchaseError::Forbidden),
            Role::Customer => {
                let user_id = user.id.ok_or(PurchaseError::Forbidden)?;
                let customer =
                    PurchaseGateway::customer_for_user(&self.gateway.db, user_id).await?;
                Ok(PurchaseAccess::Customer {
                    customer_id: customer.id,
                    user_id,
                })
            }
        }
    }

    pub async fn create(
        &self,
        user: &User,
        key: &str,
        input: CreatePurchaseInput,
    ) -> Result<CreatedPurchase, PurchaseError> {
        self.create_internal(user, key, input, None).await
    }

    pub async fn create_checkout(&self, user: &User, key: &str, checkout: CheckoutPurchaseInput) -> Result<CreatedPurchase, PurchaseError> {
        let user_id = user.id.ok_or(PurchaseError::Forbidden)?;
        let customer = PurchaseGateway::customer_for_user(&self.gateway.db, user_id).await?;
        let (request, quote) = CheckoutQuoteUseCase::load_any(&self.gateway.db, customer.id, checkout.quote_id).await?;
        let input = CreatePurchaseInput { items: request.items, shipping_address: quote.shipping_address, billing_address: None };
        self.create_internal(user, key, input, Some(checkout)).await
    }

    async fn create_internal(&self, user: &User, key: &str, input: CreatePurchaseInput,
        checkout: Option<CheckoutPurchaseInput>) -> Result<CreatedPurchase, PurchaseError> {
        if user.role != Role::Customer {
            return Err(PurchaseError::Forbidden);
        }
        if key.is_empty() || key.len() > 128 || !key.bytes().all(|c| c.is_ascii_graphic()) {
            return Err(PurchaseError::Validation("invalid idempotency key"));
        }
        let user_id = user.id.ok_or(PurchaseError::Forbidden)?;
        let input = input.normalize()?;
        let hash = if let Some(ref checkout) = checkout {
            format!("{:x}", Sha256::digest(format!("{}:{}:{}:{}", input.request_hash(), checkout.quote_id, checkout.email, checkout.phone)))
        } else { input.request_hash() };
        let tx = self.gateway.db.begin().await?;
        let customer = PurchaseGateway::customer_for_user(&tx, user_id).await?;
        let access = PurchaseAccess::Customer {
            customer_id: customer.id,
            user_id,
        };

        // Serialize retries before checking for an existing row. The unique
        // constraint remains the final guard, and the lock is transaction scoped.
        let lock_hash = Sha256::digest(format!("purchase:{}:{key}", customer.id).as_bytes());
        let lock_id = i64::from_be_bytes(lock_hash[..8].try_into().expect("eight bytes"));
        tx.query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT pg_advisory_xact_lock($1)",
            [lock_id.into()],
        ))
        .await?;
        if let Some(existing) = purchase_entity::Entity::find()
            .filter(purchase_entity::Column::CustomerId.eq(customer.id))
            .filter(purchase_entity::Column::IdempotencyKey.eq(key))
            .one(&tx)
            .await?
        {
            if existing.request_hash != hash {
                return Err(PurchaseError::Conflict);
            }
            let detail = PurchaseGateway::detail(&tx, access, existing.id).await?;
            tx.commit().await?;
            return Ok(CreatedPurchase {
                detail,
                replayed: true,
            });
        }

        let quote: Option<QuoteResult> = if let Some(ref checkout) = checkout {
            let (request, stored) = CheckoutQuoteUseCase::load(&tx, customer.id, checkout.quote_id).await?;
            if checkout.email.trim().is_empty() || checkout.email.len() > 500 || !checkout.email.contains('@')
                || checkout.phone.trim().len() < 8 || checkout.phone.len() > 20 {
                return Err(PurchaseError::Validation("invalid order contact"));
            }
            // Serialize all reservations for each coupon before rechecking limits.
            for seller in &stored.sellers {
                if let Some(coupon_id) = seller.coupon_id {
                    tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
                        "SELECT pg_advisory_xact_lock($1)", [coupon_id.into()])).await?;
                }
            }
            let current = CheckoutQuoteUseCase::calculate(&tx, customer.id, &request).await?;
            if current.items != stored.items || current.sellers != stored.sellers || current.shipping_address != stored.shipping_address
                || current.total_cents != stored.total_cents { return Err(PurchaseError::Conflict); }
            Some(stored)
        } else { None };

        let tax_id = customer
            .cpf
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(PurchaseError::Validation("customer tax ID required"))?;
        if customer.name.trim().is_empty()
            || customer.name.chars().count() > 500
            || tax_id.chars().count() > 50
            || customer.email.trim().is_empty()
            || customer.email.chars().count() > 500
            || customer
                .phone
                .as_ref()
                .is_some_and(|s| s.chars().count() > 20)
        {
            return Err(PurchaseError::Validation("invalid customer details"));
        }
        let mut groups = BTreeMap::<i64, Vec<order_item_entity::ActiveModel>>::new();
        let mut subtotal = 0_i64;
        for item in &input.items {
            let (sku, product, attributes) = PurchaseGateway::catalog(&tx, item.sku_id).await?;
            let tenant_id = sku
                .tenant_id
                .ok_or(PurchaseError::Validation("seller required"))?;
            let line_total = i64::from(sku.price_cents)
                .checked_mul(i64::from(item.quantity))
                .ok_or(PurchaseError::Validation("amount overflow"))?;
            subtotal = subtotal
                .checked_add(line_total)
                .ok_or(PurchaseError::Validation("amount overflow"))?;
            groups
                .entry(tenant_id)
                .or_default()
                .push(order_item_entity::ActiveModel {
                    tenant_id: Set(tenant_id),
                    sku_id: Set(sku.id),
                    sku_code: Set(sku.code),
                    product_name: Set(product.name),
                    attributes_desc: Set(attributes),
                    quantity: Set(item.quantity),
                    unit_price_cents: Set(i64::from(sku.price_cents)),
                    discount_cents: Set(0),
                    ncm: Set(product.ncm),
                    cfop: Set(None),
                    csosn: Set(None),
                    icms_rate_bp: Set(0),
                    tax_cents: Set(0),
                    total_cents: Set(line_total),
                    ..Default::default()
                });
        }
        let discount = quote.as_ref().map_or(0, |q| q.discount_cents);
        let shipping = quote.as_ref().map_or(0, |q| q.shipping_cents);
        let total = quote.as_ref().map_or(subtotal, |q| q.total_cents);
        if quote.as_ref().is_some_and(|q| q.subtotal_cents != subtotal) { return Err(PurchaseError::Conflict); }
        let paid = checkout.is_some() && total == 0;
        let order_email = checkout.as_ref().map(|c| c.email.trim().to_string()).or(Some(customer.email.clone()));
        let order_phone = checkout.as_ref().map(|c| c.phone.trim().to_string()).or(customer.phone.clone());
        let purchase = purchase_entity::ActiveModel {
            customer_id: Set(customer.id),
            customer_name: Set(customer.name.clone()),
            customer_tax_id: Set(tax_id.to_owned()),
            customer_email: Set(order_email.clone()),
            customer_phone: Set(order_phone.clone()),
            currency: Set("BRL".into()),
            status: Set(if paid { "paid" } else { "pending_payment" }.into()),
            subtotal_cents: Set(subtotal),
            discount_cents: Set(discount),
            shipping_cents: Set(shipping),
            tax_total_cents: Set(0),
            total_cents: Set(total),
            idempotency_key: Set(key.to_owned()),
            request_hash: Set(hash),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
        let payment = PaymentGateway::insert(
            &tx,
            payment_entity::ActiveModel {
                purchase_id: Set(purchase.id),
                method: Set("credit_card".into()),
                status: Set(if paid { "captured" } else { "pending_provider" }.into()),
                installments: Set(1),
                amount_cents: Set(total),
                currency: Set("BRL".into()),
                gateway_provider: Set(None),
                gateway_reference: Set(None),
                ..Default::default()
            },
        )
        .await?;
        let mut allocated = 0_i64;
        for (tenant_id, items) in groups {
            let seller_subtotal = items.iter().try_fold(0_i64, |sum, item| {
                sum.checked_add(*item.total_cents.as_ref())
                    .ok_or(PurchaseError::Validation("amount overflow"))
            })?;
            let pricing = quote.as_ref().and_then(|q| q.sellers.iter().find(|s| s.tenant_id == tenant_id));
            let seller_discount = pricing.map_or(0, |s| s.discount_cents);
            let seller_shipping = pricing.map_or(0, |s| s.shipping_cents);
            let total = seller_subtotal - seller_discount + seller_shipping;
            let row = tx
                .query_one_raw(Statement::from_string(
                    DbBackend::Postgres,
                    "SELECT nextval(pg_get_serial_sequence('orders', 'id')) AS id".to_string(),
                ))
                .await?
                .ok_or(PurchaseError::NotFound)?;
            let order_id: i64 = row.try_get("", "id")?;
            let order = OrdersGateway::insert(
                &tx,
                orders_entity::ActiveModel {
                    id: Set(order_id),
                    purchase_id: Set(purchase.id),
                    tenant_id: Set(tenant_id),
                    number: Set(format!("ORD-{order_id}")),
                    customer_id: Set(customer.id),
                    customer_name: Set(customer.name.clone()),
                    customer_tax_id: Set(tax_id.to_owned()),
                    customer_email: Set(order_email.clone()),
                    customer_phone: Set(order_phone.clone()),
                    status: Set(if paid { "paid" } else { "pending_payment" }.into()),
                    payment_status: Set(if paid { "captured" } else { "pending_provider" }.into()),
                    subtotal_cents: Set(seller_subtotal),
                    discount_cents: Set(seller_discount),
                    shipping_cents: Set(seller_shipping),
                    tax_total_cents: Set(0),
                    total_cents: Set(total),
                    coupon_id: Set(pricing.and_then(|s| s.coupon_id)),
                    coupon_code: Set(pricing.and_then(|s| s.coupon_code.clone())),
                    placed_at: Set(chrono::Utc::now().naive_utc()),
                    ..Default::default()
                },
            )
            .await?;
            if let Some(coupon_id) = pricing.and_then(|s| s.coupon_id) {
                tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
                    "INSERT INTO checkout_coupon_reservation (coupon_id,customer_id,purchase_id,order_id,status,expires_at) VALUES ($1,$2,$3,$4,$5,$6)",
                    [coupon_id.into(), customer.id.into(), purchase.id.into(), order.id.into(),
                     (if paid { "redeemed" } else { "reserved" }).into(),
                     (chrono::Utc::now().naive_utc() + chrono::Duration::minutes(15)).into()])).await?;
                if paid {
                    coupon_redemption_entity::ActiveModel { tenant_id: Set(Some(tenant_id)), coupon_id: Set(coupon_id),
                        order_id: Set(order.id), customer_id: Set(customer.id), ..Default::default() }
                        .insert(&tx).await?;
                }
            }
            for mut item in items {
                item.order_id = Set(order.id);
                OrderItemGateway::insert(&tx, item).await?;
            }
            for (kind, address) in [
                ("shipping", &input.shipping_address),
                (
                    "billing",
                    input.billing_address.as_ref().expect("normalized billing"),
                ),
            ] {
                OrderAddressGateway::insert(
                    &tx,
                    order_address_entity::ActiveModel {
                        tenant_id: Set(tenant_id),
                        order_id: Set(order.id),
                        address_type: Set(kind.into()),
                        recipient: Set(address.recipient.clone()),
                        address_line1: Set(address.address_line1.clone()),
                        address_line2: Set(address.address_line2.clone()),
                        locality: Set(address.locality.clone()),
                        administrative_area: Set(address.administrative_area.clone()),
                        postal_code: Set(address.postal_code.clone()),
                        country_code: Set(address.country_code.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            }
            OrderStatusHistoryGateway::insert(
                &tx,
                order_status_history_entity::ActiveModel {
                    tenant_id: Set(tenant_id),
                    order_id: Set(order.id),
                    from_status: Set(None),
                    to_status: Set(if paid { "paid" } else { "pending_payment" }.into()),
                    actor_type: Set("customer".into()),
                    actor_id: Set(Some(user_id)),
                    note: Set(None),
                    ..Default::default()
                },
            )
            .await?;
            PaymentAllocationGateway::insert(
                &tx,
                payment_allocation_entity::ActiveModel {
                    purchase_id: Set(purchase.id),
                    payment_id: Set(payment.id),
                    order_id: Set(order.id),
                    amount_cents: Set(total),
                    ..Default::default()
                },
            )
            .await?;
            allocated = allocated
                .checked_add(total)
                .ok_or(PurchaseError::Validation("amount overflow"))?;
        }
        if allocated != payment.amount_cents {
            return Err(PurchaseError::Validation("allocation mismatch"));
        }
        if let Some(ref checkout) = checkout {
            let updated = tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
                "UPDATE checkout_quote SET purchase_id=$1 WHERE id=$2 AND customer_id=$3 AND purchase_id IS NULL AND expires_at > CURRENT_TIMESTAMP",
                [purchase.id.into(), checkout.quote_id.into(), customer.id.into()])).await?;
            if updated.rows_affected() != 1 { return Err(PurchaseError::Conflict); }
        }
        let detail = PurchaseGateway::detail(&tx, access, purchase.id).await?;
        tx.commit().await?;
        Ok(CreatedPurchase {
            detail,
            replayed: false,
        })
    }

    pub async fn get(&self, user: &User, id: i64) -> Result<PurchaseDetail, PurchaseError> {
        PurchaseGateway::detail(&self.gateway.db, self.access(user).await?, id).await
    }
    pub async fn order(&self, user: &User, id: i64) -> Result<OrderDetail, PurchaseError> {
        PurchaseGateway::order_detail(&self.gateway.db, self.access(user).await?, id).await
    }
    pub async fn orders(
        &self,
        user: &User,
        filter: &OrderFilter,
    ) -> Result<Page<crate::domain::orders::Orders>, PurchaseError> {
        self.gateway.orders(self.access(user).await?, filter).await
    }
    pub async fn purchases(
        &self,
        user: &User,
        filter: &OrderFilter,
    ) -> Result<Page<crate::domain::purchase::Purchase>, PurchaseError> {
        self.gateway
            .purchases(self.access(user).await?, filter)
            .await
    }
    pub async fn history(&self, user: &User, id: i64) -> Result<History, PurchaseError> {
        self.gateway.history(self.access(user).await?, id).await
    }
    pub async fn transactions(
        &self,
        user: &User,
        id: i64,
    ) -> Result<Vec<crate::domain::payment_transaction::PaymentTransaction>, PurchaseError> {
        self.gateway
            .transactions(self.access(user).await?, id)
            .await
    }
}
