import { Command } from "../deps.js";
import logger from "../logger.js";
import { getConnectionName } from "../connection-accessor.js";

const removeConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }
  localStorage.removeItem(connectionName);
  logger.info("Connection removed");
};

export default new Command()
  .arguments("[connection]")
  .description("Remove selected connection")
  .action(async (_, connection) => {
    await removeConnection(connection);
  });
