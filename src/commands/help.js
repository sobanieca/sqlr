const help = `
sqlr - A command-line tool for interacting with SQL databases.

Usage:

  sqlr [command] [options]

Connection scoping:

  Connections are automatically scoped to the nearest git repository.
  When you run sqlr inside a git repository, connections are stored and
  retrieved only for that repository. If no git repository is found
  (searching up to HOME directory), connections are stored in global scope.

  Use -g (--global) flag with any connection command to explicitly use
  global scope regardless of the current directory.

  Use 'set-global true' to switch to global mode (no need for -g).
  Use 'set-global false' to switch back to repository-scoped mode.

Commands:

  help                  Display this help text
  version               Display version info
  add                   Add new connection. Run without parameters to use interactive wizard.
                        Usage: sqlr add [-n name] [-t type] [-s connection-string] [-g]

                        Supported types can be listed using 'ls-types' command.
                        Connections are stored locally and can be reused across commands.
                        Optionally encrypt sensitive connections with a password.

  rm                    Remove selected connection.
                        Usage: sqlr rm [connection]
                               sqlr rm -n my-connection

  get                   Get details of selected connection (name, type, connection string).
                        Usage: sqlr get [connection]
                               sqlr get -n my-connection

  ls-types              Get available connection types and connection string hints.
                        Usage: sqlr ls-types

  ls                    List all defined connections (names and types).
                        The current default connection is marked with [current].
                        Usage: sqlr ls [-g]

  clear                 Remove all connections in current scope.
                        Usage: sqlr clear [-g]

  set                   Set default connection for current scope. When set,
                        query and describe commands will use it automatically
                        without requiring -n flag or interactive selection.
                        Run without arguments to clear the default connection.
                        Usage: sqlr set [connection]
                               sqlr set -n my-connection
                               sqlr set -n shared-db -g
                               sqlr set                    (clear default)

                        When used with -g, also enables global mode so the
                        default connection applies everywhere.

  set-global            Toggle global mode for connections.
                        Usage: sqlr set-global true    (use global scope by default)
                               sqlr set-global false   (use repository scope)

  describe              Describe all tables and columns available in database.
                        Usage: sqlr describe [-n connection] [--table] [--compact] [--tables-only] [-f filter]

                        Shows database schema including table names, column names, types,
                        nullable flags, and foreign key relations.
                        Use --filter to narrow results by schema/table name.
                        Use --table or --compact for different output formats.
                        Use --tables-only to list only table names without column details.

  query                 Run SQL query against specified database.
                        The argument can be a SQL string or a path to a .sql file
                        (auto-detected). Use {{variable}} syntax in SQL and provide
                        values with -i "variable: value".

                        The 'query' keyword is optional — any unrecognized argument
                        is treated as a query automatically:
                          sqlr "SELECT * FROM users"
                          sqlr query.sql -i "status: active"

                        Usage: sqlr "SELECT * FROM users"
                               sqlr query.sql -i "name: John" -i "status: active"
                               sqlr query "SELECT * FROM users" -n my-connection

                        Options:
                          -i, --input-variable     Input variable in "key: value" format for {{key}} substitution
                          -o, --output-file        Save results as JSON to file
                          -n, --name               Connection name
                          -t, --type               Connection type (use instead of -n)
                          -s, --connection-string   Connection string (use instead of -n)
                          --table                  Display results as table
                          --compact                Display results in compact CSV-like form
                          --ignore-input-validation  Skip validation for missing input variables
                          -y, --yes                Skip confirmation prompt when executing SQL from a file

  update                Display instructions for updating sqlr to the latest version.
                        Usage: sqlr update [--deno]

Typical workflow:

  1. Configure a connection:
     sqlr add

  2. Set a default connection:
     sqlr set my-connection

  3. Explore database schema:
     sqlr describe

  4. Run queries:
     sqlr "SELECT * FROM users LIMIT 10"
     sqlr query.sql -i "status: active"

  5. Use global connections across projects:
     sqlr add -n shared-db -t postgresql -s "..." -g
     sqlr set shared-db -g

AI agent instructions:

  When using sqlr as an AI agent, follow these steps:

  1. First, check available connections:
     sqlr ls

  2. If no connections are configured, add one using:
     sqlr add -n <name> -t <type> -s <connection-string>
     Use 'sqlr ls-types' to see supported database types and connection string formats.

  3. Set a default connection to avoid passing -n on every command:
     sqlr set <connection-name>

  4. To understand the database schema, run:
     sqlr describe --compact
     This returns a compact list of all tables and columns.
     Use --filter to narrow results: sqlr describe -f "users"

  5. To query data, use:
     sqlr "SELECT * FROM table LIMIT 10"
     For larger results, save to file: sqlr "SELECT *" -o results.json
     Use SQL files with variables: sqlr query.sql -i "param: value"
     To skip file confirmation prompt: sqlr query.sql -y

  Important notes for AI agents:
  - Use 'sqlr describe' to discover schema before writing queries
  - Use LIMIT in SELECT queries to avoid overwhelming output
  - Use --compact or --table flags for more parseable output
  - Use -o flag to save large result sets to a JSON file
  - Use -y flag to skip confirmation prompt when executing SQL from files
  - Use --debug global flag for troubleshooting connection issues
  - For encrypted connections, set SQLR_ENCRYPTION_PASSWORD env var to avoid interactive prompts
  - Connections are scoped to the git repository. Use -g flag to access global connections

Environment variables:

  SQLR_ENCRYPTION_PASSWORD  Password for encrypting/decrypting connections.
                            When set during 'add' with -s flag, the connection
                            will be encrypted automatically using this password.
                            When set during 'get', 'describe', or 'query',
                            encrypted connections will be decrypted without prompting.
                            This enables non-interactive usage by AI agents and scripts.

Global options:

  --help                Display help for a command
  --debug               Enable debug logs
  -g, --global          Use global scope for connections (ignore git repository scope)

For each command use '--help' flag for details on additional options and arguments.
`;

export default {
  help,
};
