import { Input, Secret, Select } from "../deps.js";
import { DatabaseError } from "../database-error.js";
import mysql from "npm:mysql2@3/promise";

const mysqlConnector = {
  getDatabaseName: () => "MySQL",
  getConnectionStringHint: () =>
    `
mysql://user:password@host:port/database_name
Additional url parameters:
'ssl' - {"rejectUnauthorized":true|false}
More details: https://github.com/sidorares/node-mysql2#readme
  `.trim(),
  getConnectionString: async () => {
    const host = await Input.prompt(
      "Database host (default: localhost | example: my.db.com)",
    ) || "localhost";
    const port = await Input.prompt("Port (default: 3306)") || 3306;
    const dbName = await Input.prompt("Database name");
    const user = await Input.prompt("Username");
    const password = encodeURIComponent(await Secret.prompt("Password"));
    const ssl = await Select.prompt({
      message: "SSL mode (default: disabled)",
      options: [
        { name: "disabled", value: "disabled" },
        { name: "enabled (verify certificate)", value: "verify" },
        { name: "enabled (trust any certificate)", value: "trust" },
      ],
    });

    const sslParam = ssl === "verify"
      ? '?ssl={"rejectUnauthorized":true}'
      : ssl === "trust"
      ? '?ssl={"rejectUnauthorized":false}'
      : "";

    return `mysql://${user}:${password}@${host}:${port}/${dbName}${sslParam}`;
  },
  getTables: async (connectionString) => {
    const connection = await mysql.createConnection(connectionString);

    try {
      const url = new URL(connectionString);
      const dbName = url.pathname.replace("/", "");

      const [tables] = await connection.execute(
        `SELECT
          table_schema,
          table_name
        FROM
          information_schema.tables
        WHERE
          table_schema = ?
          AND table_type = 'BASE TABLE'
        ORDER BY
          table_schema, table_name`,
        [dbName],
      );

      const [columns] = await connection.execute(
        `SELECT
          table_schema,
          table_name,
          column_name,
          column_default,
          is_nullable,
          data_type
        FROM
          information_schema.columns
        WHERE
          table_schema = ?
        ORDER BY
          table_schema, table_name, ordinal_position`,
        [dbName],
      );

      const [foreignKeys] = await connection.execute(
        `SELECT
          table_schema,
          table_name,
          column_name,
          referenced_table_schema AS foreign_table_schema,
          referenced_table_name AS foreign_table_name,
          referenced_column_name AS foreign_column_name
        FROM
          information_schema.key_column_usage
        WHERE
          table_schema = ?
          AND referenced_table_name IS NOT NULL`,
        [dbName],
      );

      return tables.map((table) => ({
        schema: table["TABLE_SCHEMA"] || table["table_schema"],
        name: table["TABLE_NAME"] || table["table_name"],
        columns: columns
          .filter((column) =>
            (column["TABLE_SCHEMA"] || column["table_schema"]) ===
              (table["TABLE_SCHEMA"] || table["table_schema"]) &&
            (column["TABLE_NAME"] || column["table_name"]) ===
              (table["TABLE_NAME"] || table["table_name"])
          )
          .map((column) => ({
            name: column["COLUMN_NAME"] || column["column_name"],
            type: column["DATA_TYPE"] || column["data_type"],
            nullable:
              (column["IS_NULLABLE"] || column["is_nullable"]) === "YES",
            relation: foreignKeys
              .map((fk) => ({
                ...fk,
                relation: `${
                  fk["REFERENCED_TABLE_SCHEMA"] || fk["foreign_table_schema"]
                }.${fk["REFERENCED_TABLE_NAME"] || fk["foreign_table_name"]}.${
                  fk["REFERENCED_COLUMN_NAME"] || fk["foreign_column_name"]
                }`,
              }))
              .find((fk) =>
                (fk["TABLE_SCHEMA"] || fk["table_schema"]) ===
                  (table["TABLE_SCHEMA"] || table["table_schema"]) &&
                (fk["TABLE_NAME"] || fk["table_name"]) ===
                  (table["TABLE_NAME"] || table["table_name"]) &&
                (fk["COLUMN_NAME"] || fk["column_name"]) ===
                  (column["COLUMN_NAME"] || column["column_name"])
              )?.relation,
          })),
      }));
    } finally {
      await connection.end();
    }
  },
  query: async (connectionString, query) => {
    const connection = await mysql.createConnection({
      uri: connectionString,
      supportBigNumbers: true,
      bigNumberStrings: true,
    });

    try {
      const [rows, _fields] = await connection.query(query);

      if (Array.isArray(rows)) {
        return {
          rowsAffected: rows.length,
          rows: rows,
        };
      }

      return {
        rowsAffected: rows.affectedRows || 0,
        rows: [],
      };
    } catch (err) {
      if (err.code && err.sqlState) {
        throw new DatabaseError(`MySQL Error: ${err.message}`);
      }
      throw err;
    } finally {
      await connection.end();
    }
  },
};

export { mysqlConnector };
