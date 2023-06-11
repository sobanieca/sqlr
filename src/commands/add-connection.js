import { Command, EnumType, Input, Secret, Select, Toggle } from "../deps.js";
import logger from "../logger.js";
import { connectors } from "../connectors.js";
import guard from "../guard.js";

const MIN_PASSWORD_LENGTH = 8;

const validateConnectionName = (connectionName) => {
  const connectionNameRule = /^[A-Za-z0-9]+\-*[A-Za-z0-9]+$/g;

  if (!connectionNameRule.test(connectionName)) {
    logger.error(
      "Invalid connection name, please use only alphanumeric characters and/or at most 1 '-'",
    );
    Deno.exit(1);
  }

  if (localStorage.getItem(connectionName)) {
    logger.error("Invalid connection name. Such connection already exists.");
    Deno.exit(1);
  }
};

const getConnectionType = async () => {
  const options = [];
  for (const [key, value] of Object.entries(connectors)) {
    options.push({ name: value.getDatabaseName(), value: key });
  }

  return await Select.prompt({
    message: "Select database type",
    options,
  });
};

const getConnectionString = async (connectionType) => {
  logger.info("Provide connection details");
  return await connectors[connectionType].getConnectionString();
};

const addConnection = async (name, type, connectionString) => {
  const connection = {};
  const connectionName = name ||
    await Input.prompt("Provide name of the connection");

  validateConnectionName(connectionName);

  connection.name = connectionName;

  connection.type = type || await getConnectionType();

  connection.connectionString = connectionString ||
    await getConnectionString(connection.type);

  connection.isEncrypted = connectionString ? false : await Toggle.prompt(
    "Do you want to encrypt connection? (Use for Production connections, each time when connection will be used you will need to specify password",
  );

  if (connection.isEncrypted) {
    const password = await Secret.prompt(
      "Provide password to encrypt connection",
    );
    if (password && password.length > MIN_PASSWORD_LENGTH) {
      connection.connectionString = await guard.encrypt(
        connection.connectionString,
        password,
      );
    } else {
      logger.error(
        `Please provide password with at least ${MIN_PASSWORD_LENGTH} characters`,
      );
      Deno.exit(1);
    }
  }

  localStorage.setItem(connection.name, JSON.stringify(connection));
};

export default new Command()
  .type("ConnectorType", new EnumType(Object.keys(connectors)))
  .option("-n, --name [name]", "Name of the connection")
  .option("-t, --type [type:ConnectorType]", "Type of the connection")
  .option("-s, --connection-string [connection-string]", "Connection string")
  .description("Add new connection. Run without parameters to use wizard.")
  .meta(
    "Connection Types",
    "Available types and connection string hints can be found using 'get-connection-types' command",
  )
  .action(async function ({ name, type, connectionString }) {
    await addConnection(name, type, connectionString);
  });
