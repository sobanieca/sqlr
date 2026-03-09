import { postgresConnector } from "./connectors/postgres.js";
import { mysqlConnector } from "./connectors/mysql.js";
import { mssqlConnector } from "./connectors/mssql.js";
import { clickhouseConnector } from "./connectors/clickhouse.js";

export const connectors = {
  postgresql: postgresConnector,
  mysql: mysqlConnector,
  mssql: mssqlConnector,
  clickhouse: clickhouseConnector,
};
