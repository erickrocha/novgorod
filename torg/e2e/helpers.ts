import { expect, type Page } from '@playwright/test';

export const CARD = { number: '4539620659922097', expiry: '12/28', cvv: '123', holder: 'PAGSEGURO TESTE' };
export const PSD = process.env.E2E_PSD_URL ?? 'http://127.0.0.1:8099';

export async function addToCart(page: Page, slug: string) {
  await page.goto(`/produto/${slug}`);
  await page.getByRole('button', { name: /Adicionar ao Carrinho/ }).click();
}

// Customer C1 of the infra/test kit seed (default saved address AD1, SP).
export async function loginAtCheckout(page: Page) {
  await page.goto('/checkout');
  await page.getByPlaceholder('seu@email.com ou 000.000.000-00').fill('c1@test.local');
  await page.getByPlaceholder('******').fill('Cust-test-1');
  await page.getByRole('button', { name: 'Entrar' }).click();
  await expect(page.getByText('Seus dados')).toBeVisible();
}

export async function reachPayment(page: Page) {
  await loginAtCheckout(page);
  await page.getByRole('button', { name: /Calcular total com frete/ }).click();
  await expect(page.locator('#pagseguro-card-number')).toBeVisible({ timeout: 20_000 });
}

export async function fillCard(page: Page) {
  await page.locator('#pagseguro-card-number').fill(CARD.number);
  await page.locator('#pagseguro-card-holder').fill(CARD.holder);
  await page.locator('#pagseguro-card-expiry').fill(CARD.expiry);
  await page.locator('#pagseguro-card-cvv').fill(CARD.cvv);
}

// Deterministic stand-in for the PagSeguro SDK (the real one is used by the sandbox spec).
export const STUB_SDK = `window.PagSeguro = { encryptCard: function () { return { encryptedCard: window.__ENCRYPTED || 'approved-card' }; } };`;

export async function stubSdk(page: Page, encrypted = 'approved-card') {
  await page.addInitScript(value => { (window as unknown as { __ENCRYPTED: string }).__ENCRYPTED = value; }, encrypted);
  await page.route('**/checkout-sdk-js/**/pagseguro.min.js', route =>
    route.fulfill({ contentType: 'application/javascript', body: STUB_SDK }));
}

export async function psd(path: string, init?: RequestInit) {
  return fetch(`${PSD}${path}`, init);
}
