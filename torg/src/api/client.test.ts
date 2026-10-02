import { describe, expect, it, vi } from 'vitest';

describe('apiClient tenant header (SR-TEN-020)', () => {
  it('removes the stale tenant id and never sends X-Tenant-Id', async () => {
    localStorage.setItem('novgorod_tenant_id', '7');
    vi.resetModules();
    const { apiClient } = await import('./client');

    expect(localStorage.getItem('novgorod_tenant_id')).toBeNull();

    // run the request interceptor chain on a config
    const handlers = (apiClient.interceptors.request as unknown as {
      handlers: Array<{ fulfilled: (c: { headers: Record<string, string> }) => { headers: Record<string, string> } }>;
    }).handlers;
    let config = { headers: {} as Record<string, string> };
    for (const h of handlers) config = h.fulfilled(config);
    expect(config.headers['X-Tenant-Id']).toBeUndefined();
  });
});
