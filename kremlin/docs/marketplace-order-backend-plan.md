# Marketplace order backend for Kremlin

## 1. Assessment and agreed scope

The new tables are a useful starting point, but **the migrations do not yet support the complete intended flow and will not all apply successfully as written**.

The agreed model is:

- Customers are global authenticated users, without a seller tenant.
- One request can contain products from several sellers.
- The backend creates one purchase containing one order per seller.
- One future credit-card payment covers the purchase, with allocations to each seller order.
- This implementation delivers persistence and APIs, including automatic order splitting.
- Real payment processing, cart conversion, stock reservations, shipping charges, discounts, and tax calculation remain deferred.

For five products—three from seller X and two from seller Y—the result is one purchase, two orders, and one purchase-level payment record with two allocations.

### Problems found in the migrations and existing code

| Area | Finding and required correction |
|---|---|
| Migration ordering | Coupon redemption is registered before the order table it references. Move it after orders. |
| Order table mapping | The new `Order::Table` resolves to `order`; the existing entity uses `orders`. Standardize on `orders`, retaining existing Rust module names. |
| Address table | `address_line1` is declared twice; `order_id` and `recipient` are not created; shipping/billing purpose is missing. |
| Payment table | `order_id` is not created. The agreed combined-payment model instead requires a purchase reference. |
| Foreign keys | New address/payment/card/transaction migrations use `to_col` where a source column is required. Correct all relationships. |
| Rollback | Several `down` methods omit the table to drop or drop foreign keys without a table. Payment transaction also uses inconsistent constraint names. |
| Card details | Missing uniqueness on `payment_id` for its intended one-to-one relationship. |
| Payment transactions | Missing operation type, status, amount, currency, and retry/deduplication information. Provider transaction ID cannot be required before a provider responds. |
| Customer relationships | Tenant/customer composite references conflict with global customers. Reference customers by customer ID. |
| Tenant integrity | Orders need mandatory seller ownership and consistent seller relationships for items and other children. Coupon redemption needs a valid referenced composite key. |
| Existing backend | Entities, mappings, DTOs, search fields, and tests still contain the removed `ship_*` columns. |
| Authorization | Existing order access assumes customers have a tenant. The tenant audit hook can overwrite the seller ID with `None` for global customers. |
| Obsolete migration | The September 23 payment migration is still declared but not registered. Remove this superseded draft. |

These findings originated in source inspection. The corrected migrations were
subsequently exercised against an isolated PostgreSQL database; see
`marketplace-order-backend.md` for the delivered contract.

## 2. Schema and entity changes

### Purchase and seller orders

Add a `purchase` table containing:

- Internal ID, UUID, authenticated customer ID, currency, status, aggregate totals, and audit fields.
- Idempotency key and normalized request hash, unique per customer and key.
- Customer snapshot fields needed to preserve the purchase independently of later profile edits.

Update `orders`:

- Require `purchase_id`, `tenant_id`, and `customer_id`.
- Enforce one order per seller per purchase.
- Retain customer snapshots, monetary breakdown, order/payment statuses, coupon snapshot fields, and audit fields.
- Generate order numbers on the server from the database-generated ID; widen the number column sufficiently to avoid timestamp collisions.
- Add indexes for customer/date, seller/date/status, and purchase lookup.
- Add the composite uniqueness needed by seller-scoped child foreign keys.

Update order items:

- Retain immutable SKU, product, attributes, quantity, price, and fiscal snapshots.
- Require the SKU seller to match the order seller through validation and composite references.
- Snapshot NCM from the product. Make CFOP and CSOSN nullable until fiscal rules exist; do not invent fiscal values.
- Use checked integer arithmetic and `BIGINT`/Rust `i64` cents throughout the new purchase/order/payment model.

Repair order addresses:

- Add `order_id`, seller ID, recipient, and `address_type`.
- Support exactly one shipping and one billing snapshot per order.
- Retain the new general address fields and remove the duplicate column.
- Copy shipping into a separate billing snapshot when billing is omitted.

### Combined payment

Replace the proposed order-owned payment with purchase-owned `payment`:

- Purchase ID, method, status, amount, currency, installments, optional provider/reference, and audit fields.
- Support multiple attempts over time, with at most one active attempt per purchase.
- Initially create one `credit_card` payment in `pending_provider`; provider fields remain null.

Add `payment_allocation`:

- Payment ID, seller order ID, and allocated amount.
- Unique payment/order pair.
- Validate that each order belongs to the payment’s purchase and that allocations sum to the payment amount.
- Allocations describe amounts attributable to seller orders; settlement and payouts remain future work.

Repair `credit_card_details` and `payment_transaction`:

- Card details are optional until an actual provider supplies metadata; enforce one details row per payment.
- Do not accept or store PAN or CVV. Defer reusable token storage until its provider requirements are defined.
- Transactions record operation, status, amount, currency, idempotency reference, optional provider transaction ID, response information, and timestamps.
- Transaction records are append-only; provider identifiers are uniquely scoped where applicable.
- Do not manufacture successful transactions for the initial pending payment.

### Migration execution and compatibility

- Correct the unapplied replacement migrations directly, based on the stated absence of old order data.
- Order migrations by dependency and reverse that order for rollback.
- Fix coupon redemption’s customer reference and entity relation for global customers.
- Repair the existing cart/customer foreign key that references the absent customer composite key, without redesigning carts or adding cart conversion.
- For prerequisite tables already deployed, use additive corrections rather than assuming edits to old migration files will run again.
- Do not reset shared databases or rewrite migration history automatically.
- Create/update all corresponding SeaORM models, columns, relations, module exports, and prelude exports.

## 3. Business layer and authorization

