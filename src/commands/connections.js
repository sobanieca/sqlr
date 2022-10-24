import { Command, Input } from "../deps.js";

let showConnection = async (connectionName) => {

};

let removeConnection = async (connectionName) => {

};

let addConnection = async (connectionName) => {
  if (!connectionName) {
    connectionName = await Input.prompt("Provide name of the connection"); 
  }
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
  .option("-c, --connection", "Name of the connection")
  .action(async ({add, remove, show, connection}) => {
    add && await addConnection(connection);
    remove && await removeConnection(connection);
    show && await showConnection(connection);
    
    console.log("Add: " + add);
    console.log("Remove: " + remove);
    console.log("Show: " + show);
    console.log("Connection: " + connection);
  });
