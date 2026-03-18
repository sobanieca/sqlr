import { Command, Table } from "../deps.js";
import logger from "../logger.js";
import { styledName } from "../connection-style.js";
import storage from "../scoped-storage.js";

const showAllConnections = (isGlobal) => {
  const keys = storage.getAllKeys(isGlobal);
  const connections = [];

  for (const key of keys) {
    const connection = JSON.parse(storage.getItem(key, isGlobal));
    connections.push([styledName(connection), connection.type]);
  }

  logger.info(
    new Table()
      .header(["Name", "Type"])
      .body(connections)
      .maxColWidth(25)
      .padding(1)
      .indent(2)
      .border(true)
      .toString(),
  );
};

export default new Command()
  .description("List all defined connections")
  .action(({ global: g }) => showAllConnections(g));
