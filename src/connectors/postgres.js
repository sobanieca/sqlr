import { DbClient } from "../deps.js";

const postgresConnector = {
  getDatabaseName: () => "PostgreSQL",
  getConnectionStringDescription: () => "postgres://host:port/database_name?user=user&password=password&application_name=my_app",
  getDescription: async (connectionString) => {
    // TODO: url encode password!
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
    

    // TODO: remove console.log
    console.log(tables);
    console.log(columns);
    console.log(foreignKeys);

    const result = {};

    result.tables = [];

    return {
      tables: [
        {
          schema: "dbo",
          name: "orders",
          columns: [
            {
              name: "id",
              type: "bigint",
              nullable: false,
            },
            {
              name: "user_id",
              relation: "dbo.users.id"
            },
            {
              name: "created_on",
              type: "datetime",
              nullable: true
            }
          ]
        },
        {
          schema: "invoices",
          name: "documents",
          columns: [ { name: "id", type: "bigint" } ]
        }
      ]
    }
  }
}

export { postgresConnector }
