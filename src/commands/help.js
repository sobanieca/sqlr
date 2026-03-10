const help = `
sqlr - A command-line tool for interacting with SQL databases.

Usage:

  sqlr [command] [options]

Commands:

  help                  Display this help text
  version               Display version info
  add-connection        Add new connection. Run without parameters to use interactive wizard.
                        Usage: sqlr add-connection [-n name] [-t type] [-s connection-string]

                        Supported types can be listed using 'get-connection-types' command.
                        Connections are stored locally and can be reused across commands.
                        Optionally encrypt sensitive connections with a password.

  rm-connection         Remove selected connection.
                        Usage: sqlr rm-connection [connection]
                               sqlr rm-connection -n my-connection

  get-connection        Get details of selected connection (name, type, connection string).
                        Usage: sqlr get-connection [connection]
                               sqlr get-connection -n my-connection

  get-connection-types  Get available connection types and connection string hints.
                        Usage: sqlr get-connection-types

  get-connections       List all defined connections (names and types).
                        Usage: sqlr get-connections

  describe              Describe all tables and columns available in database.
                        Usage: sqlr describe [-n connection] [--table] [--compact] [-f filter]

                        Shows database schema including table names, column names, types,
                        nullable flags, and foreign key relations.
                        Use --filter to narrow results by schema/table name.
                        Use --table or --compact for different output formats.

  query                 Run SQL query against specified database.
                        Usage: sqlr query "SELECT * FROM users" [-n connection]
                               sqlr query -q "SELECT 1" -n my-connection
                               sqlr query -i query.sql -n my-connection

                        Options:
                          -q, --query              SQL query to execute
                          -i, --input-file         Path to file containing SQL query
                          -o, --output-file        Save results as JSON to file
                          -n, --name               Connection name
                          -t, --type               Connection type (use instead of -n)
                          -s, --connection-string   Connection string (use instead of -n)
                          --table                  Display results as table
                          --compact                Display results in compact CSV-like form

  update                Display instructions for updating sqlr to the latest version.
                        Usage: sqlr update [--deno]

Typical workflow:

  1. Configure a connection:
     sqlr add-connection

  2. List available connections:
     sqlr get-connections

  3. Explore database schema:
     sqlr describe -n my-connection

  4. Run queries:
     sqlr query "SELECT * FROM users LIMIT 10" -n my-connection

AI agent instructions:

  When using sqlr as an AI agent, follow these steps:

  1. First, check available connections:
     sqlr get-connections

  2. If no connections are configured, add one using:
     sqlr add-connection -n <name> -t <type> -s <connection-string>
     Use 'sqlr get-connection-types' to see supported database types and connection string formats.

  3. To understand the database schema, run:
     sqlr describe -n <connection-name> --compact
     This returns a compact list of all tables and columns.
     Use --filter to narrow results: sqlr describe -n <connection-name> -f "users"

  4. To query data, use:
     sqlr query "SELECT * FROM table LIMIT 10" -n <connection-name>
     For larger results, save to file: sqlr query -q "SELECT *" -n <name> -o results.json

  Important notes for AI agents:
  - Always use -n <connection-name> to specify which connection to use
  - Use 'sqlr describe' to discover schema before writing queries
  - Use LIMIT in SELECT queries to avoid overwhelming output
  - Use --compact or --table flags for more parseable output
  - Use -o flag to save large result sets to a JSON file
  - Use --debug global flag for troubleshooting connection issues

Global options:

  --help                Display help for a command
  --debug               Enable debug logs

For each command use '--help' flag for details on additional options and arguments.
`;

export default {
  help,
};
