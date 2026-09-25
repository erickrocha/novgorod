use crate::endpoints::json::orders_json::*;
use business::domain::marketplace::{OrderDetail, PaymentDetail, PurchaseDetail};

impl From<business::domain::purchase::Purchase> for PurchaseJson {
    fn from(value: business::domain::purchase::Purchase) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            customer_id: value.customer_id,
            customer_name: value.customer_name,
            customer_tax_id: value.customer_tax_id,
            customer_email: value.customer_email,
            customer_phone: value.customer_phone,
            currency: value.currency,
            status: value.status,
            subtotal_cents: value.subtotal_cents,
            discount_cents: value.discount_cents,
            shipping_cents: value.shipping_cents,
            tax_total_cents: value.tax_total_cents,
            total_cents: value.total_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::orders::Orders> for OrdersJson {
    fn from(value: business::domain::orders::Orders) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            purchase_id: value.purchase_id,
            tenant_id: value.tenant_id,
            number: value.number,
            customer_id: value.customer_id,
            customer_name: value.customer_name,
            customer_tax_id: value.customer_tax_id,
            customer_email: value.customer_email,
            customer_phone: value.customer_phone,
            status: value.status,
            payment_status: value.payment_status,
            subtotal_cents: value.subtotal_cents,
            discount_cents: value.discount_cents,
            shipping_cents: value.shipping_cents,
            tax_total_cents: value.tax_total_cents,
            total_cents: value.total_cents,
            coupon_id: value.coupon_id,
            coupon_code: value.coupon_code,
            placed_at: value.placed_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::order_item::OrderItem> for OrderItemJson {
    fn from(value: business::domain::order_item::OrderItem) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            tenant_id: value.tenant_id,
            order_id: value.order_id,
            sku_id: value.sku_id,
            sku_code: value.sku_code,
            product_name: value.product_name,
            attributes_desc: value.attributes_desc,
            quantity: value.quantity,
            unit_price_cents: value.unit_price_cents,
            discount_cents: value.discount_cents,
            ncm: value.ncm,
            cfop: value.cfop,
            csosn: value.csosn,
            icms_rate_bp: value.icms_rate_bp,
            tax_cents: value.tax_cents,
            total_cents: value.total_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::order_status_history::OrderStatusHistory> for OrderStatusHistoryJson {
    fn from(value: business::domain::order_status_history::OrderStatusHistory) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            tenant_id: value.tenant_id,
            order_id: value.order_id,
            from_status: value.from_status,
            to_status: value.to_status,
            actor_type: value.actor_type,
            actor_id: value.actor_id,
            note: value.note,
            created_at: value.created_at,
        }
    }
}
impl From<business::domain::order_address::OrderAddress> for OrderAddressJson {
    fn from(value: business::domain::order_address::OrderAddress) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            tenant_id: value.tenant_id,
            order_id: value.order_id,
            address_type: value.address_type,
            recipient: value.recipient,
            address_line1: value.address_line1,
            address_line2: value.address_line2,
            locality: value.locality,
            administrative_area: value.administrative_area,
            postal_code: value.postal_code,
            country_code: value.country_code,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::payment::Payment> for PaymentJson {
    fn from(value: business::domain::payment::Payment) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            purchase_id: value.purchase_id,
            method: value.method,
            status: value.status,
            installments: value.installments,
            amount_cents: value.amount_cents,
            currency: value.currency,
            gateway_provider: value.gateway_provider,
            gateway_reference: value.gateway_reference,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::credit_card_details::CreditCardDetails> for CreditCardDetailsJson {
    fn from(value: business::domain::credit_card_details::CreditCardDetails) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            payment_id: value.payment_id,
            cardholder_name: value.cardholder_name,
            brand: value.brand,
            last_four_digits: value.last_four_digits,
            expiration_month: value.expiration_month,
            expiration_year: value.expiration_year,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<business::domain::payment_transaction::PaymentTransaction> for PaymentTransactionJson {
    fn from(value: business::domain::payment_transaction::PaymentTransaction) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            payment_id: value.payment_id,
            operation: value.operation,
            status: value.status,
            amount_cents: value.amount_cents,
            currency: value.currency,
            gateway_provider: value.gateway_provider,
            gateway_transaction_id: value.gateway_transaction_id,
            response_code: value.response_code,
            created_at: value.created_at,
        }
    }
}
impl From<business::domain::payment_allocation::PaymentAllocation> for PaymentAllocationJson {
    fn from(value: business::domain::payment_allocation::PaymentAllocation) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            purchase_id: value.purchase_id,
            payment_id: value.payment_id,
            order_id: value.order_id,
            amount_cents: value.amount_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<OrderDetail> for OrderDetailJson {
    fn from(value: OrderDetail) -> Self {
        Self {
            order: value.order.into(),
            items: value.items.into_iter().map(Into::into).collect(),
            addresses: value.addresses.into_iter().map(Into::into).collect(),
            allocations: value.allocations.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PaymentDetail> for PaymentDetailJson {
    fn from(value: PaymentDetail) -> Self {
        Self {
            payment: value.payment.into(),
            card: value.card.map(Into::into),
        }
    }
}
pub struct PurchaseMapper;
impl PurchaseMapper {
    pub fn json(value: PurchaseDetail) -> PurchaseDetailJson {
        PurchaseDetailJson {
            purchase: value.purchase.into(),
            orders: value.orders.into_iter().map(Into::into).collect(),
            payments: value.payments.into_iter().map(Into::into).collect(),
        }
    }
    pub fn input(
        value: CreatePurchaseInputJson,
    ) -> business::domain::marketplace::CreatePurchaseInput {
        use business::domain::marketplace::PurchaseItemInput;
        business::domain::marketplace::CreatePurchaseInput {
            items: value
                .items
                .into_iter()
                .map(|v| PurchaseItemInput {
                    sku_id: v.sku_id,
                    quantity: v.quantity,
                })
                .collect(),
            shipping_address: value.shipping_address.into(),
            billing_address: value.billing_address.map(Into::into),
        }
    }
}
impl From<AddressInputJson> for business::domain::marketplace::AddressInput {
    fn from(v: AddressInputJson) -> Self {
        Self {
            recipient: v.recipient,
            address_line1: v.address_line1,
            address_line2: v.address_line2,
            locality: v.locality,
            administrative_area: v.administrative_area,
            postal_code: v.postal_code,
            country_code: v.country_code,
        }
    }
}
