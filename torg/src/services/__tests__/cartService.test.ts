import { beforeEach, describe, expect, it, vi } from 'vitest';
import apiClient from '../../api/client';
import cartService from '../cartService';

vi.mock('../../api/client');

const option = (serviceName: string, priceCents: number, transitDays: number | null) => ({ serviceName, priceCents, transitDays });

describe('cartService.estimateShipping', () => {
  beforeEach(() => vi.mocked(apiClient.post).mockReset());

  it('names the service for one seller', async () => {
    vi.mocked(apiClient.post).mockResolvedValue({ data: { sellers: [{ tenantId: 1, options: [option('Entrega Padrão', 1500, 2)] }] } });
    expect(await cartService.estimateShipping('01001000', [{ skuId: 1, quantity: 1 }]))
      .toEqual({ cost: 15, deliveryDays: 2, service: 'Entrega Padrão', sellerCount: 1 });
  });

  it('adds the cheapest option of each seller and keeps the longest delivery time', async () => {
    vi.mocked(apiClient.post).mockResolvedValue({ data: { sellers: [
      { tenantId: 1, options: [option('Expresso', 3000, 1), option('Padrão', 1500, 2)] },
      { tenantId: 2, options: [option('Padrão', 1000, 5)] },
    ] } });
    expect(await cartService.estimateShipping('01001000', [{ skuId: 1, quantity: 1 }, { skuId: 10, quantity: 2 }]))
      .toEqual({ cost: 25, deliveryDays: 5, service: 'Frete de 2 vendedores', sellerCount: 2 });
  });

  it('fails when a seller has no option', async () => {
    vi.mocked(apiClient.post).mockResolvedValue({ data: { sellers: [{ tenantId: 1, options: [] }] } });
    await expect(cartService.estimateShipping('01001000', [{ skuId: 1, quantity: 1 }])).rejects.toThrow('Frete indisponível');
  });
});
