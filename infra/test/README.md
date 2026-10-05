# Test stack (kremlin)

Disposable PostgreSQL plus a Rust container that runs the kremlin tests. Same Postgres image as `../dev`, but nothing persists (tmpfs) and it listens on port 5433, so it never touches the dev database.

```
./run.sh                      # cargo test --workspace -- --include-ignored
./run.sh clippy               # cargo clippy --workspace --all-targets
./run.sh cargo test -p business --test cart_cleanup_postgres -- --ignored --nocapture
```

Requires Docker with the compose plugin. The first run downloads the images and compiles the workspace; the cargo registry and build cache live in named volumes, so later runs are fast. `run.sh` tears the stack down (`docker compose down -v`) when it exits.

- The Postgres tests are `#[ignore]d`, read `KREMLIN_TEST_DATABASE_URL`, and refuse a database whose name does not end in `_test`. The compose file sets both. Each test creates its own schema and runs the migrations in it.
- Credentials are throwaway defaults for a local, non-persistent database. Override with `TEST_DATABASE_NAME`, `TEST_DATABASE_USER`, `TEST_DATABASE_PASSWORD`, `TEST_DATABASE_PORT`.
- The source is mounted from `../../kremlin`; build output goes to the `cargo_target` volume, not into the repo.

## HTTP kit (NOV-3 T-CHK-01..03)

Runs the kremlin app itself on the disposable database and drives it over HTTP. Python 3 stdlib on the host, plus Docker. Nothing here touches production code, and nothing persists.

```
docker compose up -d --build --wait kremlin mpd   # app on :8080, Mercado Pago double on :8099, Postgres on :5433
./kit.py seed                                      # fixtures E1 (once per fresh stack)
./kit.py tchk08                                    # Customer auth-layer regression: marketplace, store mode, coupon payment
./kit.py tchk11                                    # Capture consumes and failure releases stock, including replays
./kit.py tchk12                                    # Expired stock is released; in-flight payments are preserved
./kit.py tchk13                                    # Payment window and stock re-reservation on retry
./kit.py tchk14                                    # Late capture consumes available stock or logs oversell
./kit.py tchk15                                    # Coupon write validation
./kit.py tchk16                                    # Only SysAdmin can create coupon redemptions
./kit.py --help                                    # every command, with examples in the header of kit.py
./smoke.py                                         # up, seed, login, public GET, quote, purchase, payment, webhook, reset, down -v
docker compose down -v                             # always tear down; the database is gone with it
```

Stack (`docker-compose.yml`): `kremlin` runs `cargo run -p application --bin kremlin` (migrations and the SysAdmin seed happen at boot; healthy when the port answers; the first start compiles for a few minutes, `down -v` also drops the cargo cache volumes), `mpd` is `mpd.py` in `python:3-alpine`. Every env var the app needs is set in the compose file with a made-up local-only value (JWT secrets, SysAdmin password, `MP_*`, `GATEWAY_TOKEN`); override with `TEST_ACCESS_TOKEN_SECRET`, `TEST_REFRESH_TOKEN_SECRET`, `TEST_SYSADMIN_PASSWORD`, `TEST_MP_WEBHOOK_SECRET`, `TEST_GATEWAY_TOKEN`, `TEST_KREMLIN_PORT`, `TEST_MPD_PORT`, `TEST_KREMLIN_LOG`, `TEST_STOCK_EXPIRY_JOB_ENABLED`. No `.env` file is read. SQS, S3 and the shipping/payment encryption keys are not configured (the consumer is off, tenants use fixed shipping, the provider comes from the `MP_*` variables).

### Fixtures (`kit.py seed`, test-cases.md environment E1)

| What | Created by |
|------|-----------|
| T1 (listed), T2; shipping mode `fixed` (the default without settings); rates T1 SP 1500 / RJ 0, T2 SP 1000 | API: `POST /tenant`, `PUT /tenant/{id}/listing`, `POST /shipping-rates` |
| S1 (TenantOwner T1, `s1`), S2 (TenantOwner T2, `s2`), SysAdmin (`sysadmin`) | API: `POST /user` as SysAdmin |
| C1, C2, C3 with user and valid CPF (`c1`..`c3`) | API: `POST /signup` |
| SKUs A1 5000, A2 1005, A3 999, Z1 0, X1 (inactive), W1 (inactive product), P1 (PRO-1), V1/V2 (PRO-2) on T1; B1 3000 on T2; stock 10, reserved 0 | API: `POST /products`, `/skus`, `/sku-stocks` as the owner |
| Coupons CP10, CP10b, CP33, CF5000, CP100, CP100b (1 per customer), CP1U (max 1 use), CIN, CFUT, CEXP, CMIN (T1), CB (T2) | API: `POST /coupons` as the owner (`kit.py coupon` posts more) |
| Saved addresses AD1 (C1, SP), AD2 (C1, SP), AD3 (C1, RJ), AD9 (C2, SP), AD10 (C3, SP) | **SQL**: `POST /customer-addresses` as a Customer returns 400 (the tenant hook writes `tenant_id` NULL, the column is NOT NULL); `POST /signup` with an address fails the same way. Rows are inserted with `tenant_id` = T1 |
| U1 (TenantUser T1, `u1`, same password as S1) | **SQL**: no API creates a TenantUser (`POST /user` as an owner forces TenantOwner) |

`./kit.py ids` prints the ids. Names work as `{{A1}}`, `{{AD1}}`, `{{T1}}`, `{{C1}}`, `{{CP10}}` in paths, headers and bodies. `./kit.py reset` returns to the seeded state between cases: truncates quotes, purchases, orders, payments, reservations, carts and coupons (ids restart at 1), restores stock to 10/0, recreates the coupons, resets the double. Users, tenants, SKUs and addresses stay.

