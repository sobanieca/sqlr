import logger from "./logger.js";
import guard from "./guard.js";
import { Secret, Select } from "./deps.js";
import storage from "./scoped-storage.js";
import { getDefaultConnection } from "./scope.js";

const getConnection = async (connectionName, isGlobal) => {
  try {
    const connection = JSON.parse(storage.getItem(connectionName, isGlobal));
    if (!connection) {
      logger.error(
        `Connection ${connectionName} not found. Use get-connections command to list available connections.`,
      );
      Deno.exit(1);
    }

    if (connection.isEncrypted) {
      const envPassword = Deno.env.get("SQLR_ENCRYPTION_PASSWORD");
      const password = envPassword ||
        await Secret.prompt("Provide password used to encrypt connection");
      try {
        connection.connectionString = await guard.decrypt(
          connection.connectionString,
          password,
        );
      } catch (err) {
        logger.debug(err);
        logger.error(
          `Something went wrong when trying to decrypt connection ${connectionName}. Ensure that password you've provided is valid`,
        );
        Deno.exit(1);
      }
    }

    return connection;
  } catch (err) {
    logger.debug(`Error when reading connection ${connectionName}`);
    logger.debug(err);
    logger.error(
      `Unknown error occured when reading connection. Enable debug mode for details.`,
    );
    Deno.exit(1);
  }
};

const getConnectionName = async (isGlobal) => {
  const defaultConnection = getDefaultConnection(isGlobal);
  if (defaultConnection && storage.getItem(defaultConnection, isGlobal)) {
    return defaultConnection;
  }

  const keys = storage.getAllKeys(isGlobal);
  const options = [];

  for (const key of keys) {
    const connection = JSON.parse(storage.getItem(key, isGlobal));
    options.push({ name: connection.name, value: connection.name });
  }

  if (keys.length === 0) {
    throw new Error("No connections defined. Add new connection first.");
  }

  return await Select.prompt({
    message: "Select connection",
    options,
  });
};

export { getConnection, getConnectionName };
