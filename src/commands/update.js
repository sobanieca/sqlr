import { Command } from "../deps.js";
import logger from "../logger.js";

export default new Command()
  .description("Update sqlr to the latest version")
  .option("--deno", "Perform the update using deno installer")
  .action(async ({ deno }) => {
    if (deno) {
      logger.info("Updating sqlr to the latest version...");
      logger.info("");

      const command = new Deno.Command("deno", {
        args: [
          "install",
          "-g",
          "--allow-all",
          "-f",
          "-r",
          "-n",
          "sqlr",
          "jsr:@sobanieca/sqlr",
        ],
        stdout: "inherit",
        stderr: "inherit",
      });

      const { code } = await command.output();

      if (code === 0) {
        logger.info("");
        logger.info("sqlr has been updated successfully!");
      } else {
        logger.error("");
        logger.error("ERROR: Failed to update sqlr");
        Deno.exit(1);
      }
    } else {
      logger.info("To update sqlr to the latest version, run:");
      logger.info("");
      logger.info(
        "  deno install -g --allow-all -f -r -n sqlr jsr:@sobanieca/sqlr",
      );
      logger.info("");
      logger.info("Or use deno updater:");
      logger.info("  sqlr update --deno");
      logger.info(
        "  (This will ask for run permission and perform the update)",
      );
      logger.info("");
      logger.info("For standalone binaries, run the install script:");
      logger.info(
        "  curl -fsSL sobanieca.github.io/sqlr/install.sh | bash",
      );
      logger.info("");
      logger.info("Or download manually from:");
      logger.info(
        "  https://github.com/sobanieca/sqlr/releases/latest",
      );
    }
  });
