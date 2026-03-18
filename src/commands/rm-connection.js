import { Command } from "../deps.js";
import logger from "../logger.js";
import { getConnectionName } from "../connection-accessor.js";
import storage from "../scoped-storage.js";

const removeConnection = async (connectionName, isGlobal) => {
  if (!connectionName) {
    connectionName = await getConnectionName(isGlobal);
  }
  storage.removeItem(connectionName, isGlobal);
  logger.info("Connection removed");
};

export default new Command()
  .arguments("[connection:string]")
  .option("-n, --name [name]", "Name of the connection")
  .description("Remove selected connection")
  .action(async ({ name, global: g }, connection) => {
    await removeConnection(name || connection, g);
  });
