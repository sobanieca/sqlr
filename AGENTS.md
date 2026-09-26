## Project

SQLR is a Deno-based CLI tool for executing SQL queries across PostgreSQL,
MySQL, MSSQL, and ClickHouse. Published as `@sobanieca/sqlr` on JSR.

## Commands

```bash
./sqlr                    # Run CLI (deno run --allow-all main.js)
deno task check                  # Format check + lint + type check + tests
deno task start-db               # Start all test databases via Docker
deno task stop-db                # Stop test databases
cd test && deno task update-snapshots             # Update snapshots after changes
```

## Architecture

**Entry point:** `main.js` wires up CLI commands using Cliffy's `Command`
framework.

**Connector pattern:** Each database implements a standard interface in
`src/connectors/`:

- `getDatabaseName()`, `getConnectionStringHint()`, `getConnectionString()` —
  metadata and interactive setup
- `getTables(connectionString)` — returns
  `[{schema, name, columns: [{name, type, nullable, relation}]}]`
- `query(connectionString, sql)` — returns `{rowsAffected, rows: []}`

New databases are added by creating a connector file and registering it in
`src/connectors.js`. Commands automatically pick it up.

**Connection storage:** `src/connection-accessor.js` uses Deno's `localStorage`
to persist connections as JSON (`{name, type, connectionString, isEncrypted}`).
Encryption uses AES-GCM + PBKDF2 in `src/guard.js` with machine-specific IV/salt
derived from `Deno.hostname()` + `Deno.osRelease()`.

**Dependencies:** All external imports are centralized in `src/deps.js`.

## Key Conventions

- Passwords in connection strings are URL-encoded via `encodeURIComponent()`
  during storage. Connectors decode them when connecting. `get-connection`
  displays decoded strings.
- Schema foreign keys are represented as `"schema.table.column"` strings.
- ClickHouse connector distinguishes SELECT-like queries from mutations for
  different client methods.
- The `--debug` flag enables full stack traces; without it, `DatabaseError`
  messages are shown as user-friendly text.

## IMPORTANT!

- Don't add comments to the code. Use self-documenting code
- When updating version update version.js and deno.json file. Update changelog
  if it's present.
- Use ES6 shorthand property syntax whenever possible when initiating objects
- When updating or introducing new features, ensure the `help` command text in
  `src/commands/help.js` is updated to reflect the changes
- All external dependencies (`npm:`, `jsr:`, `node:`, `https:` imports) MUST be
  imported and re-exported via `src/deps.js`. It is the single source of truth
  for external dependencies — no other file should import them directly.
