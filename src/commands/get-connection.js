import { Command, Table } from "../deps.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { maxTableColumnWidth } from "../const.js";

const showConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }
  const connection = await getConnection(connectionName);

  logger.info(
    new Table()
      .header(["Name", "Type", "Connection string"])
      .body([[connection.name, connection.type, connection.connectionString]])
      .maxColWidth(maxTableColumnWidth)
      .padding(1)
      .indent(2)
      .border(true)
      .toString(),
  );
};

export default new Command()
  .arguments("[connection]", "Name of the connection")
  .option("-n, --name [name]", "Name of the connection")
  .description("Get details of selected connection")
  .action(async ({ name }, connection) => {
    await showConnection(name || connection);
  });
