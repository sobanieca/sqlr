import { Command } from "cliffy/command/mod.ts";

export default new Command()
  .option("-s, --show", "Show connection data")
  .option("-c, --connection", "Name of the connection")
  .option("-q, --query", "Query", {
    conflicts: ["input"],
  })
  .option("-i, --input", "Input SQL file", {
    conflicts: ["query"],
  })
  .action(async ({ show, connection }) => {
    console.log("Show: " + show);
    console.log("Connection: " + connection);
  });
