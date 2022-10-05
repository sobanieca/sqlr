import { Command } from "https://deno.land/x/cliffy@v0.25.2/command/mod.ts";

await new Command()
  .name("sqlr")
  .version("0.1.0")
  .description("Command line for executing SQL queries")
  .option("-c, --connection <connection:string>", "Connection to be used")
  .globalOption("--debug", "Enable debug logs")
  .parse(Deno.args);


