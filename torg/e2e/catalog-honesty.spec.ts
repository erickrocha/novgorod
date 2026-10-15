import { expect, test } from '@playwright/test';
import { addToCart } from './helpers';

// Fixtures from infra/test/kit.py seed: products have no rating, review count or image in the API.

test('the catalog shows no invented rating, review count, stock photo or category', async ({ page }) => {
  const stockPhotos: string[] = [];
  page.on('request', r => { if (r.url().includes('images.unsplash.com')) stockPhotos.push(r.url()); });
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Product B1', exact: true })).toBeVisible();
  const text = await page.locator('main').innerText();
  expect(text).not.toMatch(/\b4[.,]8\b/);
  expect(text).not.toMatch(/\(12\)|\(24\)/);
  expect(text).not.toContain('Vinhos'); // main only: the footer has its own text
  await expect(page.getByRole('img', { name: /sem imagem/ }).first()).toBeVisible();
  expect(stockPhotos).toEqual([]);
});

test('the product page shows no invented rating and the real stock of the SKU', async ({ page }) => {
  await page.goto('/produto/product-pro-1');
  await expect(page.getByRole('heading', { name: 'Product PRO-1' })).toBeVisible();
  const text = await page.locator('main').innerText();
  expect(text).not.toMatch(/\/ 5\.0|avaliações/);
  await expect(page.getByText(/Em Estoque \(\d+ garrafas disponíveis\)/)).toBeVisible();
  // a product without category: no category label and no category entry in the breadcrumb
  await expect(page.getByRole('navigation', { name: 'Breadcrumb' }).getByRole('listitem')).toHaveText(['Catálogo', 'Product PRO-1']);
  // the seller card shows the real seller and no claim the data does not hold
  await expect(page.getByRole('heading', { name: 'Seller T1' })).toBeVisible();
  expect(text).not.toMatch(/Vendedor Verificado|Envio direto da vinícola|Novgorod Vinhos/);
  await expect(page.getByRole('img', { name: /sem imagem/ }).first()).toBeVisible();
});

test('an unknown product says not found and does not show another product', async ({ page }) => {
  await page.goto('/produto/nao-existe-xyz');
  await expect(page.getByText('Produto não encontrado')).toBeVisible();
  await expect(page.getByRole('heading', { name: /Product/ })).toHaveCount(0);
});

test('an unreachable backend is an error with a retry, not "not found"', async ({ page }) => {
  await page.route('**/api/public/products/product-pro-1', route => route.abort());
  await page.goto('/produto/product-pro-1');
  await expect(page.getByRole('alert').filter({ hasText: 'Não foi possível carregar o produto' })).toBeVisible();
  await expect(page.getByText('Produto não encontrado')).toHaveCount(0);
  await page.unroute('**/api/public/products/product-pro-1');
  await page.getByRole('button', { name: 'Tentar novamente' }).click();
  await expect(page.getByRole('heading', { name: 'Product PRO-1' })).toBeVisible();
});

test('the failure of the related products is shown, not an empty list', async ({ page }) => {
  await page.route('**/api/public/products/query**', route => route.abort());
  await page.goto('/produto/product-pro-1');
  await expect(page.getByRole('heading', { name: 'Product PRO-1' })).toBeVisible();
  await expect(page.getByText(/Não foi possível carregar as sugestões/)).toBeVisible();
});

test('the cart has no demo coupon: coupons are applied at checkout, per seller', async ({ page }) => {
  await addToCart(page, 'product-pro-1');
  await page.goto('/carrinho');
  await expect(page.getByText(/Cupons de desconto são aplicados na finalização da compra, por vendedor/)).toBeVisible();
  await expect(page.locator('#coupon-code')).toHaveCount(0);
  await expect(page.getByText('Desconto:', { exact: true })).toHaveCount(0);
  const text = await page.locator('main').innerText();
  expect(text).not.toMatch(/TORG10|NOVGOROD50/);
  const couponCalls: string[] = [];
  page.on('request', r => { if (r.url().includes('/marketing/coupons/validate')) couponCalls.push(r.url()); });
  await page.reload();
  expect(couponCalls).toEqual([]);
});

test('the quick view of a product without category shows no category label, rating or invented image', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Product B1', exact: true })).toBeVisible();
  await page.getByTitle('Visualização Rápida').first().click({ force: true });
  const dialog = page.locator('div.fixed').filter({ has: page.getByRole('button', { name: 'Fechar' }) }).first();
  await expect(dialog).toBeVisible();
  const text = await dialog.innerText();
  expect(text).not.toMatch(/\b4[.,]8\b|avaliações|Vinhos/);
  await expect(dialog.getByRole('img', { name: /sem imagem/ })).toBeVisible();
});

