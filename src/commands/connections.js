import { Command, Input, Select, Table, Toggle } from "../deps.js";
import logger from "../logger.js";
import connectionTypes from "../connection-types.js";
import guard from "../guard.js";

const showAllConnections = async () => {
  let connections = [];

  for(let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    const connection = JSON.parse(localStorage.getItem(key));
    connections.push([ connection.name, connection.type ]);
  }

  logger.info(new Table()
    .header(["Name", "Type" ])
    .body(connections)
    .maxColWidth(25)
    .padding(1)
    .indent(2)
    .border(true)
    .toString());
};

export default new Command()
  .action(async () => await showAllConnections());
