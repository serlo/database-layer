#!/bin/bash
#
# Checks the compatbility of the Metadata API against the AMB specification

source scripts/utils.sh

TMP_DIR=$(mktemp -tdu serlo-database-layer-check-metadata.XXXXXXXXXX)
ZIP_FILE="$TMP_DIR/amb.zip"
SCHEMAS_PATH="amb-main/draft/schemas/"
SCHEMAS_DIR="$TMP_DIR/$SCHEMAS_PATH"
SCHEMA="${SCHEMAS_DIR}schema.json"

function init() {
  set -e
  trap 'tear_down' EXIT

  setup_mysql
  setup_server

  mkdir "$TMP_DIR"
  curl -s -o "$ZIP_FILE" -L \
    'https://github.com/dini-ag-kim/amb/archive/refs/heads/main.zip'
  unzip -q "$ZIP_FILE" -d "$TMP_DIR"
}

function main() {
  for ID in 1495 35596 18514 2823 2217 18865; do
    check_metadata $ID
  done
}

function check_metadata() {
  (( AFTER = $1 - 1 ))
  PAYLOAD="{ \
    \"type\": \"EntitiesMetadataQuery\", \
    \"payload\": { \"first\": 1, \"after\": $AFTER } \
  }"

  METADATA=$(mktemp -t metadata-raw.XXXXXXXXXX.json -p "$TMP_DIR")

  curl -s --header "Content-Type: application/json" --data "$PAYLOAD" \
    http://localhost:8080/ | jq '.entities[0]' > "$METADATA"

  yarn ajv -c ajv-formats -s "$SCHEMA" -r "$SCHEMAS_DIR!(schema).json" \
    -d "$METADATA"

  if [ $? -eq 0 ]; then
    log "Metadata with id $1 is combatible!"
  fi
}

function tear_down() {
  rm -r "$TMP_DIR"
  kill_cargo_background_process
}

init
main
