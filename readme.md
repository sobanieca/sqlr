Website: https://sobanieca.github.io/sqlr/

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

Supported databases: PostgreSQL, MySQL, MSSQL, ClickHouse, SQLite.

## Usage

**1. Add a connection**

```bash
sqlr add
# ? Provide name of the connection: mydb
# ? Select database type: PostgreSQL
# ? Provide connection details...
# ? Do you want to additionally secure connection settings? No
# Connection has been added
```

**2. Set default connection**

```bash
sqlr set mydb
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
`set`, agents can run queries directly (e.g. `sqlr "SELECT ..."`), without ever
needing access to the actual connection string. Connection strings stay on your
machine and are never exposed in the agent's context.

To let the agent learn about all available sqlr commands and options, have it
run `sqlr skill` (the help text with a short preamble for agents).

### Creating a `/sqlr` skill in your project

```bash
sqlr skill --init
```

This writes `SKILL.md` to `.agents/skills/sqlr/` (Codex, Cursor) and
`.claude/skills/sqlr/` (Claude Code, Cursor). Inside a git repository the skill
goes to the git root, so the agent sees it in every folder of the repository,
monorepo packages included. Outside a git repository the skill goes to your home
directory (`~/.agents/skills/sqlr/` and `~/.claude/skills/sqlr/`), so agents see
it in every project. Then:

```
/sqlr how many orders were placed last week, grouped by country
```

### One skill per connection

```bash
sqlr skill --init -n clickhouse-prod
```

This creates `/sqlr-clickhouse-prod`, a skill that always runs
`sqlr set clickhouse-prod` first. Add one per environment, e.g.
`/sqlr-postgres-staging`, `/sqlr-mysql-local`. The connection strings stay on
your machine, the agent only ever references the connection by name.

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

`sqlr` is a single binary for Linux and macOS (x64 and arm64). No runtime is
needed.

### Option 1: Quick Install Script (Recommended)

```bash
curl -fsSL sobanieca.github.io/sqlr/install.sh | bash
```

This script automatically detects your OS and architecture (Linux/macOS,
x64/arm64) and installs the appropriate binary to `/usr/local/bin`, for all users. If you
can't write to `/usr/local/bin`, the script stops with an error. Run it with
`sudo`:

```bash
curl -fsSL sobanieca.github.io/sqlr/install.sh | sudo bash
```

Or install to a custom location: set `INSTALL_DIR` to a directory that is on
your `PATH`:

```bash
curl -fsSL sobanieca.github.io/sqlr/install.sh | INSTALL_DIR=~/.local/bin bash
```

### Option 2: Manual Binary Installation

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

```bash
sqlr update
```

This downloads the latest release and replaces the binary. If `sqlr` is
installed in a system directory (like `/usr/local/bin`) run `sudo sqlr update`.
`sqlr` checks for new versions once a day and prints a notice when one is
available (set `SQLR_NO_UPDATE_CHECK=1` to disable it).

## Hints

- If your queries are failing due to certificate validation errors (and you
  trust the target server), relax certificate validation in the connection
  string: `sslmode=require` (PostgreSQL), `ssl={"rejectUnauthorized":false}`
  (MySQL) or `trustServerCertificate=true` (MSSQL). Run `sqlr ls-types` to see
  all connection string options.

- If you want to disable colors, you can use:

```bash
NO_COLOR=1 sqlr ...
```

- Connections are stored in `~/.config/sqlr/storage.json` (readable only by
  your user). Use encrypted connections (`sqlr add` wizard or
  `SQLR_ENCRYPTION_PASSWORD`) for sensitive environments.

## Development

`sqlr` is written in Rust. Development tasks are cargo aliases:

```bash
cargo sqlr [args]        # Run sqlr from source
cargo verify             # fmt --check, clippy and tests (run before committing)
cargo start-db           # Start the test databases (Docker)
cargo test-db            # Run tests against the test databases
cargo stop-db            # Stop the test databases
cargo update-snapshots   # Accept new CLI output in tests/snapshots
```

## Contribution

If you want to implement/request new features you are more than welcome to
contribute. Before you start working on some new features please keep in mind
that this tool is supposed to be extremely simple to use and cover most of the
daily use cases without handling edge cases. For edge cases proper database GUI
app may be required.
