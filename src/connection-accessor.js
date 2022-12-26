import logger from "./logger.js";
import guard from "./guard.js";
import { Secret } from "cliffy/prompts/secret.ts";

const getConnection = async (connectionName) => {

  try {
    const connection = JSON.parse(localStorage.getItem(connectionName));

    if (connection.isEncrypted) {
      const password = await Secret.prompt("Provide password used to encrypt connection");
      connection.connectionString = guard.decrypt(connection.connectionString, password);
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
