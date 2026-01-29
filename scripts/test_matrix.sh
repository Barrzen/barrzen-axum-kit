#!/bin/bash
set -e

echo "Testing barrzen-axum-kit feature matrix..."

# Ensure we are in the kit workspace root
cd "$(dirname "$0")/.."
echo "Working directory: $(pwd)"

# 1. No features (minimal)
echo "------------------------------------------------"
echo "Testing: Minimal Core"
cargo test -p barrzen-axum-core --no-default-features

# 2. Core + Moka Cache
echo "------------------------------------------------"
echo "Testing: Core + Moka"
cargo test -p barrzen-axum-infra --features cache-moka

# 3. DB + Moka
echo "------------------------------------------------"
echo "Testing: DB + Moka"
cargo check -p barrzen-axum-infra --features "db,cache-moka"

# 4. Search + Broker
echo "------------------------------------------------"
echo "Testing: Meilisearch + NATS"
cargo check -p barrzen-axum-infra --features "meilisearch,nats"

# 5. All Features
echo "------------------------------------------------"
echo "Testing: All Features"
cargo test --workspace --all-features

echo "------------------------------------------------"
echo "Feature matrix tests passed!"
