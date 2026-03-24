import { Input } from "../deps.js";
import { DatabaseError } from "../database-error.js";
import { Database } from "jsr:@db/sqlite@0.13";

const isSelectQuery = (query) => {
  const trimmed = query.trim().toUpperCase();
  return trimmed.startsWith("SELECT") ||
    trimmed.startsWith("WITH") ||
    trimmed.startsWith("PRAGMA") ||
    trimmed.startsWith("EXPLAIN");
};

const sqliteConnector = {
  getDatabaseName: () => "SQLite",
  getConnectionStringHint: () =>
    `
/path/to/database.db
Use ':memory:' for an in-memory database
More details: https://www.sqlite.org/uri.html
  `.trim(),
  getConnectionString: async () => {
    const dbPath = await Input.prompt(
      "Database file path (example: /data/mydb.sqlite | :memory:)",
    );
    return dbPath;
  },
  getTables: (connectionString) => {
    const db = new Database(connectionString, { readonly: true });

    try {
      const tables = db.prepare(
        `SELECT name AS table_name
        FROM sqlite_master
        WHERE type = 'table'
          AND name NOT LIKE 'sqlite_%'
        ORDER BY name`,
      ).all();

      return tables.map((table) => {
        const columns = db.prepare(
          `PRAGMA table_info('${table.table_name}')`,
        ).all();

        const foreignKeys = db.prepare(
          `PRAGMA foreign_key_list('${table.table_name}')`,
        ).all();

        return {
          schema: "main",
          name: table.table_name,
          columns: columns.map((column) => ({
            name: column.name,
            type: column.type || "ANY",
            nullable: column.notnull === 0 && column.pk === 0,
            relation: foreignKeys
              .filter((fk) => fk.from === column.name)
              .map((fk) => `main.${fk.table}.${fk.to}`)[0],
          })),
        };
      });
    } catch (err) {
      if (err.code && err.code.startsWith("SQLITE_")) {
        throw new DatabaseError(`SQLite Error: ${err.message}`);
      }
      throw err;
    } finally {
      db.close();
    }
  },
  query: (connectionString, sql) => {
    const db = new Database(connectionString);

    try {
      if (isSelectQuery(sql)) {
        const rows = db.prepare(sql).all();
        return {
          rowsAffected: rows.length,
          rows,
        };
      }

      const result = db.prepare(sql).run();
      return {
        rowsAffected: result,
        rows: [],
      };
    } catch (err) {
      if (err.code && err.code.startsWith("SQLITE_")) {
        throw new DatabaseError(`SQLite Error: ${err.message}`);
      }
      throw err;
    } finally {
      db.close();
    }
  },
};

export { sqliteConnector };
