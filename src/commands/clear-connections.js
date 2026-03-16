import { Command } from "../deps.js";
import logger from "../logger.js";
import storage from "../scoped-storage.js";

export default new Command()
  .description("Remove all connections")
  .action(({ global: g }) => {
    storage.clear(g);
    logger.info("All connections removed");
  });
