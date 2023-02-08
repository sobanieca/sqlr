import { DbClient } from "../deps.js";

const postgresConnector = {
  getDatabaseName: () => "PostgreSQL",
  getConnectionStringDescription: () => "driver://host:port/database_name?user=user&password=password&application_name=my_app",
  getDescription: async (connectionString) => {
    // TODO: url encode password!
    const dbClient = new DbClient(connectionString);
    await dbClient.connect();

    const tables = await dbClient.queryArray('select * from information_schema.tables where table_schema not in (\'pg_catalog\', \'information_schema\')');
    console.log(tables)

    await Promise.resolve();
    console.log("Postgres connector running describe command with conn string:");
    console.log(connectionString);

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
