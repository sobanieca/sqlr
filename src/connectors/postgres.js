import { DbClient } from "../deps.js";

const postgresConnector = {
  getDatabaseName: () => "PostgreSQL",
  getConnectionStringDescription: () =>
    "postgres://host:port/database_name?user=user&password=password&application_name=sqlr",
  getTables: async (connectionString) => {
    // TODO: url encode password! or provide readme details on how to setup connection
    // consider creating connection string builder, or provide some more instructions
    const dbClient = new DbClient(connectionString);
    await dbClient.connect();

    const tablesQuery = await dbClient.queryObject(`
      select 
        table_schema,
        table_name
      from 
        information_schema.tables 
      where 
        table_schema not in ('pg_catalog', 'information_schema')`);

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
        table_schema not in ('pg_catalog', 'information_schema')`);

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

    const result = {};

    result.tables = [];

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
  },
};

export { postgresConnector };
