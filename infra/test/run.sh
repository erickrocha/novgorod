#!/usr/bin/env bash
# Run the kremlin test suite (unit + Postgres integration) against a disposable PostgreSQL.
#   ./run.sh                      full suite, excluding the live PagSeguro charge
#   ./run.sh pagseguro             explicit PagSeguro sandbox charge + status test
#   ./run.sh mercadopago           explicit Mercado Pago sandbox charge + status test
#   ./run.sh clippy               cargo clippy --workspace --all-targets
#   ./run.sh cargo test -p business --test cart_cleanup_postgres -- --ignored --nocapture
set -euo pipefail
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then
    set -- cargo test --workspace -- --include-ignored --skip live_sandbox_pagseguro_charge_and_status --skip live_sandbox_mercado_pago_charge_and_status --test-threads=1
elif [ "$1" = pagseguro ]; then
    set -- cargo test -p application live_sandbox_pagseguro_charge_and_status -- --ignored --nocapture
elif [ "$1" = mercadopago ]; then
    set -- cargo test -p application live_sandbox_mercado_pago_charge_and_status -- --ignored --nocapture
elif [ "$1" = clippy ]; then
    set -- cargo clippy --workspace --all-targets
fi
trap 'docker compose down -v' EXIT
docker compose run --rm --build kremlin-tests "$@"
