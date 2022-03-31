#!/bin/sh

source scripts/utils.sh

nvim scripts/changelog.ts

print_header "Generating CHANGELOG"
yarn ts-node scripts/changelog.ts

print_header "Updating version in Cargo.toml"
VERSION=$(cat scripts/changelog.ts \
  | grep 'name:' \
  | tail -1 \
  | awk -F: '{ print $2 }' \
  | sed "s/[', ]//g")

sed -i "0,/version/{s/version.*$/version = \"$VERSION\"/g}" server/Cargo.toml
cargo update -p server

print_header "Formatting"
yarn format

print_header "Time to commit"
git add -p
