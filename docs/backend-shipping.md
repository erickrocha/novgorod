# Backend shipping

Checkout supports `fixed` and `correios` modes per seller. Sellers without a shipping settings row use existing destination-UF fixed rates. A failed Correios request never falls back to fixed rates. Frontend code is unchanged.

## Deployment

The application applies migration `m20260926_000001_shipping` at startup. It creates `tenant_shipping_settings` and a nullable `orders.shipping_snapshot`. Older quotes and historical orders remain readable without shipping metadata.

Supply these values through the deployment secret manager:

- `SHIPPING_ENCRYPTION_KEYS`: JSON object mapping key versions to 64 hexadecimal characters (32 random bytes each), for example `{"v1":"<64 hex characters>"}`.
- `SHIPPING_ACTIVE_KEY_VERSION`: the version used for new credential writes, for example `v1`.
- `CORREIOS_ENVIRONMENT`: `homologation` (default) or `production`. Credentials must match that environment.

Generate encryption keys with a cryptographically secure generator. Never commit actual keys or carrier credentials. Missing encryption configuration leaves fixed shipping available; saving credentials or quoting with unavailable keys returns 503. Invalid configured keys stop application startup.

Credentials use AES-256-GCM with a new random 96-bit nonce for every write. Authenticated data binds ciphertext to the seller, carrier, and encryption key version. Keep previous keys in the key ring until all corresponding records have been re-encrypted. To rotate, deploy both keys with the new active version, submit a credential update per seller (even an empty `credentials` object re-encrypts existing values), then retire the old key only after verifying no rows reference it. Key rotation must preserve recoverability of backups.

Tokens are isolated by seller and credential version, reused until 60 seconds before expiration, and refreshed once after HTTP 401. Each process allows eight concurrent seller queries (at most sixteen price/time HTTP requests); requests time out after eight seconds, with a 25-second overall bound including admission and authentication. Cache capacity is 1,024 sellers. Errors contain no carrier bodies, tokens, or credentials.

## Seller settings

`GET /tenants/{tenantId}/shipping-settings` returns configuration, its version, and `credentialsConfigured`. Only the matching `TenantOwner` and `SysAdmin` can read or write settings. Seller staff, customers, and other tenants receive 403.

`PUT /tenants/{tenantId}/shipping-settings` returns 204:

```json
{
  "configuration": {
    "mode": "correios",
    "originCep": "01001000",
    "services": [
      {"code": "03298", "name": "PAC"},
      {"code": "03220", "name": "SEDEX"}
    ],
    "packaging": {"weightG": 100, "lengthMm": 10, "widthMm": 10, "heightMm": 10}
  },
  "credentials": {
    "username": "<Meu Correios username>",
    "apiAccessCode": "<API access code>",
    "postingCard": "<posting card>",
    "contract": "<contract number>",
    "regionalIdentifier": "<regional identifier>"
  }
}
```

Use the service codes actually enabled on the seller's contract; the codes above are examples. Omitted credential fields preserve their values. Explicit empty strings are rejected. Credentials are write-only and never returned. Omitted origin CEP defaults from the seller address. Packaging allowances default to zero. Every settings write increments the configuration version and invalidates previously issued quotes. Credential writes also increment the credential version used for token caching.

## Quote and purchase flow

1. `POST /checkout/quotes` retains its existing request shape (items, address ID or inline address, seller coupons). Each seller now includes `shipping`, with `options` and `selectedOption`, configuration version, parcel measurements, origin/destination CEPs, and a fingerprint of the individual SKU inputs. Monetary fields remain integer BRL cents. The default is lowest price, shortest transit time, then service code.
2. To change service, `POST /checkout/quotes/{id}/shipping-selection` with exactly one stored option ID per seller:

   ```json
   [{"tenantId": 1, "optionId": "<option UUID>"}, {"tenantId": 2, "optionId": "<option UUID>"}]
   ```

   It returns 201 with a new quote ID and recalculated totals. The original quote is unchanged; the new quote retains its expiration. Selection makes no carrier calls.
3. `POST /purchases` accepts `{"quoteId":123,"email":"buyer@example.com","phone":"11999999999"}` and the existing `Idempotency-Key` header. A valid quote is required; the legacy item/address payload cannot bypass shipping. The selected snapshot is copied into each order's `shippingSnapshot`. Payment amounts and allocations use those stored totals.

Quotes last 15 minutes. Purchase validation locks and rechecks catalog, stock, coupons, saved address and shipping inputs, without carrier calls. Carrier price fluctuations alone do not change an unexpired quote. Configuration, fixed rates, address, or individual parcel input changes require a new quote. To change cart quantities, create a new quote with the new items. An idempotent retry for an already created purchase returns the original purchase even after quote expiration.

One estimated parcel is built per seller: largest length and width, summed height and weight multiplied by quantities, then packaging allowances. Correios requires positive SKU measurements. Millimeters round upward to centimeters. This release rejects parcels above 30 kg, any dimension above 100 cm, or a dimension sum above 200 cm; service-specific restrictions remain subject to the carrier response. Parcels are never split. Only configured services with both a valid price and transit result are offered. A seller with no available service prevents a complete quote.

Delivery days mean carrier transit time **from posting**, excluding seller preparation. Address autocomplete, labels, tracking, insurance add-ons, and multiple parcels are outside this release.

## Errors and rollout

- 400: invalid CEP, missing/invalid SKU measurements, unsupported parcel, no delivery option, invalid selection or settings. Responses explain the validation failure.
- 403: role or tenant access denied.
- 404: seller/quote not found, or quote belongs to another customer.
- 409: expired quote, changed inputs, or conflicting idempotency key.
- 503 with `shipping_unavailable`: temporary carrier failure, malformed response, unavailable encryption key, or carrier authentication failure. Retry transient outages; check seller credentials for repeated authentication failures.

Configure each seller in homologation, verify measurements and contracted services, and obtain a successful test quote before enabling that seller in production. Production credentials and quotes need their own smoke test. Leave unconfigured sellers on fixed mode. Switching mode invalidates outstanding quotes. Live homologation requires seller credentials and cannot be replaced by the mocked tests.

## Verification

```sh
cargo test --workspace --manifest-path kremlin/Cargo.toml
KREMLIN_TEST_DATABASE_URL=postgresql://USER:PASSWORD@HOST:PORT/shipping_test \
  cargo test -p business --manifest-path kremlin/Cargo.toml \
  --test shipping_postgres --test marketplace_postgres -- --ignored --nocapture
```

The database must be disposable and its name must end in `_test`. Tests create and remove isolated schemas. HTTP adapter tests use a local mock server and require loopback networking.

Protocol references: [Correios authentication](https://www.correios.com.br/atendimento/developers/manuais/manual-uso-da-api-token), [price API](https://www.correios.com.br/atendimento/developers/manuais/manual-api-preco-1), and [transit time API](https://www.correios.com.br/atendimento/developers/manuais/manual-api-prazo). The adapter uses their documented `/token/v1`, `/preco/v1`, and `/prazo/v1` routes.
