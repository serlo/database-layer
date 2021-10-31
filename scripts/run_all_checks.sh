#!/bin/bash

source scripts/utils.sh

function init() {
  set -e

  read_arguments "$@"

  print_header "Make sure yarn packages are up to date"
  yarn install --frozen-lockfile

  setup_mysql
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

function main() {
  if [ -n "$NO_UNCOMMITTED_CHANGES" ]; then
    print_header "Check that there are no uncommitted changes when pushing"
    test_no_uncommitted_changes_when_pushing
  fi

  print_header "Check sqlx-data.json is up to date"
  test_sqlx_data_up_to_date

  print_header "Run linter"
  yarn clippy

  print_header "Run all tests"
  cargo test

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

init "$@"
main
