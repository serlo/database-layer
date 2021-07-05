#!/bin/bash

function init() {
  set -e

  BOLD=$(tput bold)
  NORMAL=$(tput sgr0)

  WAIT_FOR_MYSQL_TIMEOUT=20

  read_arguments "$@"

  print_header "Make sure yarn packages are up to date"
  yarn install --frozen-lockfile

  if ! mysql_is_running; then
    print_header "Make sure mysql is running"
    yarn start
    log "MySQL need to start, let's wait 30 seconds until it has started..."
    sleep 30
  fi
}

function read_arguments() {
  if [ -n "$1" ]; then
    if [ "$1" = "--no-uncommitted-changes" ]; then
      NO_UNCOMMITTED_CHANGES="True"
    else
      error "Unknown parameter provided"
    fi
  fi
}

function mysql_is_running() {
  nc -z localhost 3306
}

function main() {
  if [ -n "$NO_UNCOMMITTED_CHANGES" ]; then
    print_header "Check that there are no uncommitted changes when pushing"
    test_no_uncommitted_changes_when_pushing
  fi

  print_header "Check sqlx-data.json is up to date"
  test_sqlx_data_up_to_date

  print_header "Run all tests"
  cargo test

  print_header "Run linter"
  yarn clippy

  print_header "Run pact tests"
  ./scripts/pacts.sh
}

function test_no_uncommitted_changes_when_pushing() {
  if [ -n "$(git diff HEAD)" ]; then
    error "There are uncommitted changes in your workspace"
  fi
}

function test_sqlx_data_up_to_date() {
  yarn sqlx:prepare

  if [ -n "$(git diff sqlx-data.json)" ]; then
    error "You need to run sqlx:prepare and commit the changes in sqlx-data.json!"
  fi
}

function error() {
  log "Error: $@"
  exit 1
}

function print_header() {
  echo
  log "=== $@ ==="
}

function log() {
  echo "${BOLD}$@${NORMAL}"
}

function current_timestamp() {
  date "+%s"
}

init "$@"
main
