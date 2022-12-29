import { Command } from "cliffy/command/mod.ts";
import { Table } from "cliffy/table/mod.ts";
import { getConnection } from "../connection-accessor.js";
import logger from "../logger.js";
import { Select } from "cliffy/prompt/select.ts";

const getConnectionName = async () => {
  const options = [];

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
    options.push({ name: connection.name, value: connection.name });
  }

  return await Select.prompt({
    message: "Select connection you want to view",
    options,
  });
};

const showConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }
  const connection = await getConnection(connectionName);

  logger.info(
    new Table()
      .header(["Name", "Type", "Connection string"])
      .body([[connection.name, connection.type, connection.connectionString]])
      .maxColWidth(80)
      .padding(1)
      .indent(2)
      .border(true)
      .toString(),
  );
};

export default new Command()
  .arguments("[connection-name]")
  .description("Get details of selected connection")
  .action(async (_, connectionName) => {
    await showConnection(connectionName);
  });
