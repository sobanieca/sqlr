#!/bin/bash
openssl req -new -x509 -days 365 -nodes -newkey rsa:2048 \
  -out /etc/ssl/certs/pg-server.crt \
  -keyout /etc/ssl/private/pg-server.key \
  -subj "/CN=localhost" 2>/dev/null
chown postgres:postgres /etc/ssl/certs/pg-server.crt /etc/ssl/private/pg-server.key
chmod 600 /etc/ssl/private/pg-server.key
exec gosu postgres docker-entrypoint.sh postgres
