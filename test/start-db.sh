#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --wait

echo ""
echo "=== Connection strings ==="
echo "PostgreSQL: postgresql://world:world123@localhost:5432/world-db"
echo "MySQL:      mysql://world:world123@localhost:3306/world-db"
echo "MSSQL:      Server=localhost,1433;Database=world-db;User Id=sa;Password=World123!;"
echo "ClickHouse: http://world:world123@localhost:8123?database=world"
