#!/bin/bash
cp /tmp/server.key /etc/ssl/private/pg-server.key
chown postgres:postgres /etc/ssl/private/pg-server.key
chmod 600 /etc/ssl/private/pg-server.key
cp /tmp/server.crt /etc/ssl/certs/pg-server.crt
chown postgres:postgres /etc/ssl/certs/pg-server.crt
exec gosu postgres docker-entrypoint.sh postgres
