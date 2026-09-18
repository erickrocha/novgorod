# Wine Catalog Sample Data and Editable CSV/XLSX Import

## Summary

Create a complete wine ecommerce sample catalog with at least 50 products and all relational catalog records, generate CSV and XLSX files, and add a tenant-aware import workflow modeled after SocialFit's editable CSV preview.

## Dataset

- Generate 60 wine products with categories, attributes, values, product links, SKUs, SKU attribute values, and stock records.
- Generate `data/wines/wine_catalog.csv` as a flat SKU/product import format.
- Generate `data/wines/wine_catalog.xlsx` with normalized sheets: Categories, Attributes, AttributeValues, Products, ProductCategories, ProductAttributes, SKUs, SkuAttributeValues, and SkuStock.
- Include a reproducible `scripts/generate_wine_catalog.mjs` builder and stable tenant-scoped import keys.

## Import API

- Add `POST /catalog/import` accepting multipart `file`, `tenantId`, and optional edited-row JSON.
- Accept CSV and XLSX, normalize both formats, validate every entity and relationship, and commit atomically.
- SysAdmin selects the target tenant; other roles are restricted to their authenticated tenant.
- Upsert by stable external keys and return created, updated, unchanged, and validation-error details.

## Frontend

- Add CSV/XLSX parsing, full editable preview, cell validation, tenant selection, import submission, and result/error views.
- Follow the SocialFit CsvImportModal interaction pattern while using this repository's React/Redux architecture.

## Verification

- Validate generated counts, workbook sheet headers, CSV/XLSX equivalence, stable-key relationships, tenant isolation, atomic rollback, repeat imports, and frontend preview edits.
