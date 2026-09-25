use super::{
    credit_card_details_gateway::CreditCardDetailsGateway,
    order_address_gateway::OrderAddressGateway, order_item_gateway::OrderItemGateway,
    order_status_history_gateway::OrderStatusHistoryGateway, orders_gateway::OrdersGateway,
    payment_allocation_gateway::PaymentAllocationGateway, payment_gateway::PaymentGateway,
    payment_transaction_gateway::PaymentTransactionGateway,
};
use crate::commons::entity_mapper::EntityMapper;
use crate::domain::{
    credit_card_details::CreditCardDetailsEntityMapper,
    marketplace::*,
    order_address::OrderAddressEntityMapper,
    order_item::OrderItemEntityMapper,
    order_status_history::OrderStatusHistoryEntityMapper,
    orders::OrdersEntityMapper,
    payment::PaymentEntityMapper,
    payment_allocation::PaymentAllocationEntityMapper,
    payment_transaction::{PaymentTransaction, PaymentTransactionEntityMapper},
    purchase::PurchaseEntityMapper,
};
use entity::{
    catalog_attribute_entity, catalog_attribute_value_entity, customer_entity, orders_entity,
    payment_entity, product_entity, purchase_entity, sku_attribute_value_entity, sku_entity,
};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DbConn, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Select,
};

