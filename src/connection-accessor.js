import logger from "./logger.js";
import guard from "./guard.js";
import { Secret, Select } from "./deps.js";

const getConnection = async (connectionName) => {
  try {
    const connection = JSON.parse(localStorage.getItem(connectionName));
    if (!connection) {
      logger.error(
        `Connection ${connectionName} not found. Use connections command to list available connections.`,
      );
      Deno.exit(1);
    }

    if (connection.isEncrypted) {
      const password = await Secret.prompt(
        "Provide password used to encrypt connection",
      );
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

const getConnectionName = async () => {
  const options = [];

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
    options.push({ name: connection.name, value: connection.name });
  }

  if(localStorage.length == 0) throw new Error("No connections defined. Add new connection first.");

  return await Select.prompt({
    message: "Select connection you want to remove",
    options,
  });
};

export { getConnection, getConnectionName };
