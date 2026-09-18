# Province and City Administration

## Summary

Extend the existing read-only province/city functionality into a SysAdmin-only management area with add/edit/list operations and CSV imports. Generate checked-in Brazilian location datasets using IBGE identifiers, while preserving the existing lookup endpoints used by tenant forms.

## Backend changes

- Add persisted IBGE identifiers for provinces and cities with unique indexes.
- Extend domain models, entities, gateways, use cases, JSON schemas, mappers, OpenAPI, and routes.
- Add authenticated province/city add and edit endpoints, while retaining existing GET lookup endpoints.
- Add separate multipart CSV import endpoints for provinces and cities.
- Require `SysAdmin` for add/edit/import operations.
- Implement upsert-plus-report imports keyed by IBGE code; valid rows are committed and invalid rows are reported.
- Do not implement deletion; the agreed scope is list/add/edit/import.

## Dataset and CSV contract

- Generate data from the official IBGE locality dataset/API.
- Check in `data/brazil/provinces.csv` and `data/brazil/cities.csv`.
- Province columns: `ibge_code,acronym,name,country_code`.
- City columns: `ibge_code,province_ibge_code,name`.

## Frontend changes

- Add province/city services, types, Redux state/thunks, and import result handling.
- Add SysAdmin-only routes `/system-settings/provinces` and `/system-settings/cities`.
- Add a SysAdmin-only System Settings sidebar group.
- Add province and city list/search/add/edit/import screens with loading, error, and import-summary states.
- Add English and Portuguese translations for the new management UI.

## Verification

- Test authorization, validation, upsert identity, CSV errors, unknown province references, and import summaries.
- Run Rust checks/tests, frontend lint/build, dataset validation, and `git diff --check`.

## Assumptions

- Official IBGE data is the source of truth.
- Existing GET endpoint behavior remains backward-compatible.
- Imports allow partial success.
- No delete endpoints or controls are added.
