#!/bin/bash
echo "ssl = on" >> "$PGDATA/postgresql.conf"
echo "ssl_cert_file = '/var/lib/postgresql/ssl/server.crt'" >> "$PGDATA/postgresql.conf"
echo "ssl_key_file = '/var/lib/postgresql/ssl/server.key'" >> "$PGDATA/postgresql.conf"
