import { Command, Table } from "../deps.js";
import logger from "../logger.js";
import { styledName } from "../connection-style.js";

const showAllConnections = () => {
  const connections = [];

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
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
  .action(showAllConnections);
