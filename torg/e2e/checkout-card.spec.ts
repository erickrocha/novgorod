import { expect, test } from '@playwright/test';
import { addToCart, CARD, fillCard, psd, reachPayment, stubSdk } from './helpers';

// Runs against the kremlin test stack with the psd PagSeguro double (infra/test: `docker compose up -d --wait kremlin psd`,
// `./kit.py seed`). The PagSeguro SDK is replaced by a stub so the outcome is deterministic.
test.describe('card payment through the screens (provider double)', () => {
  test.beforeEach(async () => { await psd('/_psd/calls', { method: 'DELETE' }); });

  test('approved: only the SDK token reaches kremlin and the payment is confirmed', async ({ page }) => {
    await stubSdk(page, 'approved-card');
    await addToCart(page, 'product-pro-1');
    await reachPayment(page);
    const submit = page.waitForRequest(r => /\/purchases\/\d+\/payments\/submit$/.test(r.url()));
    await fillCard(page);
    await page.locator('#pagseguro-submit').click();
    const body = (await submit).postData() ?? '';
    expect(JSON.parse(body).token).toBe('approved-card');
    expect(body).not.toContain(CARD.number);
    expect(body).not.toContain(CARD.cvv);
    await expect(page.getByText('Pagamento confirmado')).toBeVisible();
    await expect(page.getByText(/Compra #\d+ confirmada/)).toBeVisible();
  });

  test('declined: the recovery message is shown and the cart is kept', async ({ page }) => {
    await stubSdk(page, 'reject-card');
    await addToCart(page, 'product-pro-1');
    await reachPayment(page);
    await fillCard(page);
    await page.locator('#pagseguro-submit').click();
    await expect(page.getByText(/Pagamento recusado/)).toBeVisible();
    await expect(page.locator('#pagseguro-card-number')).toBeVisible();
    await page.goto('/carrinho');
    await expect(page.getByText('Product PRO-1').first()).toBeVisible();
  });

  test('the card form stays disabled and sends nothing when the PagSeguro SDK is blocked', async ({ page }) => {
    await page.route('**/checkout-sdk-js/**/pagseguro.min.js', route => route.abort());
    const requests: string[] = [];
    page.on('request', r => { if (r.url().includes('/payments/submit')) requests.push(r.url()); });
    await addToCart(page, 'product-pro-1');
    await reachPayment(page);
    await expect(page.getByText(/ambiente seguro de pagamento/)).toBeVisible();
    await fillCard(page);
    await expect(page.locator('#pagseguro-submit')).toBeDisabled();
    await page.locator('form#form-pagseguro').evaluate(form => (form as HTMLFormElement).requestSubmit());
    await page.waitForTimeout(500);
    expect(requests).toEqual([]);
  });

  test('without payment settings the checkout shows an error instead of waiting forever', async ({ page }) => {
    await stubSdk(page);
    await addToCart(page, 'product-pro-1');
    await page.route('**/checkout/payment-config**', route =>
      route.fulfill({ status: 503, contentType: 'application/json', body: JSON.stringify({ message: 'Pagamento indisponível. Verifique a configuração do meio de pagamento.' }) }));
    await page.goto('/checkout');
    await page.getByPlaceholder('seu@email.com ou 000.000.000-00').fill('c1@test.local');
    await page.getByPlaceholder('******').fill('Cust-test-1');
    await page.getByRole('button', { name: 'Entrar' }).click();
    await page.getByRole('button', { name: /Calcular total com frete/ }).click();
    await expect(page.getByText(/Pagamento indisponível/)).toBeVisible();
  });
});
