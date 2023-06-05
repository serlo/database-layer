#!/bin/bash

source scripts/utils.sh

function init() {
  set -e
  trap 'tear_down' EXIT

  setup_mysql
  setup_server
}

function main() {
  if [ -n "$2" ]; then
    DATA="{ \"type\": \"$1\", \"payload\": $2 }"
  else
    DATA="{ \"type\": \"$1\" }"
  fi

  log "INFO: Fetch with data $(echo "$DATA" | jq)"

  RESULT=$(mktemp -t serlo-database-layer-fetch.XXXXXXXXXX)

  # See https://stackoverflow.com/a/22508743
  STATUS=$(curl --header "Content-Type: application/json" \
    --data "$DATA" \
    --verbose \
    -o "$RESULT" -w "%{http_code}" \
    http://localhost:8080/)

  if [ $STATUS -eq 200 ]; then
    jq < "$RESULT"
  else
    cat "$RESULT"
    echo
  fi

  rm "$RESULT"
}

function tear_down() {
  kill_cargo_background_process
}

init
main "$@"
