#!/bin/bash

function init() {
  set -e
  trap 'tear_down' EXIT

  BOLD=$(tput bold)
  NORMAL=$(tput sgr0)

  WAIT_FOR_CARGO_TIMEOUT=60
  CARGO_BACKGROUND_PID=
}

function main() {
  if ! server_is_running; then
    cargo run &
    CARGO_BACKGROUND_PID=$!
  fi

  wait_for_server

  yarn pacts
}

function tear_down() {
  kill_cargo_background_process
}

function wait_for_server() {
  START_TIMESTAMP=$(current_timestamp)

  while true; do
    if server_is_running; then
      break
    fi

    if [ -n "$CARGO_BACKGROUND_PID" ]; then
      if ! cargo_is_running; then
        error "Server could not be compiled"
      fi
    fi

    if (($(current_timestamp) - $START_TIMESTAMP > $WAIT_FOR_CARGO_TIMEOUT)); then
      error "Timeout: The server has not be started"
    fi

    sleep 1
  done
}

function server_is_running() {
  curl "http://localhost:8080/" > /dev/null 2>&1
}

function cargo_is_running() {
  ps -p $CARGO_BACKGROUND_PID > /dev/null 2>&1
}

function kill_cargo_background_process() {
  if [ -n "$CARGO_BACKGROUND_PID" ]; then
    if cargo_is_running; then
      kill $CARGO_BACKGROUND_PID
    fi
  fi
}

function current_timestamp() {
  date "+%s"
}

function error() {
  echo "${BOLD}ERROR: $*${NORMAL}" >&2
  exit 1
}

init
main
