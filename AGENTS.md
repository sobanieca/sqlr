## Project

SQLR is a CLI tool for executing SQL queries across PostgreSQL, MySQL, MSSQL,
ClickHouse and SQLite. It's written in Rust and distributed as a single binary.

## Commands

Development tasks are cargo aliases defined in `.cargo/config.toml` (backed by
the `xtask` crate):

```bash
cargo sqlr [args]        # Run sqlr from source
cargo verify             # fmt --check, clippy -D warnings and all tests
cargo start-db           # Start all test databases via Docker
cargo test-db            # Tests against the test databases (TZ=UTC)
cargo stop-db            # Stop test databases
cargo update-snapshots   # Accept new CLI output (needs the test databases)
```

`cargo verify` is the pre-commit check. CI runs it and `cargo test-db`.

## Architecture

**Entry point:** `src/main.rs` defines the command tree and dispatches to
`src/commands/`. Any first argument that isn't an option or a command is a
query (`sqlr "SELECT 1"` is `sqlr query "SELECT 1"`).

**CLI and output:** `src/cli.rs` (argument parsing, validation errors and help
output) and `src/table.rs` (tables) reproduce the output of the Cliffy library
used by the former Deno version. `src/prompt.rs` holds the interactive prompts
(Input, Secret, Select, Toggle).

**Connector pattern:** Each database implements `Connector` in
`src/connectors/`:

- `database_name`, `connection_string_hint`, `prompt_connection_string` —
  metadata and interactive setup
- `get_tables(connection_string)` — tables with columns
  (`{name, data_type, nullable, relation}`)
- `query(connection_string, sql)` — `{rows_affected, rows}`

New databases are added by creating a connector module and registering it in
`CONNECTORS` in `src/connectors/mod.rs`. Commands automatically pick it up.

**Values:** `src/value.rs` models result values with JavaScript semantics
(numbers, strings, dates, buffers...) so the output matches what the Node.js
drivers produced: `Deno.inspect`-style output (`src/inspect.rs`),
`JSON.stringify` for `-o` and `String(value)` for `--compact`/`--table`.
Connectors convert driver values the way the Node.js drivers did (e.g. bigint
and decimal as strings, dates as `Date`).

**Connection storage:** `src/storage.rs` keeps connections and settings in
`$XDG_CONFIG_HOME/sqlr/storage.json` (default `~/.config/sqlr`). Connections are
scoped to the nearest git repository or the global scope (`src/scope.rs`).
Encryption (`src/guard.rs`) uses AES-256-GCM with a PBKDF2-SHA256 key and a
random salt and nonce per connection.

## Testing

- Unit tests live next to the code (`#[cfg(test)]`).
- `tests/cli.rs` and `tests/databases.rs` run the binary in an isolated
  environment and compare the output with `tests/snapshots/*.snap`. Database
  tests are `#[ignore]`d and need the databases from `tests/db`.

## Key Conventions

- Passwords in connection strings are URL-encoded. Connectors decode them when
  connecting. `get` displays decoded strings.
- Schema foreign keys are represented as `"schema.table.column"` strings.
- ClickHouse and SQLite connectors distinguish SELECT-like queries from
  mutations.
- Errors reported by the database are shown as is; other failures show a
  generic message, details with `--debug`. Failed commands exit with code 1.

## IMPORTANT!

- Don't add comments to the code. Use self-documenting code (module doc
  comments are fine)
- When updating the version update `Cargo.toml`.
- When updating or introducing new features, ensure the help text in
  `src/help.txt` is updated to reflect the changes
- When updating or introducing new features, also update the landing page in
  `docs/index.html` so it reflects the changes
- Keep dependencies minimal: database drivers, TLS and `ureq`. Formatting,
  tables, prompts and JSON are implemented in the crate.

## Publishing

Every push to `main` builds binaries for Linux (musl, static) and macOS
(x64/arm64) and publishes them as GitHub release `v<version>` (version from
`Cargo.toml`). `install.sh` and `sqlr update` download these release assets,
so their names (`sqlr-linux-x64`, ...) must not change.
