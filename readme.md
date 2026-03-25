> **Beta:** This project is in beta. Database connectors need more real-world
> usage to be fully validated across all supported databases.

# About

- :zap: Don't wait for your GUI tools to load, when you need to run simple SQL
  query
- :lock: Connection encryption for enhanced security (recommended for remote
  environments)
- :scroll: List your database structure to quickly identify which columns are in
  specific table
- :avocado: Present results in compact form, json or tables
- :file_folder: Use queries from `.sql` files and/or save results to `.json`
  files
- :abacus: CI/CD friendly - commands can be executed directly, without
  `interactive` prompts
- :robot: AI agent friendly - agents can query databases without exposing
  connection strings in their context

Supported databases: PostgreSQL, MySQL, MSSQL, ClickHouse.

## Usage

**1. Add a database connection**

```bash
sqlr add-connection
# ? Provide name of the connection: mydb
# ? Select database type: PostgreSQL
# ? Provide connection details...
# ? Do you want to additionally secure connection settings? No
# Connection has been added
```

**2. Set default connection for current scope**

```bash
sqlr set-connection mydb
# Default connection set to mydb
```

**3. Run queries directly**

```bash
sqlr "select id, name, email from users limit 3"
# 🟢 mydb
# Query executed in: 42ms
# Rows affected: 3
# [
#   { id: 1, name: "Alice", email: "alice@example.com" },
#   { id: 2, name: "Bob", email: "bob@example.com" },
#   { id: 3, name: "Charlie", email: "charlie@example.com" }
# ]
```

**4. Use SQL files with input variables**

```sql
-- queries/users-by-status.sql
SELECT id, name, email FROM users WHERE status = '{{status}}' LIMIT {{limit}}
```

```bash
sqlr queries/users-by-status.sql -i "status: active" -i "limit: 10"
```

Variables use `{{key}}` syntax in SQL and are provided via `-i "key: value"`.

## Using with AI agents

Sqlr stores database connections locally. Once you set a default connection with
`set-connection`, agents can run queries directly (e.g. `sqlr "SELECT ..."`),
without ever needing access to the actual connection string. Connection strings
stay on your machine and are never exposed in the agent's context.

To let the agent learn about all available sqlr commands and options, have it
run `sqlr help`.

**Example prompt:**

```
Run `sqlr help` to learn about sqlr capabilities. Then run
`sqlr set my-db-connection` and `sqlr describe --compact` to learn about my db
structure. Finally, update all users last login date so it's more than 7 days ago.
```

## SQL file collections

You can maintain a collection of `.sql` files and execute them with sqlr
whenever needed. This is useful for queries you run repeatedly - health checks,
reports, data fixes, etc.

```
queries/
  health-check.sql
  daily-report.sql
  cleanup-stale-sessions.sql
```

Run any file from the collection:

```bash
sqlr queries/health-check.sql
sqlr queries/daily-report.sql
```

This way your queries are version-controlled, reusable, and don't need to be
typed out or remembered each time.

## Learn More

Run `sqlr` for details on available commands. For each command use `--help` flag
for details on additional options and arguments.

## Installation

### Option 1: Install via Deno (Recommended)

> Prerequisites
>
> Deno runtime environment `https://deno.com`

```bash
deno install -g -f -r --allow-env --allow-net --allow-read --allow-write jsr:@sobanieca/sqlr
```

**Permissions:**

- `--allow-write` - Required for writing results to files (`-o` flag)
- `--allow-net` - Required for connecting to databases
- `--allow-read` - Required for reading `.sql` files
- `--allow-env` - Required for encryption and hostname detection

### Option 2: Quick Install Script (Standalone Binary)

If you don't have Deno installed, you can install the pre-compiled binary with a
single command:

```bash
curl -fsSL sobanieca.github.io/sqlr/install.sh | bash
```

This script automatically detects your OS and architecture (Linux/macOS,
x64/arm64) and installs the appropriate binary to `/usr/local/bin`.

To install to a custom location:

```bash
curl -fsSL sobanieca.github.io/sqlr/install.sh | INSTALL_DIR=~/bin bash
```

### Option 3: Manual Binary Installation

Download the latest pre-compiled binary for your operating system from the
[releases page](https://github.com/sobanieca/sqlr/releases/latest):

**Example for Linux x64:**

```bash
curl -L -o sqlr https://github.com/sobanieca/sqlr/releases/latest/download/sqlr-linux-x64
chmod +x sqlr
sudo mv sqlr /usr/local/bin/
```

Available binaries: `sqlr-linux-x64`, `sqlr-linux-arm64`, `sqlr-macos-x64`,
`sqlr-macos-arm64`

## Updating

Use `sqlr update` command and follow presented instructions to update.

## Hints

- If your queries are failing due to certificate validation errors (and you
  trust target server) you can install using following command:

```bash
deno install -g -f -r --unsafely-ignore-certificate-errors --allow-env --allow-net --allow-read --allow-write jsr:@sobanieca/sqlr
```

You can consider introducing `sqlr-unsafe` sitting next to your main `sqlr`
instance to work with trusted servers with problematic certificates:

```bash
deno install -g -f -r -n sqlr-unsafe --unsafely-ignore-certificate-errors --allow-net --allow-read --allow-write jsr:@sobanieca/sqlr
```

- If you want to disable colors (at least for main log messages), you can use:

```bash
NO_COLOR=1 sqlr ...
```

## Contribution

If you want to implement/request new features you are more than welcome to
contribute. Before you start working on some new features please keep in mind
that this tool is supposed to be extremely simple to use and cover most of the
daily use cases without handling edge cases. For edge cases proper database GUI
app may be required.
