import { Command, Input } from "../deps.js";
import logger from "../logger.js";
import connectioTypes from "../connection-types.js";

const validateConnectionName = (connectionName) => {
  let connectionNameRule = /^[A-Za-z0-9]+\-*[A-Za-z0-9]+$/g;

  if (!connectionNameRule.test(connectionName)) {
    logger.error("Invalid connection name, please use only alphanumeric characters and/or at most 1 '-'");
    Deno.exit(1);
  }
};

const addConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await Input.prompt("Provide name of the connection"); 
  }

  validateConnectionName(connectionName);



  let connection = {
    name: connectionName
  }
};

const showConnection = async (connectionName) => {

};

const removeConnection = async (connectionName) => {

};

const showAllConnections = async () => {
};

export default new Command()
  .option("-a, --add [flag:boolean]", "Add new connection", {
    conflicts: [ "remove", "show" ]
  })
  .option("-r, --remove [flag:boolean]", "Remove connection", {
    conflicts: [ "add", "show" ]
  })
  .option("-s, --show [flag:boolean]", "Show connection data", {
    conflicts: [ "add", "remove"]
  })
  .option("-c, --connection [value:string]", "Name of the connection")
  .action(async function ({add, remove, show, connection}) {
    if (!(add || remove || show)) {
      showAllConnections();
      return;
    }

    add && await addConnection(connection);
    remove && await removeConnection(connection);
    show && await showConnection(connection);
  });