`./kit.py tchk08` resets the disposable data and runs the T-CHK-08 regression through the real HTTP server and authentication middleware. It verifies zero-total coupon purchases as C1 without a tenant header and with `x-tenant-id: 1`, then confirms an approved CP10 payment as C1; each result is checked against Postgres.

`./kit.py tchk11` resets the disposable data and verifies through HTTP/Postgres that an approved A1 x3 payment consumes its stock reservation once, while a rejected payment releases its reservation once across status and signed webhook replays.

`./kit.py tchk12` verifies expired reservations are released only for `pending_provider` payments without an attempt or failed payments; a `pending` payment stays reserved. Run with `TEST_STOCK_EXPIRY_JOB_ENABLED=false` when starting Compose, then `./kit.py tchk12 --expect-disabled`, to verify the switch prevents releases.

`./kit.py tchk13` verifies TC-25: attempts after the 15-minute window are refused without MPD calls; retries inside the window re-reserve stock before charging; insufficient stock returns 409 without changing the failed payment or released reservation; pending attempts remain in flight.

`./kit.py tchk14` verifies late approved webhooks: released stock is consumed if available; otherwise the payment is confirmed, inventory stays unchanged, and one oversell error is logged with purchase, SKU, and quantity.

`./kit.py tchk15` verifies lowercase coupon type normalization plus rejection of invalid coupon types, values, and usage limits on both create and update.

`./kit.py tchk16` verifies `POST /coupon-redemptions` returns 403 for TenantOwner, TenantUser, and Customer, then returns 201 for SysAdmin with the requested tenant scope.

### Driving

```
./kit.py call POST /checkout/quotes --as c1 --json '{"items":[{"skuId":{{A1}},"quantity":1}],"addressId":{{AD1}},"coupons":[{"tenantId":{{T1}},"code":"CP10"}]}'
./kit.py call POST /purchases --as c1 -H 'Idempotency-Key: k1' --json '{"quoteId":1,"email":"c1@test.local","phone":"11988887777"}'
./kit.py call POST /purchases/1/payments/submit --as c1 --json '{"token":"tok","paymentMethodId":"visa","installments":1}'
./kit.py coupon CPX PERCENTAGE 15 --tenant T1 --field maxUses=2
```

`--as` is `sysadmin|s1|s2|u1|c1|c2|c3|anon`; add `-H 'x-tenant-id: 1'` for store mode. Output is the status, the time and the body.

### Mercado Pago double (T-CHK-02)

`mpd.py` answers what `application/src/infrastructure/mercado_pago.rs` calls (`POST /v1/payments`, `GET /v1/payments/{id}`, `GET /v1/payments/search`) through `MP_API_BASE_URL=http://mpd:8099`, and records every call.

```
./kit.py mpd approved|rejected|pending|authorized|error [--amount-cents N]   # applies to the next charges; --amount-cents answers a different amount
./kit.py mpd-calls        # every call received, oldest first (method, path, query, body, idempotency key)
./kit.py mpd-payments     # payments it created (the id is what a webhook carries)
./kit.py mpd-reset        # clear calls, payments and config
./kit.py webhook PAYMENT_ID [--request-id X] [--bad]   # POST /webhooks/mercado-pago, signed with MP_WEBHOOK_SECRET
```

Also: a card `token` starting with `reject`, `pending`, `authorized` or `error` overrides the mode for that one charge; `POST /_mpd/payments/{id}` with `{"status": "...", "amount_cents": N}` changes a stored payment (for the status call, the reconciliation and a webhook); the same `X-Idempotency-Key` returns the same payment. Signature: header `x-signature: ts=<ms>,v1=<hex HMAC-SHA256 of "id:<data.id>;request-id:<x-request-id>;ts:<ts>;">`, as `verify_signature` in `checkout_payment_endpoint.rs` checks it (`--bad` signs with a wrong secret, expect 401).

### Evidence, time and concurrency (T-CHK-03)

```
./kit.py evidence all [PURCHASE_ID]    # purchase, orders, payment, allocation, coupon-reservation, redemption, stock, quote, address checksum, stock-reservation
./kit.py evidence stock-reservation    # prints a note while checkout_stock_reservation does not exist yet
./kit.py db "SELECT ..."               # any read-only query (the session is read-only; a write fails)
./kit.py shift quote 1 20              # checkout_quote.expires_at (and created_at) 20 minutes into the past
./kit.py shift purchase 1 20           # purchase.created_at
./kit.py shift attempt 1 20            # payment.attempt_started_at of the purchase
./kit.py shift reservation 1 20        # expires_at of the coupon reservation and, when the table exists, the stock reservation
./kit.py sql "UPDATE ..."              # anything else (writes)
./kit.py burst --repeat 5 POST /purchases --as c1 -H 'Idempotency-Key: same' --json '{...}'
./kit.py burst --spec '[{"as":"c1","path":"/purchases","headers":{"Idempotency-Key":"a"},"json":{"quoteId":1,"email":"c1@test.local","phone":"11988887777"}},{"as":"c2","path":"/purchases","headers":{"Idempotency-Key":"b"},"json":{"quoteId":2,"email":"c2@test.local","phone":"11988887777"}}]'
```

`burst` logs in first, holds every request at a barrier and fires them together (threads); it prints one line per request and a status summary. `{{i}}` is the request index (use it for distinct keys). The same functions are importable (`import kit; kit.burst([...])`, `kit.sign(...)`, `kit.api(...)`).

Notes: the fixtures file `.kit-fixtures.json` is git-ignored. Not wired into CI. Used by NOV-3 plan tasks T-CHK-01..03.
