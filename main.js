import { Command } from "./src/deps.js";
import describe from "./src/commands/describe.js";
import connections from "./src/commands/connections.js";
import addConnection from "./src/commands/add-connection.js";
import removeConnection from "./src/commands/rm-connection.js";
import getConnection from "./src/commands/get-connection.js";
import getConnectionTypes from "./src/commands/get-connection-types.js";
import query from "./src/commands/query.js";
import { VERSION } from "./src/version.js";
import logger from "./src/logger.js";

logger.debug("Debug mode enabled");

try {
  await new Command()
    .name("sqlr")
    .version(VERSION)
    .description(
      "Command line for interacting with SQL databases. Use '--help' for each command to list it's parameters",
    )
    .action(function () {
      this.showHelp();
    })
    .globalOption("--debug", "Enable debug logs")
    .command("add-connection", addConnection)
    .command("rm-connection", removeConnection)
    .command("get-connection", getConnection)
    .command("get-connection-types", getConnectionTypes)
    .command("connections", connections)
    .command("describe", describe)
    .command("query", query)
    .parse();
} catch (err) {
  logger.error(err.message);
  logger.debug(err);
}
