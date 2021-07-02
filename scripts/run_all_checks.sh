#!/bin/bash

function init() {
  set -e

  BOLD=$(tput bold)
  NORMAL=$(tput sgr0)

  read_arguments "$@"

  if ! mysql_is_running; then
    yarn start
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
    print_test_header "Check that there are no uncommitted changes when pushing"
    test_no_uncommitted_changes_when_pushing
  fi

  print_test_header "Run all tests"
  cargo test

  print_test_header "Run linter"
  yarn clippy

  print_test_header "Check sqlx-data.json is up to date"
  test_sqlx_data_up_to_date

  print_test_header "Run pact tests"
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

function print_test_header() {
  echo
  log "=== $@ ==="
}

function log() {
  echo "${BOLD}$@${NORMAL}"
}

init "$@"
main
