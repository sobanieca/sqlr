import { Command, Table } from "../deps.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { maxTableColumnWidth } from "../const.js";
import { styledName } from "../connection-style.js";

const showConnection = async (connectionName, isGlobal) => {
  if (!connectionName) {
    connectionName = await getConnectionName(isGlobal);
  }
  const connection = await getConnection(connectionName, isGlobal);

  logger.info(
    new Table()
      .header(["Name", "Type", "Connection string"])
      .body([[
        styledName(connection),
        connection.type,
        decodeURIComponent(connection.connectionString),
      ]])
      .maxColWidth(maxTableColumnWidth)
      .padding(1)
      .indent(2)
      .border(true)
      .toString(),
  );
};

export default new Command()
  .arguments("[connection:string]")
  .option("-n, --name [name]", "Name of the connection")
  .description("Get details of selected connection")
  .action(async ({ name, global: g }, connection) => {
    await showConnection(name || connection, g);
  });
