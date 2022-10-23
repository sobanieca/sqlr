import { Command } from "./deps.js";

import describe from "./commands/describe.js";
import connections from "./commands/connections.js";
import query from "./commands/query.js";

await new Command()
  .name("sqlr")
  .version("0.1.0")
  .description("Command line for executing SQL queries")
  .command("connections", connections)
  .command("describe", describe)
  .command("query", query)
  .globalOption("--debug", "Enable debug logs")
  .parse(Deno.args);
//await new Command()
//  .name("sqlr")
//  .version("0.1.0")
//  .description("Command line for executing SQL queries")
//  .option("-c, --connection <connection:string>", "Connection to be used")
//  .option("-s, --connection-string <connection-string:string>", "Connection string to be used")
//  .option("-q, --query <query:string>", "SQL query to be used")
//  .option("-i, --input <input:string>", "Input SQL file")
//  .option("-o, --output <output:string>", "Output SQL file")
//  .command("describe", "Describe database tables and columns")
//  .option("-c, --connection <connection:string>", "Connection to be used")
//  .option("-s, --connection-string <connection-string:string>", "Connection string to be used")
//  .option("--compact", "Output each table in single line")
//  .command(connections)
//  .globalOption("--debug", "Enable debug logs")
//  .parse(Deno.args);

