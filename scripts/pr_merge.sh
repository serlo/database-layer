#!/bin/bash

source scripts/utils.sh

set -e

print_header "Checking the Pacts"
#PACT_FILE=../api.serlo.org/pacts/api.serlo.org-serlo.org-database-layer.json ./scripts/pacts.sh

print_header "Test it manually!"
cd ../api.serlo.org
yarn start &
firefox localhost:3001/___graphql
cd -

print_header "Is it alright?(y/n)"
read -r is_alright

if [ "$is_alright" != 'y' ]; then
  print_header "Aborting..."
  exit
fi

print_header "Do you want to make a new version?(y/n)"
read -r make_new_version

if [ "$make_new_version" == 'y' ]; then
  yarn update-version
fi

print_header "Time to merge 🚀️"
gh pr merge
