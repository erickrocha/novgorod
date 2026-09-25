# Marketplace purchase backend

`POST /purchases` accepts an authenticated customer's SKU IDs and quantities, a
shipping address, an optional billing address, and a required `Idempotency-Key`
header. Customer identity comes from the bearer token. The request cannot set
sellers, prices, totals, payment outcomes, or card credentials. Equivalent
retries return the original purchase with HTTP 200; a new purchase returns 201;
reusing a key for different input returns 409.

The backend validates active SKUs and products, groups lines by catalog seller,
and writes one purchase, one order per seller, item and address snapshots,
initial status history, one pending credit-card payment, and an allocation for
each order in one transaction. For five items from two sellers, the response
contains one purchase, two orders, one payment, and two allocations. Currency is
BRL; shipping, discounts, and taxes are zero. Billing defaults to a separate
copy of shipping. Orders and the purchase start in `pending_payment`; payment
starts in `pending_provider`. Creation does not reserve stock, charge a card,
or prepare an order for fulfillment.

Customers can list and read their own purchases and orders, including payment
summaries. Seller staff can list and read only their seller's orders, allocated
amounts, and status histories. SysAdmin can inspect all records. Purchase-level
payments and transactions are visible only to the owning customer or SysAdmin.
The read routes are `/purchases/paged`, `/purchases/{id}`,
`/purchases/{id}/payments`, `/orders`, `/orders/paged`, `/orders/{id}`,
`/orders/{id}/status-history`, and `/payments/{id}/transactions`.

The migration chain creates `purchase` before `orders`, then order children,
purchase-owned payments, and payment allocations. Coupon redemption follows
orders. A final additive migration repairs the cart/customer foreign key for
installations that already applied the old cart migration. Apply migrations
normally to the target database after a backup; do not reset a shared database
or edit its migration ledger. The isolated PostgreSQL tests cover migration
round trips and the previously deployed cart relationship.

Payment submission, webhooks, refunds, cancellations, fulfillment mutations,
cart conversion, stock reservation, and shipping or tax calculation remain
future work.
