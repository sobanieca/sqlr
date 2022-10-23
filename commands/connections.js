import { Command } from "../deps.js";

export default new Command()
  .option("-a, --add", "Add new connection")
  .option("-r, --remove", "Remove connection")
  .option("-s, --show", "Show connection data")
  .option("-c, --connection", "Name of the connection")
  .action(async ({add, remove, show, connection}) => {
    console.log("Add: " + add);
    console.log("Remove: " + remove);
    console.log("Show: " + show);
    console.log("Connection: " + connection);
  });
