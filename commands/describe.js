import { Command } from "../deps.js";

export default new Command()
  .description("Describe all tables and columns available in database")
  .option("-s, --connection-string", "Connection string", {
    conflicts: [ "connection" ]
  })
  .option("-c, --connection", "Name of the connection", {
    conflicts: [ "connection-string" ]
  })
  .option("--compact", "Show every table in one line")
  .action(async ({ connection, connectionString, compact }) => {
    console.log("Connection: " + connection);
    console.log("Connection string: " + connectionString);
    console.log("Compact: " + compact);
  });
