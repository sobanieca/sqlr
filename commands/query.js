import { Command } from "../deps.js";

export default new Command()
  .option("-s, --show", "Show connection data")
  .option("-c, --connection", "Name of the connection")
  .option("-q, --query", "Query")
  .option("-i, --input", "Input SQL file")
  .global()
  .action(async ({show, connection}) => {
    console.log("Show: " + show);
    console.log("Connection: " + connection);
  });
