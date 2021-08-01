#!/bin/bash

source scripts/utils.sh

function init() {
  set -e
  trap 'tear_down' EXIT

  setup_mysql
  setup_server
}

function main() {
  curl --request POST \
    --header "Content-Type: application/json" \
    --data "{ \"type\": \"$1\", \"payload\": $2 }" \
    --verbose \
    http://localhost:8080/ | jq
}

function tear_down() {
  kill_cargo_background_process
}

init
main "$@"
