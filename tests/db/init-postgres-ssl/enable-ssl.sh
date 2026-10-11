#!/bin/bash
echo "ssl = on" >> "$PGDATA/postgresql.conf"
echo "ssl_cert_file = '/var/lib/postgresql/ssl/server.crt'" >> "$PGDATA/postgresql.conf"
echo "ssl_key_file = '/var/lib/postgresql/ssl/server.key'" >> "$PGDATA/postgresql.conf"

cat > "$PGDATA/pg_hba.conf" <<EOF
local all all trust
hostssl all all 0.0.0.0/0 md5
hostssl all all ::/0 md5
EOF
