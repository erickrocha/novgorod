#!/usr/bin/env python3
"""Puts the PagSeguro sandbox credentials of infra/test/.env (PAGSEGURO_TOKEN, PAGSEGURO_PUBLIC_KEY, PAGSEGURO_ENVIRONMENT)
on the seeded tenants, for runs against the live sandbox (stack started with TEST_PAGSEGURO_API_BASE_URL=https://sandbox.api.pagseguro.com).
Secret values are never printed."""
import os, sys, kit

env = {}
for line in open(os.path.join(kit.HERE, ".env")):
    line = line.strip()
    if line and not line.startswith("#") and "=" in line:
        key, value = line.split("=", 1)
        env[key] = value.strip().strip('"').strip("'")
missing = [k for k in ("PAGSEGURO_TOKEN", "PAGSEGURO_PUBLIC_KEY") if not env.get(k)]
if missing:
    sys.exit(f"missing in .env: {', '.join(missing)}")
body = {"provider": "pagseguro", "credentials": {"pagseguro": {
    "token": env["PAGSEGURO_TOKEN"], "publicKey": env["PAGSEGURO_PUBLIC_KEY"], "environment": env.get("PAGSEGURO_ENVIRONMENT") or "sandbox"}}}
for name, tenant_id in kit.fixtures()["tenants"].items():
    code, resp, _ = kit.api("PUT", f"/tenants/{tenant_id}/payment-settings", "sysadmin", body)
    print(f"{name} (#{tenant_id}): HTTP {code}")
    if code != 204:
        sys.exit(f"failed: {resp}")
