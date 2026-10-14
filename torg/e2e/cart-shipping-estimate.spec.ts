import { expect, test, type Page } from '@playwright/test';

// Fixtures from infra/test/kit.py seed: product-pro-1 (T1, R$ 15,00, shipping SP R$ 15,00 / RJ R$ 0, 2 days),
// product-b1 (T2, R$ 30,00, shipping SP R$ 10,00, no RJ rate).
async function addToCart(page: Page, slug: string) {
  await page.goto(`/produto/${slug}`);
  await page.getByRole('button', { name: /Adicionar ao Carrinho/ }).click();
}

async function estimate(page: Page, cep: string) {
  await page.goto('/carrinho');
  await page.locator('#shipping-cep').fill(cep);
  await page.getByRole('button', { name: 'Calcular', exact: true }).click();
}

test('anonymous visitor sees the shipping price and delivery time in the cart', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  const request = page.waitForRequest(r => r.url().endsWith('/api/public/shipping/estimate'));
  await estimate(page, '01001-000');
  expect((await request).headers()['authorization']).toBeUndefined();
  await expect(page.getByText(/Entrega Padrão \(em até 2 dias\)/)).toBeVisible();
  await expect(page.getByText(/Frete calculado: Entrega Padrão - R\$ 15\.00/)).toBeVisible();
});

test('two sellers: the prices add up and the longest delivery time is shown', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await addToCart(page, 'product-b1');
  await estimate(page, '01001-000');
  await expect(page.getByText(/Frete de 2 vendedores \(prazo máximo: 2 dias\)/)).toBeVisible();
  await expect(page.getByText(/R\$ 25\.00/)).toBeVisible();
});

test('the backend message is shown when a seller does not deliver to the CEP', async ({ page }) => {
  await addToCart(page, 'product-b1');
  const response = page.waitForResponse(r => r.url().endsWith('/api/public/shipping/estimate'));
  await estimate(page, '20040-020');
  const body = await (await response).json();
  expect((await response).status()).toBe(400);
  await expect(page.getByText(body.message)).toBeVisible();
});

test('an invalid CEP shows an error and no price', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await estimate(page, '00500-000');
  await expect(page.getByText(/Frete calculado/)).toHaveCount(0);
  await expect(page.getByText(/(invalid CEP|CEP inválido)/i).first()).toBeVisible();
});

test('an unreachable backend shows an error and no price', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await page.route('**/api/public/shipping/estimate', route => route.abort());
  await estimate(page, '01001-000');
  await expect(page.getByText(/Frete calculado/)).toHaveCount(0);
  await expect(page.getByText(/Entrega Padrão \(em até/)).toHaveCount(0);
  await expect(page.getByText(/(Network Error|Falha ao calcular frete)/).first()).toBeVisible();
});

test('the public estimate answers 429 with Retry-After after 10 requests from one client', async ({ request }) => {
  const url = `${process.env.E2E_API_URL ?? 'http://127.0.0.1:8080'}/api/public/shipping/estimate`;
  const headers = { 'x-forwarded-for': `203.0.113.${Math.floor(Math.random() * 200) + 1}`, 'accept-language': 'pt-BR' };
  const data = { items: [{ skuId: 1, quantity: 1 }], cep: '01001-000' };
  for (let call = 1; call <= 10; call += 1) {
    expect((await request.post(url, { headers, data })).status(), `call ${call}`).toBe(200);
  }
  const limited = await request.post(url, { headers, data });
  expect(limited.status()).toBe(429);
  expect(Number(limited.headers()['retry-after'])).toBeGreaterThan(0);
  expect((await limited.json()).message).toMatch(/Muitas solicitações/);
  const other = await request.post(url, { headers: { ...headers, 'x-forwarded-for': '198.51.100.250' }, data });
  expect(other.status()).toBe(200);
});
