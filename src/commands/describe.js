import { Command } from "cliffy/command/mod.ts";

export default new Command()
  .description("Describe all tables and columns available in database")
  .option("-s, --connection-string [value:string]", "Connection string", {
    conflicts: [ "connection" ]
  })
  .option("-c, --connection [value:string]", "Name of the connection", {
    conflicts: [ "connection-string" ]
  })
  .option("--compact [flag:boolean]", "Show every table in one line")
  .action(async ({ connection, connectionString, compact }) => {
    console.log("Connection: " + connection);
    console.log("Connection string: " + connectionString);
    console.log("Compact: " + compact);
  });
