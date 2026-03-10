import { Input, Secret } from "../deps.js";
import { DatabaseError } from "../database-error.js";
import { createClient } from "npm:@clickhouse/client@1";

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

  const baseUrl = `${url.protocol}//${
    url.username ? `${url.username}:${url.password}@` : ""
  }${url.host}`;

  return { baseUrl, database };
};

const clickhouseConnector = {
  getDatabaseName: () => "ClickHouse",
  getConnectionStringHint: () =>
    `
http://user:password@host:port?database=database_name
Default port: 8123 (HTTP interface)
More details: https://clickhouse.com/docs/en/interfaces/http
  `.trim(),
  getConnectionString: async () => {
    const host = await Input.prompt(
      "Database host (example: localhost | my.db.com)",
    );
    const port = await Input.prompt("Port (default: 8123)") || 8123;
    const dbName = await Input.prompt("Database name (default: default)") ||
      "default";
    const user = await Input.prompt("Username (default: default)") || "default";
    const password = encodeURIComponent(await Secret.prompt("Password"));

    return `http://${user}:${password}@${host}:${port}?database=${dbName}`;
  },
  getTables: async (connectionString) => {
    const { baseUrl, database } = parseConnectionString(connectionString);
    const client = createClient({
      url: baseUrl,
      database: database,
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
            database = '${database}'
            AND is_temporary = 0
            AND name NOT LIKE '.%'
          ORDER BY
            database, name`,
        format: "JSONEachRow",
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
            database = '${database}'
          ORDER BY
            database, table, position`,
        format: "JSONEachRow",
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
    } finally {
      await client.close();
    }
  },
  query: async (connectionString, query) => {
    const { baseUrl, database } = parseConnectionString(connectionString);
    const client = createClient({
      url: baseUrl,
      database: database,
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
      throw new DatabaseError(
        `ClickHouseError: ${err.message || err.toString()}`,
      );
    } finally {
      await client.close();
    }
  },
};

export { clickhouseConnector };
