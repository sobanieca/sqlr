/* @ts-self-types="./main.d.ts" */
import { Command } from "./src/deps.js";
import describe from "./src/commands/describe.js";
import connections from "./src/commands/connections.js";
import addConnection from "./src/commands/add-connection.js";
import removeConnection from "./src/commands/rm-connection.js";
import getConnection from "./src/commands/get-connection.js";
import getConnectionTypes from "./src/commands/get-connection-types.js";
import query from "./src/commands/query.js";
import update from "./src/commands/update.js";
import clearConnections from "./src/commands/clear-connections.js";
import setConnection from "./src/commands/set-connection.js";
import setGlobal from "./src/commands/set-global.js";
import unsetGlobal from "./src/commands/unset-global.js";
import helpText from "./src/commands/help.js";
import { version } from "./src/version.js";
import logger from "./src/logger.js";

logger.debug("Debug mode enabled");

try {
  const cmd = new Command()
    .name("sqlr")
    .version(version)
    .description(
      "Command line for interacting with SQL databases. Use '--help' for each command to list it's parameters",
    )
    .action(function () {
      this.showHelp();
    })
    .globalOption("--debug", "Enable debug logs")
    .globalOption(
      "-g, --global",
      "Use global scope for connections (ignore git repository scope)",
    )
    .command("add", addConnection)
    .command("rm", removeConnection)
    .command("get", getConnection)
    .command("ls-types", getConnectionTypes)
    .command("ls", connections)
    .command("clear", clearConnections)
    .command("set", setConnection)
    .command("set-global", setGlobal)
    .command("unset-global", unsetGlobal)
    .command("describe", describe)
    .command("query", query)
    .command("update", update)
    .command(
      "help",
      new Command()
        .description("Display detailed help text with usage instructions")
        .action(() => {
          console.log(helpText.help);
        }),
    );

  const knownCommands = new Set(cmd.getCommands().map((c) => c.getName()));
  const firstArg = Deno.args[0];
  const isImplicitQuery = firstArg &&
    !firstArg.startsWith("-") &&
    !knownCommands.has(firstArg);
  const args = isImplicitQuery ? ["query", ...Deno.args] : Deno.args;

  await cmd.parse(args);
} catch (err) {
  logger.error(err.message);
  logger.debug(err);
}
