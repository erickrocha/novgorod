# Checkout with customer identity, delivery, and Mercado Pago

## Summary

Replace the simulated checkout with a working sandbox flow: authenticate the customer, confirm contact and delivery details, calculate shipping and seller coupons on the server, then pay by credit card through Mercado Pago.

Collect one payment into the platform account while retaining separate orders and payment allocations for each seller. Keep payment integration behind provider interfaces so another provider can be added later.

## Customer and checkout experience

- On checkout entry, validate the session and fetch the current customer and saved addresses. Show a loading state until this finishes. Expired sessions return to the existing login form; registration returns the customer to checkout with their cart preserved.
- Display **name, CPF, email, and phone**. Name and CPF are read-only. Email and phone remain editable and apply only to the purchase and its seller orders.
- Add a separate profile-completion step for customers missing CPF. Validate and save it once; reject attempts to replace an existing CPF through this endpoint.
- Show saved addresses, placing the default first, and explicitly ask whether to use a saved address for delivery. Customers can choose another saved address or enter an order-specific address. With no complete saved address, require manual entry.
- Include recipient, street/number, complement, city, UF, CEP, and country. Default country to Brazil; remove fabricated CEP, city, and state values. Copy the confirmed address into each seller order without modifying the customer's address book.
- Reveal payment selection after contact and delivery validation. Offer only **Cartão de crédito**; selecting it mounts the Mercado Pago card form.
- Keep the seller-grouped summary, showing each seller's merchandise subtotal, coupon discount, shipping, and total, followed by the overall amount.
- Replace the fake confirmation and random order number with backend purchase/order numbers and actual payment status. Pending payments show “processing”; rejected payments offer recovery. Clear purchased cart items only after confirmed success.

## Backend contracts and pricing

- Add authenticated `GET /customers/me`, returning the actual customer ID, name, CPF, email, phone, and owned addresses. Resolve ownership from the authenticated user, never a client-supplied customer ID. Update frontend session hydration accordingly.
- Add `POST /customers/me/tax-id` for one-time CPF completion, using existing CPF validation and uniqueness rules.
- Add `POST /checkout/quotes`. Input includes SKU quantities, delivery address or owned address ID, and optional coupon code per seller. Return an owned quote ID, expiration, canonical items, seller breakdowns, and overall totals in integer cents.
- Quotes expire after 15 minutes. Recalculate and compare their prices, shipping, and coupon eligibility when creating the purchase; changes require a new quote and customer review before payment.
- Extend `POST /purchases` to accept the quote ID and order-only email/phone. Name and CPF always come from the customer record. Preserve idempotent purchase creation and immutable customer/address snapshots.
- Preserve the existing purchase API for compatibility, but require the new validated checkout flow before initiating payment.
- Resolve every cart line to a real active SKU. Automatically resolve products with exactly one active SKU; require variant selection for ambiguous products, including older saved carts.
- Calculate shipping once per seller using its configured destination-UF rate. Sum the seller charges. Missing rates block checkout for that destination with a seller-specific message.
- Accept one coupon per seller. Validate seller ownership, active dates, minimum merchandise subtotal, and total/per-customer usage limits. Support existing `PERCENTAGE` and `FIXED` types; round percentage discounts to cents, cap discounts at merchandise subtotal, and exclude shipping.
- Reserve coupon usage transactionally when creating a purchase, redeem it once on successful payment, and release it on confirmed cancellation or failure. Unsubmitted purchases expire after 15 minutes; uncertain provider outcomes retain their reservations until reconciled.
- Replace browser-generated shipping estimates and local discount calculations with server quotes. The displayed total, stored purchase total, payment amount, and seller allocations must agree.

## Mercado Pago and provider abstraction

- Use MercadoPago.js **CardForm with secure fields** for card number, expiration, and security code. Collect cardholder details separately from the read-only customer identity. Tokenize in the browser and send only the token and necessary payment metadata to Kremlin. Follow the documented [CardForm payment flow](https://www.mercadopago.com.br/developers/pt/docs/checkout-api-payments/integration-configuration/card/integration-via-cardform?scope=prod).
- Offer Mercado Pago's eligible credit-card installment options, displaying installment amounts and any financing cost. Reinitialize options whenever the payable amount changes. Reject other payment types server-side.
- Add a frontend provider adapter for mounting, submitting, and disposing of the card form. Extend the existing backend `PaymentProviderGateway` with provider-neutral charge, status, and normalized outcome handling; isolate Mercado Pago payloads and status mappings inside its adapter.
- Add authenticated payment configuration, submission, and status endpoints under checkout/purchases. The server supplies public configuration, derives the charge amount and payer identity from the purchase, and stores the chosen installment count.
- Use Mercado Pago's Payments API with a stable idempotency key per payment attempt. Persist the attempt before calling the provider; perform network calls outside database transactions. Repeated submission must return the same attempt rather than create another charge.
- Map provider results into pending, captured, and failed states. Update purchase, seller orders, transactions, and coupon reservations atomically. A timeout remains pending until status reconciliation establishes the outcome.
- Add a signed Mercado Pago webhook endpoint and background reconciliation for unresolved attempts. Fetch provider payment details and verify merchant, reference, currency, and amount before changing local state. Process duplicate notifications idempotently, following [Mercado Pago's webhook verification](https://www.mercadopago.com.br/developers/pt/docs/checkout-bricks/additional-content/your-integrations/notifications/webhooks).
- Never store or log card numbers, CVV, or payment tokens. Store only permitted card metadata returned by the provider; make unavailable metadata nullable through an additive migration.
- Add migrations for checkout quotes, coupon reservations, and payment-attempt recovery fields. Configure sandbox public key, backend access token, and webhook secret through environment settings. Missing configuration disables payment with a clear error.

## Verification and defaults

- Test session validation, login/signup return, profile hydration, read-only identity, one-time CPF completion, and order-only contact edits.
- Test saved/default/missing addresses, manual delivery entry, address ownership, and immutable order snapshots.
- Test SKU resolution, multiple sellers, missing shipping rates, coupon boundaries and concurrent usage limits, quote expiry, changed totals, and exact allocation sums.
- Test approved, declined, pending, timeout, duplicate submission, duplicate/out-of-order webhook, invalid signature, session expiry, and interrupted-payment recovery.
- Run frontend tests/build, Kremlin tests, isolated PostgreSQL integration tests, and sandbox card scenarios. Verify desktop and mobile checkout when a browser is available.
- Defaults: Portuguese UI, BRL, Brazilian delivery addresses, billing address copied from delivery, no saved cards, and no seller payouts. Zero-total purchases complete without a provider charge.
- This iteration targets sandbox payments. Inventory reservation, fulfillment, refund initiation, and production activation remain separate work. End-to-end provider verification requires Mercado Pago test credentials and an accessible webhook URL.
