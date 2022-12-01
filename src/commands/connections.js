import { Command, Input, Select, Toggle } from "../deps.js";
import logger from "../logger.js";
import connectionTypes from "../connection-types.js";

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

const addConnection = async (connectionName) => {
  const connection = {};

  if (!connectionName) {
    connectionName = await Input.prompt("Provide name of the connection"); 
  }

  validateConnectionName(connectionName);

  connection.name = connectionName;

  connection.type = await getConnectionType();
  
  connection.connectionString = await getConnectionString(connection.type);

  connection.isEncrypted = await Toggle.prompt("Do you want to encrypt connection? (Use for Production connections, each time when connection will be used you will need to specify password");

  if (connection.isEncrypted) {
    const password = await Input.prompt("Provide password to encrypt connection");
    // TODO: encrypt (create new instance of AES (password) and then run encrypt), Ideally, encryption should be moved
    // to separate service
    connection.connectionString = aes.encrypt("");
  }
  console.log(connection);

  // TODO: add to localStorage
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
