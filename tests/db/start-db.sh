#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SQLITE_DIR=/tmp/sqlr-sqlite
mkdir -p "$SQLITE_DIR"
chmod 777 "$SQLITE_DIR"

docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --wait --renew-anon-volumes postgres postgres-ssl mysql mssql clickhouse
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up --abort-on-container-failure mssql-init sqlite-init > /dev/null

echo ""
echo "=== Connection strings ==="
echo "PostgreSQL: postgresql://world:sqlr_pg_aZ3yeTHbJe8@localhost:15432/world-db"
echo "MySQL:      mysql://world:sqlr_my_YRanBbUweR3vZzeh@localhost:13306/world-db"
echo "MSSQL:      Server=localhost,11433;Database=world-db;User Id=sa;Password=Sqlr_ms_x26MtA0uEdP7lK;"
echo "ClickHouse: http://world:sqlr_ch_o95Me4893wbRFg2NM@localhost:18123?database=world"
echo "SQLite:     $SQLITE_DIR/world.db"
