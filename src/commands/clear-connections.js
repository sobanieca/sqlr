import { Command } from "../deps.js";
import logger from "../logger.js";

export default new Command()
  .description("Remove all connections")
  .action(() => {
    localStorage.clear();
    logger.info("All connections removed");
  });
