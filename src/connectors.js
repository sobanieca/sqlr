import { postgresConnector } from "./connectors/postgres.js";

export const connectors = {
  postgresql: postgresConnector,
  mssql: {
    getDatabaseName: () => "Microsoft SQL Server",
    getConnectionStringHint: () => "mssql conn string",
  },
  mysql: {
    getDatabaseName: () => "MySQL",
    getConnectionStringHint: () => "mysql conn string",
  },
};

