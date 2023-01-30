import { Command, Table } from "../deps.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";

const showConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }
  const connection = await getConnection(connectionName);

  logger.info(
    new Table()
      .header(["Name", "Type", "Connection string"])
      .body([[connection.name, connection.type, connection.connectionString]])
      .maxColWidth(80)
      .padding(1)
      .indent(2)
      .border(true)
      .toString(),
  );
};

export default new Command()
  .arguments("[connection]")
  .description("Get details of selected connection")
  .action(async (_, connection) => {
    await showConnection(connection);
  });
