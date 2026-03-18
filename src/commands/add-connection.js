import { Command, EnumType, Input, Secret, Select, Toggle } from "../deps.js";
import logger from "../logger.js";
import { connectors } from "../connectors.js";
import guard from "../guard.js";
import { colors, promptColor, promptEmoji } from "../connection-style.js";
import storage from "../scoped-storage.js";

const MIN_PASSWORD_LENGTH = 8;

const validateConnectionName = (connectionName, isGlobal) => {
  const connectionNameRule = /^[A-Za-z0-9]+\-*[A-Za-z0-9]+$/g;

  if (!connectionNameRule.test(connectionName)) {
    logger.error(
      "Invalid connection name, please use only alphanumeric characters and/or at most 1 '-'",
    );
    Deno.exit(1);
  }

  if (storage.getItem(connectionName, isGlobal)) {
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

const addConnection = async (
  name,
  type,
  connectionString,
  color,
  emoji,
  isGlobal,
) => {
  const connection = {};
  const connectionName = name ||
    await Input.prompt("Provide name of the connection");

  validateConnectionName(connectionName, isGlobal);

  connection.name = connectionName;

  connection.type = type || await getConnectionType();

  connection.connectionString = connectionString ||
    await getConnectionString(connection.type);

  connection.color = color || (connectionString ? "none" : await promptColor());
  connection.emoji = emoji || (connectionString ? "none" : await promptEmoji());

  const envPassword = Deno.env.get("SQLR_ENCRYPTION_PASSWORD");

  if (connectionString && envPassword) {
    connection.isEncrypted = true;
  } else {
    connection.isEncrypted = connectionString ? false : await Toggle.prompt(
      "Do you want to additionally secure connection settings? (Use for Production connections, each time when connection will be used you will need to specify password.",
    );
  }

  if (connection.isEncrypted) {
    const password = envPassword ||
      await Secret.prompt("Provide password to encrypt connection");
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

  storage.setItem(connection.name, JSON.stringify(connection), isGlobal);
  logger.info("Connection has been added");
};

const colorValues = colors.map((c) => c.value).concat("none");

export default new Command()
  .type("ConnectorType", new EnumType(Object.keys(connectors)))
  .type("ConnectionColor", new EnumType(colorValues))
  .option("-n, --name [name]", "Name of the connection")
  .option("-t, --type [type:ConnectorType]", "Type of the connection")
  .option("-s, --connection-string [connection-string]", "Connection string")
  .option("-c, --color [color:ConnectionColor]", "Connection color")
  .option("-e, --emoji [emoji]", "Connection emoji")
  .description("Add new connection. Run without parameters to use wizard.")
  .meta(
    "Connection Types",
    "Available types and connection string hints can be found using 'ls-types' command",
  )
  .action(
    async function ({ name, type, connectionString, color, emoji, global: g }) {
      await addConnection(name, type, connectionString, color, emoji, g);
    },
  );
