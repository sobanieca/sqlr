import { Command } from "cliffy/command/mod.ts";
import { Input } from "cliffy/prompt/input.ts";
import { Select } from "cliffy/prompt/select.ts";
import { Toggle } from "cliffy/prompt/toggle.ts";
import logger from "../logger.js";
import connectionTypes from "../connection-types.js";
import guard from "../guard.js";

const getConnectionName = async () => {
  const options = [];

  for(let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
    options.push({ name: connection.name, value: connection.name });
  }

  return await Select.prompt({
    message: "Select connection you want to remove",
    options
  });
};

const removeConnection = async () => {
  const connectionName = await getConnectionName();
  localStorage.removeItem(connectionName);
  logger.info("Connection removed");
};

export default new Command()
  .description("Remove selected connection")
  .action(async function () {
    await removeConnection()
  });
