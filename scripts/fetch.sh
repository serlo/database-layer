#!/bin/bash

source scripts/utils.sh

function init() {
  set -e
  trap 'tear_down' EXIT

  setup_mysql
  setup_server
}

function main() {
  DATA="{ \"type\": \"$1\", \"payload\": $2 }"

  log "INFO: Fetch with data $(echo "$DATA" | jq)"

  time curl --header "Content-Type: application/json" \
    --data "$DATA" \
    --verbose \
    http://localhost:8080/ | jq
}

function tear_down() {
  kill_cargo_background_process
}

init
main "$@"
