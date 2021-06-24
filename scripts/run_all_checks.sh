#!/bin/bash

function init {
	set -e
	trap 'kill_background_jobs' EXIT

	BOLD=$(tput bold)
	NORMAL=$(tput sgr0)

	read_arguments "$@"
}

function read_arguments {
	if [ -n "$1" ]; then
		if [ "$1" = "--no-uncommitted-changes" ]; then
			NO_UNCOMMITTED_CHANGES="True"
		else
			error "Unknown parameter provided"
		fi
	fi
}

function main {
	if [ -n "$NO_UNCOMMITTED_CHANGES" ]; then
		print_test_header "Check that there are no uncommitted changes when pushing"
		test_no_uncommitted_changes_when_pushing
	fi

	print_test_header "Run all tests"
	cargo test

	print_test_header "Run linter"
	cargo clippy

	print_test_header "Check sqlx-data.json is up to date"
	test_sqlx_data_up_to_date

	print_test_header "Run pact tests"
	run_pact_tests
}

function test_no_uncommitted_changes_when_pushing {
	if [ -n "$(git diff HEAD)" ]; then
		error "There are uncommitted changes in your workspace"
	fi
}

function test_sqlx_data_up_to_date {
	yarn sqlx:prepare

	if [ -n "$(git diff sqlx-data.json)" ]; then
		error "You need to run sqlx:prepare and commit the changes in sqlx-data.json!"
	fi
}

function run_pact_tests {
	# Run db layer server in background
	cargo run --quiet &
	DB_LAYER_SERVER_ID=$!
	sleep 20

	# run tests
	yarn pacts

	# stop server
	kill $DB_LAYER_SERVER_ID
}

function kill_background_jobs {
	if [ -n "$(get_all_background_jobs)" ]; then
		kill $(get_all_background_jobs)
	fi
}

function get_all_background_jobs {
	jobs -p
}

function error {
	log "Error: $@"
	exit 1
}

function print_test_header {
	echo
	log "=== $@ ==="
}

function log {
	echo "${BOLD}$@${NORMAL}"
}

init "$@"
main
