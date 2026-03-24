#!/bin/bash
mkdir -p /var/lib/postgresql/ssl
openssl req -new -x509 -days 365 -nodes -newkey rsa:2048 \
  -out /var/lib/postgresql/ssl/server.crt \
  -keyout /var/lib/postgresql/ssl/server.key \
  -subj "/CN=localhost" 2>/dev/null
chmod 600 /var/lib/postgresql/ssl/server.key
exec docker-entrypoint.sh postgres
