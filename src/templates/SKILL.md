---
name: ${name}
description: Run SQL queries and inspect the schema of ${target} with the sqlr CLI. Use when asked to query, inspect, report on or change data in ${target}.
---

# ${name}

`sqlr` runs SQL against connections saved on this machine. Connections are
referenced by name. You never need, and must never ask for, a connection
string or password.

## Workflow

1. Run `sqlr skill` once to read the full reference (all commands and flags).
${select}
3. `sqlr describe --compact` shows tables, columns, types and foreign keys.
   Narrow it with `-f <table>`. Read the schema before writing queries.
4. Run queries: `sqlr "SELECT ... LIMIT 20"` or
   `sqlr path/to/query.sql -i "param: value"`.
5. Use `-o result.json` for large results and `--table` for readable output.

## Rules

- Always add LIMIT to exploratory SELECT statements.
- Show UPDATE, DELETE, DROP and other destructive statements to the user and
  ask before running them.
- If a command prompts for an encryption password, stop and ask the user to
  run it, or to set SQLR_ENCRYPTION_PASSWORD.
