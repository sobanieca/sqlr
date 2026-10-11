#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SQLITE_DIR=/tmp/sqlr-sqlite
mkdir -p "$SQLITE_DIR"
chmod 777 "$SQLITE_DIR"

docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --wait

echo ""
echo "=== Connection strings ==="
echo "PostgreSQL: postgresql://world:world123@localhost:5432/world-db"
echo "MySQL:      mysql://world:world123@localhost:3306/world-db"
echo "MSSQL:      Server=localhost,1433;Database=world-db;User Id=sa;Password=World123!;"
echo "ClickHouse: http://world:world123@localhost:8123?database=world"
echo "SQLite:     $SQLITE_DIR/world.db"
