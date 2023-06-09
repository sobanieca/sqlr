import { postgresConnector } from "./connectors/postgres.js";

export default {
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
