#!/bin/bash

source scripts/utils.sh

print_header "Checking the Pacts"
PACT_FILE=../api.serlo.org/pacts/api.serlo.org-serlo.org-database-layer.json ./scripts/pacts.sh

print_header "Test it manually!"
cd ../api.serlo.org
yarn start &
firefox localhost:3001/___graphql

print_header "Is it all right?(y/n)"
read -r all_right

if [ "$all_right" == 'y' ]; then
  print_header "Time to merge 🚀️"
  gh pr merge
  exit
fi

print_header "Aborting..."
