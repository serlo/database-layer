#!/bin/sh
while ! docker-compose exec -T mysql mysql --user=root --password=secret --execute="SELECT 1" >/dev/null 2>&1; do
    docker-compose exec -T mysql mysql --user=root --password=secret --execute="SELECT 1"
    sleep 1
done
echo "Database should be ready now"
