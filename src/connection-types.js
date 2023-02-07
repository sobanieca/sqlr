import { postgresConnector } from "./connectors/postgres.js";

export default {
  postgresql: postgresConnector,
  mssql: {
    getDatabaseName: () => "Microsoft SQL Server",
    getConnectionStringDescription: () => "mssql conn string",
  },
  mysql: {
    getDatabaseName: () => "MySQL",
    getConnectionStringDescription: () => "mysql conn string",
  }
};
