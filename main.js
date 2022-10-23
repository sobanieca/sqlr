import { Command } from "./deps.js";

import describe from "./commands/describe.js";
import connections from "./commands/connections.js";
import query from "./commands/query.js";

await new Command()
  .name("sqlr")
  .version("0.1.0")
  .description("Command line for executing SQL queries")
  .action(function() { this.showHelp(); })
  .globalOption("--debug", "Enable debug logs")
  .command("connections", connections)
  .command("describe", describe)
  .command("query", query)
  .parse();

