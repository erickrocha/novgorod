# torg end-to-end tests (Playwright)

Drive the real screens in headless Chromium against a running kremlin test stack. Needs Docker.

```
cd ../infra/test
docker compose up -d --build --wait kremlin psd     # kremlin :8080 on a disposable PostgreSQL, PagSeguro double :8099
./kit.py seed                                       # sellers, customers C1..C3, products, shipping rates, PagSeguro settings
cd ../../torg && npm run e2e                        # starts vite on :5173 when it is not running
docker compose -f ../infra/test/docker-compose.yml down -v
```

- `cart-shipping-estimate.spec.ts`: anonymous shipping estimate in the cart (price, delivery time, backend errors).
- `checkout-card.spec.ts`: card payment with a stubbed PagSeguro SDK against the `psd` double (approved, declined, SDK blocked, missing configuration).
- `checkout-sandbox.spec.ts`: live PagSeguro sandbox with the real SDK. Start the stack with `TEST_PAGSEGURO_API_BASE_URL=https://sandbox.api.pagseguro.com`, run `../infra/test/pagseguro_tenants.py` after the seed, then `E2E_PAGSEGURO_SANDBOX=1 npm run e2e -- checkout-sandbox`. Needs internet.

Run `./kit.py reset` between full runs if stock or carts are left over.
