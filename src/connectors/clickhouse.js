import { createClickhouseClient, Input, Secret, Select } from "../deps.js";
import { DatabaseError } from "../database-error.js";

const DEFAULT_CONNECTION_TIMEOUT_MS = 10000;

const isSelectQuery = (query) => {
  const trimmed = query.trim().toUpperCase();
  return trimmed.startsWith("SELECT") ||
    trimmed.startsWith("WITH") ||
    trimmed.startsWith("SHOW") ||
    trimmed.startsWith("DESCRIBE") ||
    trimmed.startsWith("EXISTS");
};

const parseConnectionString = (connectionString) => {
  const url = new URL(connectionString);
  const database = url.searchParams.get("database") ||
    url.pathname.replace("/", "") || "default";
  const timeoutMs = parseInt(url.searchParams.get("connectionTimeoutMs"), 10) ||
    DEFAULT_CONNECTION_TIMEOUT_MS;

  const baseUrl = `${url.protocol}//${
    url.username ? `${url.username}:${url.password}@` : ""
  }${url.host}`;

  return { baseUrl, database, timeoutMs };
};

const clickhouseConnector = {
  getDatabaseName: () => "ClickHouse",
  getConnectionStringHint: () =>
    `
http(s)://user:password@host:port?database=database_name
Default port: 8123 (HTTP), 8443 (HTTPS)
Additional url parameters:
'connectionTimeoutMs' - connection timeout in milliseconds (default: ${DEFAULT_CONNECTION_TIMEOUT_MS})
More details: https://clickhouse.com/docs/en/interfaces/http
  `.trim(),
  getConnectionString: async () => {
    const hostInput = await Input.prompt(
      "Database host (default: localhost | example: my.db.com)",
    ) || "localhost";

    const protocolMatch = hostInput.match(/^(https?):\/\//i);
    let host = protocolMatch
      ? hostInput.slice(protocolMatch[0].length)
      : hostInput;
    host = host.replace(/\/+$/, "");

    const port = await Input.prompt("Port (default: 8123)") || 8123;
    const dbName = await Input.prompt("Database name (default: default)") ||
      "default";
    const user = await Input.prompt("Username (default: default)") || "default";
    const password = encodeURIComponent(await Secret.prompt("Password"));
    const protocol = protocolMatch
      ? protocolMatch[1].toLowerCase()
      : await Select.prompt({
        message: "Protocol (default: http)",
        options: [
          { name: "http", value: "http" },
          { name: "https", value: "https" },
        ],
      });
    const timeoutMs = await Input.prompt(
      `Connection timeout in ms (default: ${DEFAULT_CONNECTION_TIMEOUT_MS})`,
    ) || DEFAULT_CONNECTION_TIMEOUT_MS;

    return `${protocol}://${user}:${password}@${host}:${port}?database=${dbName}&connectionTimeoutMs=${timeoutMs}`;
  },
  getTables: async (connectionString) => {
    const { baseUrl, database, timeoutMs } = parseConnectionString(
      connectionString,
    );
    const client = createClickhouseClient({
      url: baseUrl,
      database: database,
      request_timeout: timeoutMs,
      log: { LoggerClass: null },
    });

    try {
      const tablesResult = await client.query({
        query: `
          SELECT
            database AS table_schema,
            name AS table_name
          FROM
            system.tables
          WHERE
            database = {database: String}
            AND is_temporary = 0
            AND name NOT LIKE '.%'
          ORDER BY
            database, name`,
        format: "JSONEachRow",
        query_params: { database },
      });

      const tables = await tablesResult.json();

      const columnsResult = await client.query({
        query: `
          SELECT
            database AS table_schema,
            table AS table_name,
            name AS column_name,
            default_expression AS column_default,
            type AS data_type
          FROM
            system.columns
          WHERE
            database = {database: String}
          ORDER BY
            database, table, position`,
        format: "JSONEachRow",
        query_params: { database },
      });

      const columns = await columnsResult.json();

      return tables.map((table) => ({
        schema: table.table_schema,
        name: table.table_name,
        columns: columns
          .filter((column) =>
            column.table_schema === table.table_schema &&
            column.table_name === table.table_name
          )
          .map((column) => {
            const type = column.data_type;
            const isNullable = type.startsWith("Nullable(");
            const displayType = isNullable
              ? type.slice("Nullable(".length, -1)
              : type;

            return {
              name: column.column_name,
              type: displayType,
              nullable: isNullable,
            };
          }),
      }));
    } catch (err) {
      throw new DatabaseError(
        `ClickHouseError: ${err.message || err.toString()}`,
      );
    } finally {
      await client.close();
    }
  },
  query: async (connectionString, query) => {
    const { baseUrl, database, timeoutMs } = parseConnectionString(
      connectionString,
    );
    const client = createClickhouseClient({
      url: baseUrl,
      database: database,
      request_timeout: timeoutMs,
      log: { LoggerClass: null },
    });

    try {
      if (isSelectQuery(query)) {
        const result = await client.query({
          query: query,
          format: "JSONEachRow",
        });

        const rows = await result.json();

        return {
          rowsAffected: rows.length,
          rows: rows,
        };
      }

      await client.command({ query: query });

      return {
        rowsAffected: 0,
        rows: [],
      };
    } catch (err) {
      const msg = err.message || err.toString();
      const truncated = msg.includes("Expected one of:")
        ? msg.slice(0, msg.indexOf("Expected one of:")).trim()
        : msg;
      throw new DatabaseError(`ClickHouseError: ${truncated}`);
    } finally {
      await client.close();
    }
  },
};

export { clickhouseConnector };
