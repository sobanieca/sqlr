import { postgresConnector } from "./connectors/postgres.js";

export default {
  postgresql: {
    description: "PostgreSQL",
    connectionStringDescription:
    "driver://host:port/database_name?user=user&password=password&application_name=my_app",
    connector: postgresConnector
  },
  mssql: {
    description: "Microsof SQL Server",
    connectionStringDescription: "",
  },
  mysql: {
    description: "MySQL",
    connectionStringDescription: "",
  },
};
