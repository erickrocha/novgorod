# Test stack (kremlin)

Disposable PostgreSQL plus a Rust container that runs the kremlin tests. Same Postgres image as `../dev`, but nothing persists (tmpfs) and it listens on port 5433, so it never touches the dev database.

```
./run.sh                      # cargo test --workspace -- --include-ignored
./run.sh clippy               # cargo clippy --workspace --all-targets
./run.sh cargo test -p business --test cart_cleanup_postgres -- --ignored --nocapture
```

Requires Docker with the compose plugin. The first run downloads the images and compiles the workspace; the cargo registry and build cache live in named volumes, so later runs are fast. `run.sh` tears the stack down (`docker compose down -v`) when it exits.

- The Postgres tests are `#[ignore]d`, read `KREMLIN_TEST_DATABASE_URL`, and refuse a database whose name does not end in `_test`. The compose file sets both. Each test creates its own schema and runs the migrations in it.
- Credentials are throwaway defaults for a local, non-persistent database. Override with `TEST_DATABASE_NAME`, `TEST_DATABASE_USER`, `TEST_DATABASE_PASSWORD`, `TEST_DATABASE_PORT`.
- The source is mounted from `../../kremlin`; build output goes to the `cargo_target` volume, not into the repo.
- Used by NOV-3 plan tasks T-CHK-01..03 (test kit). Not wired into CI.
