# Tenant Product Catalog and Traceable Product Images

## Summary

Rewrite the unapplied catalog migrations into a tenant-isolated, normalized ER model. Support multiple product categories, reusable tenant attribute definitions, valid SKU combinations, single-location stock, and product/SKU image galleries backed by external object storage metadata. This change is schema-only; no upload API or S3 client is included.

## Schema changes

- Register every migration exactly once and in dependency order.
- Model tenant-owned categories, products, reusable attributes and values, product/category and product/attribute associations, SKUs, SKU attribute selections, and one stock balance per SKU.
- Enforce tenant boundaries through composite foreign keys, tenant-scoped slugs and SKU codes, one value per attribute per SKU, and a unique canonical variant key per product.
- Add checks for monetary, dimensional, inventory, and Brazilian fiscal fields.
- Use explicit foreign-key deletion behavior and retain standard UUID and audit columns.

## Product image metadata

- Add tenant/product ownership with an optional SKU reference constrained to the same tenant and product.
- Store presentation fields: alternative text, sort order, and primary-image marker.
- Store durable object identity: provider, bucket, immutable object key, optional version, ETag, and SHA-256 checksum.
- Store file metadata: original filename, MIME type, byte size, width, and height.
- Track storage lifecycle (`pending`, `available`, `delete_pending`, `deleted`, `failed`) and soft-deletion/audit fields.
- Never persist public or presigned URLs. A future storage adapter derives URLs from the durable object identity.
- Replacing an image creates a new row instead of repointing historical metadata.

## Validation

- Verify migration registration and ordering.
- Run a clean MySQL up/down/up cycle.
- Test tenant isolation, uniqueness, SKU option consistency, stock checks, image ownership, storage identity, and soft-deletion metadata.
- Run formatting, workspace compilation, and migration tests.

## Assumptions

- The catalog migrations have not been applied to a persistent database and may be rewritten directly.
- Catalogs and image metadata are tenant-owned.
- Products may belong to multiple categories; application logic will maintain one primary category.
- Inventory is one aggregate balance per SKU.
- Image bytes live in an S3-compatible service; MySQL stores only durable identity, metadata, lifecycle, and audit history.
- Application-layer catalog entities, endpoints, and storage adapters are deferred.
