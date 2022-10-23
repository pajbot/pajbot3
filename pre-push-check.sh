#!/bin/sh

set -e

# If changing any of these commands, make the same changes to the workflows in .github/workflows!
cargo fmt
cargo check --all-targets --all-features
cargo test
cargo clippy --all-targets --all-features -- -D warnings

cd web
npm i
npx --no-install prettier --write .
npm run build

echo "All ok!"
