#!/bin/bash

BOLD=$(tput bold)
NORMAL=$(tput sgr0)
CARGO_BACKGROUND_PID=
WAIT_FOR_CARGO_TIMEOUT=180

function setup_server() {
  if ! server_is_running; then
    cargo run &
    CARGO_BACKGROUND_PID=$!
  fi

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

function setup_mysql() {
  if ! mysql_is_running; then
    yarn start
    log "MySQL need to start, let's wait 30 seconds until it has started..."
    sleep 30
  fi
}

function server_is_running() {
  curl "http://localhost:8080/" > /dev/null 2>&1
}

function cargo_is_running() {
  ps -p $CARGO_BACKGROUND_PID > /dev/null 2>&1
}

function mysql_is_running() {
  nc -z localhost 3306
}

function kill_cargo_background_process() {
  if [ -n "$CARGO_BACKGROUND_PID" ]; then
    if cargo_is_running; then
      kill $CARGO_BACKGROUND_PID
    fi
  fi
}

function error() {
  log "ERROR: $@"
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
