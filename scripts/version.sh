#!/bin/sh

source scripts/utils.sh

nvim server/Cargo.toml
cargo update -p server

nvim scripts/changelog.ts
print_header "Generating CHANGELOG"
yarn ts-node scripts/changelog.ts

git add -p