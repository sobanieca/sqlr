#!/bin/bash

# Wait for SQL Server to be ready
for i in {1..30}; do
  /opt/mssql-tools18/bin/sqlcmd -S mssql -U sa -P 'Sqlr_ms_x26MtA0uEdP7lK' -C -Q "SELECT 1" > /dev/null 2>&1
  if [ $? -eq 0 ]; then
    echo "SQL Server is ready"
    break
  fi
  echo "Waiting for SQL Server... ($i/30)"
  sleep 2
done

# Run init script
/opt/mssql-tools18/bin/sqlcmd -S mssql -U sa -P 'Sqlr_ms_x26MtA0uEdP7lK' -C -i /init/01-schema.sql

echo "Database initialized"
