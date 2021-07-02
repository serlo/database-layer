#!/bin/sh
while ! curl --silent --fail http://localhost:8080/.well-known/health > /dev/null 2>&1; do
  sleep 1
done
