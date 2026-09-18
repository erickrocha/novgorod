# TanStack Table data-grid migration

## Summary

Add a shared Tailwind-compatible TanStack Table component and migrate all management list screens to it:

- Users
- Tenants
- Provinces
- Cities
- Catalog categories, products, and SKUs
- Editable province/catalog import previews

Use server-side pagination, filtering, and sorting with URL-persisted grid state.

## Frontend implementation

- Add `@tanstack/react-table`.
- Create a reusable `DataGrid` component with typed columns, manual server-side pagination/sorting/filtering, debounced global search, loading/empty/error states, pagination, page-size options `10 / 25 / 50 / 100` (default `25`), column visibility, responsive Tailwind styling, dark mode, and optional editable cells.
- Use this common result type:

  ```ts
  interface PagedResult<T> {
    items: T[];
    total: number;
    page: number;
    pageSize: number;
  }
  ```

- Persist `page`, `pageSize`, `q`, `sortBy`, `sortDir`, and endpoint-specific filters in URL parameters.
- Update Redux slices/services to request and store paged results while preserving authorization and tenant visibility rules.

## Backend API changes

Extend list endpoints to accept `page`, `pageSize`, `q`, `sortBy`, and `sortDir`, returning:

```json
{ "items": [], "total": 0, "page": 1, "pageSize": 25 }
```

Support typed filters where applicable: user role/enabled/tenantId, tenant status/location, province countryCode, city provinceId, and catalog tenantId/active/relationships. Validate bounds and whitelist sortable fields.

Endpoints: `/user`, `/tenant`, `/province`, `/cities`, `/categories`, `/products`, `/skus`.

## Import previews

Use the shared grid in editable mode for province and catalog import previews. Keep local parsing, validation, and Save-only upload behavior.

## Test plan

- Verify grid rendering, loading/empty/error states, sorting, debounced search, pagination, page-size changes, URL state, column visibility, and editable import cells.
- Add backend coverage for query defaults, safe sort fields, search/filter/pagination totals, authorization, tenant isolation, and every paged endpoint.
- Run frontend build/lint and Rust format/check/tests.

## Assumptions

- Server-side mode is used for every management list.
- `q` is the common global search parameter; structured filters are endpoint-specific.
- Existing CRUD, import, authorization, and translation behavior remains unchanged apart from grid query integration.
