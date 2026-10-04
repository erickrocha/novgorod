#!/usr/bin/env bash
# Run the kremlin test suite (unit + Postgres integration) against a disposable PostgreSQL.
#   ./run.sh                      full suite: cargo test --workspace -- --include-ignored
#   ./run.sh clippy               cargo clippy --workspace --all-targets
#   ./run.sh cargo test -p business --test cart_cleanup_postgres -- --ignored --nocapture
set -euo pipefail
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- cargo test --workspace -- --include-ignored; fi
if [ "$1" = clippy ]; then set -- cargo clippy --workspace --all-targets; fi
trap 'docker compose down -v' EXIT
docker compose run --rm --build kremlin-tests "$@"
