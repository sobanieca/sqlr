import logger from "./logger.js";
import guard from "./guard.js";
import { Secret } from "cliffy/prompt/secret.ts";

const getConnection = async (connectionName) => {

  try {
    const connection = JSON.parse(localStorage.getItem(connectionName));

    if (connection.isEncrypted) {
      const password = await Secret.prompt("Provide password used to encrypt connection");
      try {
        connection.connectionString = await guard.decrypt(connection.connectionString, password);
      } catch(err) {
        logger.debug(err);
        logger.error(`Something went wrong when trying to decrypt connection ${connectionName}. Ensure that password you've provided is valid`);
        Deno.exit(1);
      }
    }

    return connection;
  }
  catch(err) {
    logger.debug(`Error when reading connection ${connectionName}`);
    logger.debug(err);
    logger.error(`Connection ${connectionName} not found. Use connections command to list available connection names.`);
  }
}

export { getConnection }
