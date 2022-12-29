import { Command } from "cliffy/command/mod.ts";
import { Select } from "cliffy/prompt/select.ts";
import logger from "../logger.js";

const getConnectionName = async () => {
  const options = [];

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
    options.push({ name: connection.name, value: connection.name });
  }

  return await Select.prompt({
    message: "Select connection you want to remove",
    options,
  });
};

const removeConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }
  localStorage.removeItem(connectionName);
  logger.info("Connection removed");
};

export default new Command()
  .arguments("[connection-name]")
  .description("Remove selected connection")
  .action(async (_, connectionName) => {
    await removeConnection(connectionName);
  });
