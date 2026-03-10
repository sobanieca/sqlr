import { Input, Secret } from "../deps.js";
import { DatabaseError } from "../database-error.js";
import mssql from "npm:mssql@11";

const parseConnectionString = (connectionString) => {
  const url = new URL(connectionString);
  const params = url.searchParams;

  return {
    server: url.hostname,
    port: parseInt(url.port) || 1433,
    database: url.pathname.replace("/", ""),
    user: decodeURIComponent(url.username),
    password: decodeURIComponent(url.password),
    options: {
      encrypt: params.get("encrypt") !== "false",
      trustServerCertificate: params.get("trustServerCertificate") === "true",
    },
  };
};

const mssqlConnector = {
  getDatabaseName: () => "MS SQL Server",
  getConnectionStringHint: () =>
    `
mssql://user:password@host:port/database_name
Additional url parameters:
'encrypt' - true | false (default: true)
'trustServerCertificate' - true | false (default: false)
  `.trim(),
  getConnectionString: async () => {
    const host = await Input.prompt(
      "Database host (example: localhost | my.db.com)",
    );
    const port = await Input.prompt("Port (default: 1433)") || 1433;
    const dbName = await Input.prompt("Database name");
    const user = await Input.prompt("Username");
    const password = encodeURIComponent(await Secret.prompt("Password"));

    return `mssql://${user}:${password}@${host}:${port}/${dbName}?encrypt=true&trustServerCertificate=false`;
  },
  getTables: async (connectionString) => {
    const config = parseConnectionString(connectionString);
    const pool = await mssql.connect(config);

    try {
      const tablesResult = await pool.request().query(`
        SELECT
          table_schema,
          table_name
        FROM
          INFORMATION_SCHEMA.TABLES
        WHERE
          table_type = 'BASE TABLE'
          AND table_schema NOT IN ('sys', 'INFORMATION_SCHEMA')
        ORDER BY
          table_schema, table_name`);

      const tables = tablesResult.recordset;

      const columnsResult = await pool.request().query(`
        SELECT
          table_schema,
          table_name,
          column_name,
          column_default,
          is_nullable,
          data_type
        FROM
          INFORMATION_SCHEMA.COLUMNS
        WHERE
          table_schema NOT IN ('sys', 'INFORMATION_SCHEMA')
        ORDER BY
          table_schema, table_name, ORDINAL_POSITION`);

      const columns = columnsResult.recordset;

      const foreignKeysResult = await pool.request().query(`
        SELECT
          fk.TABLE_SCHEMA AS table_schema,
          fk.TABLE_NAME AS table_name,
          cu.COLUMN_NAME AS column_name,
          pk.TABLE_SCHEMA AS foreign_table_schema,
          pk.TABLE_NAME AS foreign_table_name,
          pt.COLUMN_NAME AS foreign_column_name
        FROM
          INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc
          JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS fk
            ON rc.CONSTRAINT_NAME = fk.CONSTRAINT_NAME
            AND rc.CONSTRAINT_SCHEMA = fk.CONSTRAINT_SCHEMA
          JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS pk
            ON rc.UNIQUE_CONSTRAINT_NAME = pk.CONSTRAINT_NAME
            AND rc.UNIQUE_CONSTRAINT_SCHEMA = pk.CONSTRAINT_SCHEMA
          JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE cu
            ON rc.CONSTRAINT_NAME = cu.CONSTRAINT_NAME
            AND rc.CONSTRAINT_SCHEMA = cu.CONSTRAINT_SCHEMA
          JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE pt
            ON rc.UNIQUE_CONSTRAINT_NAME = pt.CONSTRAINT_NAME
            AND rc.UNIQUE_CONSTRAINT_SCHEMA = pt.CONSTRAINT_SCHEMA`);

      const foreignKeys = foreignKeysResult.recordset;

      return tables.map((table) => ({
        schema: table.table_schema,
        name: table.table_name,
        columns: columns
          .filter((column) =>
            column.table_schema === table.table_schema &&
            column.table_name === table.table_name
          )
          .map((column) => ({
            name: column.column_name,
            type: column.data_type,
            nullable: column.is_nullable === "YES",
            relation: foreignKeys
              .map((fk) => ({
                ...fk,
                relation:
                  `${fk.foreign_table_schema}.${fk.foreign_table_name}.${fk.foreign_column_name}`,
              }))
              .find((fk) =>
                fk.table_schema === table.table_schema &&
                fk.table_name === table.table_name &&
                fk.column_name === column.column_name
              )?.relation,
          })),
      }));
    } finally {
      await pool.close();
    }
  },
  query: async (connectionString, query) => {
    const config = parseConnectionString(connectionString);
    const pool = await mssql.connect(config);

    try {
      const result = await pool.request().query(query);

      return {
        rowsAffected: result.rowsAffected
          ? result.rowsAffected.reduce((a, b) => a + b, 0)
          : 0,
        rows: result.recordset || [],
      };
    } catch (err) {
      if (err.name === "RequestError" || err.name === "MSSQLError") {
        throw new DatabaseError(`MSSQLError: ${err.message}`);
      }
      throw err;
    } finally {
      await pool.close();
    }
  },
};

export { mssqlConnector };
