import { Command } from "../deps.js";
import { getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import storage from "../scoped-storage.js";
import { setDefaultConnection, setGlobalMode } from "../scope.js";

const setConnection = async (connectionName, isGlobal) => {
  if (isGlobal) {
    setGlobalMode(true);
  }

  if (!connectionName) {
    connectionName = await getConnectionName(isGlobal);
  }

  if (!storage.getItem(connectionName, isGlobal)) {
    logger.error(
      `Connection ${connectionName} not found. Use get-connections command to list available connections.`,
    );
    Deno.exit(1);
  }

  setDefaultConnection(connectionName, isGlobal);
  logger.info(`Default connection set to ${connectionName}`);
};

export default new Command()
  .arguments("[connection:string]")
  .option("-n, --name [name]", "Name of the connection")
  .description("Set default connection for current scope")
  .action(async ({ name, global: g }, connection) => {
    await setConnection(name || connection, g);
  });
