import { expect, test } from '@playwright/test';
import { addToCart, CARD, fillCard, reachPayment } from './helpers';

// Live PagSeguro sandbox with the real SDK from assets.pagseguro.com.br. Needs the stack started with
// TEST_PAGSEGURO_API_BASE_URL=https://sandbox.api.pagseguro.com and `infra/test/pagseguro_tenants.py` after the seed.
test.skip(!process.env.E2E_PAGSEGURO_SANDBOX, 'set E2E_PAGSEGURO_SANDBOX=1 (live sandbox)');

test('real SDK encrypts the card and the sandbox captures the payment', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await reachPayment(page);
  // the installment options are the live PagBank's own: interest-free only, the first is the single payment
  const options = page.locator('#pagseguro-installments option');
  await expect(options.first()).toHaveText('1x de R$ 30,00 sem juros');
  expect(await options.count()).toBeGreaterThanOrEqual(1);
  await expect(page.getByText(/Apenas à vista está disponível/)).toHaveCount(0);
  const submit = page.waitForRequest(r => /\/purchases\/\d+\/payments\/submit$/.test(r.url()));
  await fillCard(page);
  // the sandbox does not know the BIN of its own test card: kremlin answers without the BIN
  await expect(page.getByText(/Apenas à vista está disponível/)).toHaveCount(0);
  await expect(options.first()).toHaveText('1x de R$ 30,00 sem juros');
  await page.locator('#pagseguro-submit').click();
  const body = (await submit).postData() ?? '';
  expect(body).not.toContain(CARD.number);
  expect(body).not.toContain(CARD.cvv);
  expect(JSON.parse(body).token.length).toBeGreaterThan(64);
  await expect(page.getByText('Pagamento confirmado').or(page.getByText('Pagamento em processamento'))).toBeVisible({ timeout: 60_000 });
});
