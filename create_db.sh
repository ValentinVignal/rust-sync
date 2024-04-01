#!/bin/bash

set -e
set -u

echo "  Creating database 'test'"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
	    CREATE DATABASE "test";
	    GRANT ALL PRIVILEGES ON DATABASE "test" TO "$POSTGRES_USER";
EOSQL


# https://gilgamezh.me/en/posts/postgres-non-durable-options-docker-container/
# no need to flush data to disk.
echo "fsync = off" >> /var/lib/postgresql/data/postgresql.conf
# no need to force WAL writes to disk on every commit.
echo "synchronous_commit = off" >> /var/lib/postgresql/data/postgresql.conf
# no need to guard against partial page writes.
echo "full_page_writes = off" >> /var/lib/postgresql/data/postgresql.conf