Follow the existing separation: SeaORM entities, domain objects and `EntityMapper`, gateways, use cases, application DTOs and `Mapper`.

### Atomic purchase creation

Implement `CreatePurchaseUseCase`:

1. Resolve the active customer through the authenticated user’s `user_id`; do not accept customer identity from the request.
2. Validate a nonempty item list and positive quantities; combine duplicate SKU IDs with checked arithmetic.
3. Load active SKUs and products, deriving each seller from catalog data.
4. Snapshot current prices, customer details, product descriptions, attributes, and addresses.
5. Group items by seller.
6. Compute item, order, and purchase totals. Shipping, discounts, and taxes are zero.
7. In one database transaction, persist the purchase, seller orders, items, addresses, initial histories, pending payment, and allocations.
8. Return the complete purchase with its seller orders.

Initial purchase and order state is `pending_payment`. Document that these records do not reserve inventory and are not ready for fulfillment.

Use the idempotency key to return the existing purchase for equivalent retries. Reject reuse with a different normalized request as a conflict. Concurrent retries must create only one purchase.

### Gateways and errors

- Add gateways and domain models for purchases, addresses, payments, allocations, card details, and transactions.
- Update existing order/item/history gateways and mappings.
- Allow aggregate persistence through the same `DatabaseTransaction`; child writes must not open independent transactions.
- Add explicit access context for customer, seller staff, and system administrator.
- Return typed `Result` errors for validation, missing resources, access denial, idempotency conflicts, and persistence failures. Do not turn database failures into empty successful responses.
- Define a provider interface for future charge submission and status retrieval, with test doubles only. Production purchase creation does not invoke it.

### Access rules

- Customers can create purchases and read only their own purchases, orders, and payment summaries across sellers.
- Tenant owners/users can read only their seller orders and allocated amounts.
- Sellers cannot retrieve other sellers’ orders, the full multi-seller purchase, or purchase-level card information.
- SysAdmin can inspect all records.
- Every child-resource lookup verifies access through its parent.

Use audit-only stamping for these new aggregate entities, preserving seller IDs already derived and validated by the use case. Keep existing tenant enforcement for unrelated modules; do not globally make customers unrestricted.

History is written by use cases, with actor identity derived from authentication. Remove public operations that allow arbitrary history entries, payment status changes, or independent item mutation.

## 4. Application APIs and OpenAPI

Retain Axum, camelCase DTOs, existing pagination conventions, localized errors, and Utoipa registration.

| Endpoint | Behavior |
|---|---|
| `POST /purchases` | Authenticated customer creates a purchase and seller orders; requires `Idempotency-Key`. |
| `GET /purchases/paged` | Customer’s purchases; SysAdmin may inspect all. |
| `GET /purchases/{id}` | Purchase details, seller orders, totals, and payment summary. |
| `GET /purchases/{id}/payments` | Authorized purchase payment summaries. |
| `GET /orders` and `/orders/paged` | Retain existing reads with customer-ownership or seller filtering. |
| `GET /orders/{id}` | Order detail with items, address snapshots, and the order’s allocated payment amount. |
| `GET /orders/{id}/status-history` | Read-only order history. |
| `GET /payments/{id}/transactions` | Purchase owner or SysAdmin; sanitized transaction information. |

`CreatePurchaseInput` contains SKU IDs/quantities and shipping plus optional billing address. It does not accept seller IDs, customer IDs, prices, totals, fiscal codes, payment outcomes, or card credentials. Default currency is BRL and installments is one for this phase.

Additional API work:

- Replace legacy order-creation/update contracts with purchase creation.
- Remove independent order-item writes and public history creation.
- Retain existing item/history read routes where practical, using the same parent authorization.
- Provide DTOs for every new resource, but expose dependent records through aggregate reads rather than independent CRUD.
- Return `201` for creation, `200` for idempotent replay, and `409` for key conflicts.
- Extend the existing error mapping and locale keys as necessary.
- Document access rules, pending-payment semantics, zero additional charges, lack of stock reservation, and deferred processing.
- Register every new path/schema and remove obsolete paths/schemas from OpenAPI.
- Include the two-seller example in API documentation.

No public payment execution, webhook, refund, cancellation, or fulfillment mutation endpoint is included in this persistence phase.

## 5. Verification and delivery

### Database tests

Use an isolated PostgreSQL database:

- Apply the complete migration chain from empty.
- Exercise order/payment rollback and reapplication.
- Verify the prerequisite upgrade path.
- Confirm entity/schema agreement, foreign keys, uniqueness, checks, and indexes.
- Verify transaction rollback leaves no partial purchase when any seller order fails.

### Business and API tests

- Five products across two sellers create one purchase and exactly two correctly allocated orders.
- Global customer authentication preserves each seller ID.
- Customer A cannot access customer B’s records.
- Seller X cannot inspect seller Y’s orders or the combined purchase.
- Inactive/missing SKUs, sellerless SKUs, invalid quantities, missing required customer details, and arithmetic overflow fail cleanly.
- Duplicate SKU inputs are normalized correctly.
- Retries and concurrent identical requests do not duplicate purchases.
- Catalog/profile/address changes do not alter stored snapshots.
- Payment remains pending; no successful transaction, stock change, coupon redemption, or provider call occurs.
- DTOs do not expose card tokens or accept client-controlled payment status/totals.
- OpenAPI matches actual routes, request bodies, responses, and authentication requirements.

Run formatting checks, workspace compilation, workspace tests, the existing mock test configuration, integration tests, and `git diff --check`. Separate pre-existing failures from regressions.

Deliver all changes inside the Rust `kremlin` project, together with a concise backend contract/migration note. No frontend, external payment provider, deployment, or shared-database mutation is included.
