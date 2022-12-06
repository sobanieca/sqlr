import { logger } from "./deps.js";

const getConnection = (connectionName) => {

  try {
    const connection = JSON.parse(localStorage.getItem(connectionName));

    if (connection.isEncrypted) {
      // TODO: decrypt connection string
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
