#!/bin/bash
echo "ssl = on" >> "$PGDATA/postgresql.conf"
echo "ssl_cert_file = '/etc/ssl/certs/pg-server.crt'" >> "$PGDATA/postgresql.conf"
echo "ssl_key_file = '/etc/ssl/private/pg-server.key'" >> "$PGDATA/postgresql.conf"