pub struct PurchaseGateway {
    pub(crate) db: DbConn,
}
impl PurchaseGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub(crate) async fn customer_for_user<C: ConnectionTrait>(
        db: &C,
        user_id: i64,
    ) -> Result<customer_entity::Model, PurchaseError> {
        let mut customers = customer_entity::Entity::find()
            .filter(customer_entity::Column::UserId.eq(user_id))
            .filter(customer_entity::Column::Active.eq(true))
            .limit(2)
            .all(db)
            .await?;
        if customers.len() != 1 {
            return Err(PurchaseError::Forbidden);
        }
        Ok(customers.remove(0))
    }

    pub(crate) async fn catalog<C: ConnectionTrait>(
        db: &C,
        sku_id: i64,
    ) -> Result<(sku_entity::Model, product_entity::Model, Option<String>), PurchaseError> {
        let sku = sku_entity::Entity::find_by_id(sku_id)
            .filter(sku_entity::Column::Active.eq(true))
            .one(db)
            .await?
            .ok_or(PurchaseError::Validation("SKU unavailable"))?;
        let product = product_entity::Entity::find_by_id(sku.product_id)
            .filter(product_entity::Column::Active.eq(true))
            .one(db)
            .await?
            .ok_or(PurchaseError::Validation("product unavailable"))?;
        if sku.tenant_id.is_none() || sku.tenant_id != product.tenant_id || sku.price_cents < 0 {
            return Err(PurchaseError::Validation("invalid catalog seller or price"));
        }
        let attributes = sku_attribute_value_entity::Entity::find()
            .filter(sku_attribute_value_entity::Column::SkuId.eq(sku_id))
            .filter(sku_attribute_value_entity::Column::TenantId.eq(sku.tenant_id))
            .order_by_asc(sku_attribute_value_entity::Column::AttributeId)
            .all(db)
            .await?;
        let mut descriptions = Vec::new();
        for attribute in attributes {
            let name = catalog_attribute_entity::Entity::find_by_id(attribute.attribute_id)
                .one(db)
                .await?
                .ok_or(PurchaseError::Validation("missing catalog attribute"))?;
            let value =
                catalog_attribute_value_entity::Entity::find_by_id(attribute.attribute_value_id)
                    .one(db)
                    .await?
                    .ok_or(PurchaseError::Validation("missing catalog value"))?;
            descriptions.push(format!("{}: {}", name.name, value.value));
        }
        Ok((
            sku,
            product,
            if descriptions.is_empty() {
                None
            } else {
                Some(descriptions.join(", "))
            },
        ))
    }

    pub(crate) async fn detail<C: ConnectionTrait>(
        db: &C,
        access: PurchaseAccess,
        id: i64,
    ) -> Result<PurchaseDetail, PurchaseError> {
        let purchase = purchase_entity::Entity::find_by_id(id)
            .one(db)
            .await?
            .map(PurchaseEntityMapper::from_model)
            .ok_or(PurchaseError::NotFound)?;
        if !access.can_read_purchase(&purchase) {
            return Err(PurchaseError::NotFound);
        }
        let mut orders = Vec::new();
        for order in OrdersGateway::for_parent(db, id).await? {
            orders.push(Self::order_detail(db, access, order.id).await?);
        }
        let mut payments = Vec::new();
        for payment in PaymentGateway::for_parent(db, id).await? {
            let card = CreditCardDetailsGateway::for_parent(db, payment.id)
                .await?
                .into_iter()
                .next()
                .map(CreditCardDetailsEntityMapper::from_model);
            payments.push(PaymentDetail {
                payment: PaymentEntityMapper::from_model(payment),
                card,
            });
        }
        Ok(PurchaseDetail {
            purchase,
            orders,
            payments,
        })
    }

    pub(crate) async fn order_detail<C: ConnectionTrait>(
        db: &C,
        access: PurchaseAccess,
        id: i64,
    ) -> Result<OrderDetail, PurchaseError> {
        let order = orders_entity::Entity::find_by_id(id)
            .one(db)
            .await?
            .map(OrdersEntityMapper::from_model)
            .ok_or(PurchaseError::NotFound)?;
        if !access.can_read_order(&order) {
            return Err(PurchaseError::NotFound);
        }
        Ok(OrderDetail {
            order,
            items: OrderItemEntityMapper::from_models(OrderItemGateway::for_parent(db, id).await?),
            addresses: OrderAddressEntityMapper::from_models(
                OrderAddressGateway::for_parent(db, id).await?,
            ),
            allocations: PaymentAllocationEntityMapper::from_models(
                PaymentAllocationGateway::for_parent(db, id).await?,
            ),
        })
    }

    pub(crate) async fn history(
        &self,
        access: PurchaseAccess,
        id: i64,
    ) -> Result<History, PurchaseError> {
        Self::order_detail(&self.db, access, id).await?;
        Ok(OrderStatusHistoryEntityMapper::from_models(
            OrderStatusHistoryGateway::for_parent(&self.db, id).await?,
        ))
    }

    pub(crate) async fn transactions(
        &self,
        access: PurchaseAccess,
        id: i64,
    ) -> Result<Vec<PaymentTransaction>, PurchaseError> {
        let payment = payment_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(PurchaseError::NotFound)?;
        Self::detail(&self.db, access, payment.purchase_id).await?;
        Ok(PaymentTransactionEntityMapper::from_models(
            PaymentTransactionGateway::for_parent(&self.db, id).await?,
        ))
    }

    fn order_query(access: PurchaseAccess, filter: &OrderFilter) -> Select<orders_entity::Entity> {
        use orders_entity::Column as C;
        let mut query = orders_entity::Entity::find();
        match access {
            PurchaseAccess::Customer { customer_id, .. } => {
                query = query.filter(C::CustomerId.eq(customer_id))
            }
            PurchaseAccess::Seller(tenant_id) => query = query.filter(C::TenantId.eq(tenant_id)),
            PurchaseAccess::Admin => (),
        }
        if let Some(id) = filter.tenant_id {
            query = query.filter(C::TenantId.eq(id));
        }
        if let Some(id) = filter.customer_id {
            query = query.filter(C::CustomerId.eq(id));
        }
        if let Some(status) = &filter.status {
            query = query.filter(C::Status.eq(status));
        }
        if let Some(term) = &filter.query {
            let pattern = format!("%{term}%");
            query = query.filter(
                Condition::any()
                    .add(C::Number.like(&pattern))
                    .add(C::CustomerName.like(&pattern))
                    .add(C::Status.like(&pattern)),
            );
        }
        query
    }

    pub(crate) async fn orders(
        &self,
        access: PurchaseAccess,
        filter: &OrderFilter,
    ) -> Result<Page<crate::domain::orders::Orders>, PurchaseError> {
        use orders_entity::Column as C;
        let query = Self::order_query(access, filter);
        let total = query.clone().count(&self.db).await?;
        let column = match filter.sort_by.as_str() {
            "number" => C::Number,
            "status" => C::Status,
            "paymentStatus" => C::PaymentStatus,
            "totalCents" => C::TotalCents,
            "placedAt" => C::PlacedAt,
            "createdAt" => C::CreatedAt,
            _ => C::Id,
        };
        let direction = if filter.descending {
            Order::Desc
        } else {
            Order::Asc
        };
        let models = query
            .order_by(column, direction.clone())
            .order_by(C::Id, direction)
            .offset(filter.offset)
            .limit(filter.limit)
            .all(&self.db)
            .await?;
        Ok(Page {
            items: OrdersEntityMapper::from_models(models),
            total,
        })
    }

    pub(crate) async fn purchases(
        &self,
        access: PurchaseAccess,
        filter: &OrderFilter,
    ) -> Result<Page<crate::domain::purchase::Purchase>, PurchaseError> {
        use purchase_entity::Column as C;
        let mut query = purchase_entity::Entity::find();
        match access {
            PurchaseAccess::Customer { customer_id, .. } => {
                query = query.filter(C::CustomerId.eq(customer_id))
            }
            PurchaseAccess::Seller(_) => return Err(PurchaseError::Forbidden),
            PurchaseAccess::Admin => (),
        }
        if let Some(id) = filter.customer_id {
            query = query.filter(C::CustomerId.eq(id));
        }
        if let Some(status) = &filter.status {
            query = query.filter(C::Status.eq(status));
        }
        if let Some(term) = &filter.query {
            query = query.filter(C::CustomerName.like(format!("%{term}%")));
        }
        let total = query.clone().count(&self.db).await?;
        let column = match filter.sort_by.as_str() {
            "status" => C::Status,
            "totalCents" => C::TotalCents,
            "createdAt" => C::CreatedAt,
            _ => C::Id,
        };
        let direction = if filter.descending {
            Order::Desc
        } else {
            Order::Asc
        };
        let models = query
            .order_by(column, direction.clone())
            .order_by(C::Id, direction)
            .offset(filter.offset)
            .limit(filter.limit)
            .all(&self.db)
            .await?;
        Ok(Page {
            items: PurchaseEntityMapper::from_models(models),
            total,
        })
    }
}
