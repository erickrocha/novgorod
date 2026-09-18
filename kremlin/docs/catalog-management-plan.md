# Catalog management across tenants

## Summary

Extend catalog functionality end-to-end. The current API exposes only catalog GET endpoints, so add authenticated CRUD for categories, products, and SKUs, then add matching TailAdmin/Redux screens. Province and city remain read-only.

## Backend

- Tenant-scope catalog reads and writes for `SysAdmin`, `TenantOwner`, and `TenantUser`.
- Add `POST`/`PUT` endpoints for `/categories`, `/products`, and `/skus`.
- Add JSON contracts, mappers, gateways, use-case validation, OpenAPI entries, and routes.
- Require SysAdmin to select a tenant; force other roles to their authenticated tenant.
- Reject cross-tenant references and invalid required fields, slugs, codes, and prices.

## Frontend

- Add a catalog Redux slice/service and TailAdmin pages for categories, products, and SKUs.
- Add list/add/edit routes and sidebar navigation.
- Use searchable tenant selection for SysAdmin and fixed tenant context for other roles.
- Load attributes, attribute values, and product attributes as supporting read-only form data.

## Verification

- Test tenant isolation and cross-tenant reference rejection on the backend.
- Verify role-aware selectors/forms, successful refresh/navigation, loading, empty, and error states.
- Run Rust tests/checks and Veche TypeScript/build checks.
