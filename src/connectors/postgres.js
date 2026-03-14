import { DbClient, Input, Secret, Select } from "../deps.js";
import { DatabaseError } from "../database-error.js";
import { PostgresError } from "jsr:@db/postgres@0.19.5";

const postgresConnector = {
  getDatabaseName: () => "PostgreSQL",
  getConnectionStringHint: () =>
    `
postgres://host:port/database_name?user=user&password=password(urlencoded)&application_name=sqlr
Additional url parameters: 
'sslmode' - require | prefer | disable
'options' - additional values for connection (options=--cluster=your_cluster_name)
More details: https://deno-postgres.com/#/?id=url-parameters
  `.trim(),
  getConnectionString: async () => {
    const host = await Input.prompt(
      "Database host (example: localhost | my.db.com)",
    );
    const port = await Input.prompt("Port (default: 5432)") || 5432;
    const dbName = await Input.prompt("Database name");
    const user = await Input.prompt("Username");
    const password = encodeURIComponent(await Secret.prompt("Password"));
    const sslmode = await Select.prompt({
      message: "SSL mode (default: prefer)",
      options: [
        { name: "prefer", value: "prefer" },
        { name: "require", value: "require" },
        { name: "disable", value: "disable" },
      ],
    });

    return `postgres://${host}:${port}/${dbName}?user=${user}&password=${password}&application_name=sqlr&sslmode=${sslmode}`;
  },
  getTables: async (connectionString) => {
    const dbClient = new DbClient(connectionString);
    await dbClient.connect();

    try {
      const tablesQuery = await dbClient.queryObject(`
        select
          table_schema,
          table_name
        from
          information_schema.tables
        where
          table_schema not in ('pg_catalog', 'information_schema')
        order by
          table_schema, table_name`);

      const tables = tablesQuery.rows;

      const columnsQuery = await dbClient.queryObject(`
        select
          table_schema,
          table_name,
          column_name,
          column_default,
          is_nullable,
          data_type
        from
          information_schema.columns
        where
          table_schema not in ('pg_catalog', 'information_schema')
        order by
          table_schema, table_name, ordinal_position`);

      const columns = columnsQuery.rows;

      const foreignKeysQuery = await dbClient.queryObject(`
        select
          tc.table_schema,
          tc.constraint_name,
          tc.table_name,
          kcu.column_name,
          ccu.table_schema AS foreign_table_schema,
          ccu.table_name AS foreign_table_name,
          ccu.column_name AS foreign_column_name
        from
          information_schema.table_constraints as tc
          join information_schema.key_column_usage as kcu
            on tc.constraint_name = kcu.constraint_name
            and tc.table_schema = kcu.table_schema
          join information_schema.constraint_column_usage as ccu
            on ccu.constraint_name = tc.constraint_name
            and ccu.table_schema = tc.table_schema
        where
          tc.constraint_type = 'FOREIGN KEY'`);

      const foreignKeys = foreignKeysQuery.rows;

      return tables.map((table) => ({
        schema: table["table_schema"],
        name: table["table_name"],
        columns: columns
          .filter((column) =>
            column["table_schema"] == table["table_schema"] &&
            column["table_name"] == table["table_name"]
          )
          .map((column) => ({
            name: column["column_name"],
            type: column["data_type"],
            nullable: column["is_nullable"] == "YES",
            relation: foreignKeys
              .map((foreignKey) => ({
                ...foreignKey,
                relation: `${foreignKey["foreign_table_schema"]}.${
                  foreignKey["foreign_table_name"]
                }.${foreignKey["foreign_column_name"]}`,
              }))
              .find((foreignKey) =>
                foreignKey["table_schema"] == table["table_schema"] &&
                foreignKey["table_name"] == table["table_name"] &&
                foreignKey["column_name"] == column["column_name"]
              )?.relation,
          })),
      }));
    } catch (err) {
      if (err instanceof PostgresError) {
        throw new DatabaseError(err.toString());
      }
      throw err;
    } finally {
      await dbClient.end();
    }
  },
  query: async (connectionString, query) => {
    const dbClient = new DbClient(connectionString);
    await dbClient.connect();

    try {
      const result = await dbClient.queryObject({
        camelcase: false,
        text: query,
      });

      return {
        rowsAffected: result.rowCount,
        rows: result.rows,
      };
    } catch (err) {
      if (err instanceof PostgresError) {
        throw new DatabaseError(err.toString());
      }
      throw err;
    } finally {
      await dbClient.end();
    }
  },
};

export { postgresConnector };
