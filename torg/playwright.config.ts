import { defineConfig } from '@playwright/test';

// Needs the kremlin test stack (infra/test) running with the kit fixtures and PagSeguro sandbox credentials
// on the tenants: see e2e/README.md. torg is served by vite on 5173 against VITE_API_URL (default :8080).
export default defineConfig({
  testDir: 'e2e',
  timeout: 90_000,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:5173',
    locale: 'pt-BR',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'npx vite --host 127.0.0.1 --port 5173',
    url: 'http://127.0.0.1:5173',
    reuseExistingServer: true,
  },
});
