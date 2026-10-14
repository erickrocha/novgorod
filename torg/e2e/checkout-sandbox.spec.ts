import { expect, test } from '@playwright/test';
import { addToCart, CARD, fillCard, reachPayment } from './helpers';

// Live PagSeguro sandbox with the real SDK from assets.pagseguro.com.br. Needs the stack started with
// TEST_PAGSEGURO_API_BASE_URL=https://sandbox.api.pagseguro.com and `infra/test/pagseguro_tenants.py` after the seed.
test.skip(!process.env.E2E_PAGSEGURO_SANDBOX, 'set E2E_PAGSEGURO_SANDBOX=1 (live sandbox)');

test('real SDK encrypts the card and the sandbox captures the payment', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await reachPayment(page);
  const submit = page.waitForRequest(r => /\/purchases\/\d+\/payments\/submit$/.test(r.url()));
  await fillCard(page);
  await page.locator('#pagseguro-submit').click();
  const body = (await submit).postData() ?? '';
  expect(body).not.toContain(CARD.number);
  expect(body).not.toContain(CARD.cvv);
  expect(JSON.parse(body).token.length).toBeGreaterThan(64);
  await expect(page.getByText('Pagamento confirmado').or(page.getByText('Pagamento em processamento'))).toBeVisible({ timeout: 60_000 });
});
