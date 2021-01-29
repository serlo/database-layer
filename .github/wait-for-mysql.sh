#!/bin/sh
while ! docker-compose exec -T mysql mysql --user=root --password=secret --execute="SELECT 1" >/dev/null 2>&1; do
    sleep 1
done
