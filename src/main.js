import { Command } from "./deps.js";
import describe from "./commands/describe.js";
import connections from "./commands/connections.js";
import addConnection from "./commands/add-connection.js";
import removeConnection from "./commands/rm-connection.js";
import getConnection from "./commands/get-connection.js";
import query from "./commands/query.js";
import { VERSION } from "./version.js";

await new Command()
  .name("sqlr")
  .version(VERSION)
  .description("Command line for executing SQL queries")
  .action(function() { this.showHelp(); })
  .globalOption("--debug", "Enable debug logs")
  .command("add-connection", addConnection)
  .command("rm-connection", removeConnection)
  .command("get-connection", getConnection)
  .command("connections", connections)
  .command("describe", describe)
  .command("query", query)
  .parse();

