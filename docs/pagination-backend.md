# Typed backend pagination endpoints for all exposed entities

## Summary

Add separate typed paginated endpoints for every existing list resource while preserving the current array endpoints for compatibility. The frontend TanStack grid will migrate to the new paged routes incrementally.

Covered resources:

- Users
- Tenants
- Provinces
- Cities
- Categories
- Catalog attributes
- Catalog attribute values
- Products
- Product attributes
- SKUs

Internal join/support entities without current list APIs will remain out of scope.

## API contract

Each new endpoint returns:

{                                                                                                                                                                                                                                 
"items": [],                                                                                                                                                                                                                    
"total": 0,                                                                                                                                                                                                                     
"page": 1,                                                                                                                                                                                                                      
"pageSize": 25                                                                                                                                                                                                                  
}

Shared query parameters:

page=1                                                                                                                                                                                                                            
pageSize=25                                                                                                                                                                                                                       
q=                                                                                                                                                                                                                                
sortBy=                                                                                                                                                                                                                           
sortDir=asc|desc

Rules:

- page is 1-based.
- pageSize is restricted to safe values, with a maximum enforced by the backend.
- Default page size is 25.
- sortBy accepts only an endpoint-specific allowlist.
- sortDir accepts only asc or desc.
- q performs the endpoint’s supported global text search.
- Endpoint-specific filters remain typed, such as tenantId, active, role, enabled, countryCode, and provinceId.

New route pattern:

/user/paged                                                                                                                                                                                                                       
/tenant/paged                                                                                                                                                                                                                     
/province/paged                                                                                                                                                                                                                   
/cities/paged                                                                                                                                                                                                                     
/categories/paged                                                                                                                                                                                                                 
/catalog-attributes/paged                                                                                                                                                                                                         
/catalog-attribute-values/paged                                                                                                                                                                                                   
/products/paged                                                                                                                                                                                                                   
/product-attributes/paged                                                                                                                                                                                                         
/skus/paged

Existing array routes remain unchanged.

## Backend implementation

- Add shared pagination/query structs in the application layer.
- Add a shared paged response type for OpenAPI and JSON serialization.
- Add typed query structs and sortable-field allowlists per endpoint.
- Extend gateways/use cases with paged database queries:
    - filtered count query;
    - deterministic sorted query;
    - offset/limit query;
    - mapped typed response rows.

- Preserve all existing authorization and tenant isolation rules:
    - SysAdmin visibility;
    - TenantOwner tenant scoping;
    - province/city access rules;
    - catalog tenant filtering.

- Add the new routes to the relevant Axum route builders.
- Register all new endpoints and paged response schemas in OpenAPI.
- Keep existing array endpoints for legacy callers and non-grid consumers.

## Frontend migration

- Update each service method to call its /paged endpoint.
- Update Redux state from plain arrays to:
    - items;
    - total;
    - page;
    - pageSize;
    - loading/error state.

- Configure TanStack Table with manual pagination, sorting, and filtering.
- Persist grid state in URL parameters:
    - page;
    - pageSize;
    - q;
    - sortBy;
    - sortDir;
    - typed endpoint filters.

- Remove client-side pagination for migrated list screens.
- Keep editable import preview tables local because they represent unsaved CSV rows, not backend list data.

## Tests and acceptance criteria

Backend tests:

- Default query behavior returns page 1 with 25 rows.
- Page boundaries and empty pages behave correctly.
- Total count reflects filters but not pagination.
- Search and endpoint-specific filters are applied before counting.
- Allowed sorting works; unknown sort fields are rejected or safely normalized.
- Ascending and descending ordering are deterministic.
- Tenant and role authorization cannot be bypassed through query parameters.
- Every new paged endpoint returns the common response shape.
- Existing array endpoints remain behaviorally compatible.

Frontend tests:

- Grid requests only the selected page.
- Changing page/page size/search/sort updates the URL and sends the expected query.
- Response total controls page count.
- Loading and empty states work when a page has no rows.
- Navigating between list screens does not reuse stale pagination state.
- Existing import preview editing and Save-only upload behavior remains unchanged.

Verification:

- Rust formatter, checks, unit/integration tests.
- Frontend TypeScript build and targeted lint.
- OpenAPI generation/inspection.
- Manual verification of all ten paged resources and URL back/forward navigation.

## Assumptions

- “All entities” means all resources already exposed as list endpoints, not internal join/support tables.
- New /paged routes are preferred over changing existing array response shapes.
- The common query and response contract is shared, but filtering and sortable fields remain entity-specific.
- The current TanStack grid component will be changed from client-side pagination to manualPagination, manualSorting, and manualFiltering once the endpoints are available.   