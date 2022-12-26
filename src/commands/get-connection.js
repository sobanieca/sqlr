import { Command, Input, Select, Table, Toggle } from "../deps.js";
import logger from "../logger.js";
import connectionTypes from "../connection-types.js";
import guard from "../guard.js";

const MIN_PASSWORD_LENGTH = 8;

const validateConnectionName = (connectionName) => {
  let connectionNameRule = /^[A-Za-z0-9]+\-*[A-Za-z0-9]+$/g;

  if (!connectionNameRule.test(connectionName)) {
    logger.error("Invalid connection name, please use only alphanumeric characters and/or at most 1 '-'");
    Deno.exit(1);
  }
};

const getConnectionType = async () => {
  const options = [];
  for(const [key, value] of Object.entries(connectionTypes)) {
    options.push({ name: value.description, value: key });
  }

  return await Select.prompt({
    message: "Select database type",
    options
  });
};

const getConnectionString = async ( connectionType ) => {
  const connectionStringDescription = connectionTypes[connectionType].connectionStringDescription;
  return await Input.prompt(`Provide connection string (${connectionStringDescription})`);
};

const addConnection = async () => {
  const connection = {};
  const connectionName = await Input.prompt("Provide name of the connection"); 

  validateConnectionName(connectionName);

  connection.name = connectionName;

  connection.type = await getConnectionType();
  
  connection.connectionString = await getConnectionString(connection.type);

  connection.isEncrypted = await Toggle.prompt("Do you want to encrypt connection? (Use for Production connections, each time when connection will be used you will need to specify password");

  if (connection.isEncrypted) {
    const password = await Input.prompt("Provide password to encrypt connection");
    if (password && password.length > 8) {
      connection.connectionString = guard.encrypt(password);
    }
    else {
      logger.error(`Please provide password with at least ${MIN_PASSWORD_LENGTH} characters`);
      Deno.exit(1);
    }
  }

  localStorage.setItem(connection.name, JSON.stringify(connection));
};

const showConnection = async () => {
  console.log('show conn');
};

const removeConnection = async () => {
  console.log('remove conn');
};

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
  .option("-a, --add [flag:boolean]", "Add new connection", {
    conflicts: [ "remove", "show" ]
  })
  .option("-r, --remove [flag:boolean]", "Remove connection", {
    conflicts: [ "add", "show" ]
  })
  .option("-s, --show [flag:boolean]", "Show connection data", {
    conflicts: [ "add", "remove"]
  })
  .arguments("[connection]")
  .action(async function ({add, remove, show, connection}) {
    if (!(add || remove || show)) {
      await showAllConnections();
      return;
    }

    add && await addConnection();
    remove && await removeConnection();
    show && await showConnection();
  });
