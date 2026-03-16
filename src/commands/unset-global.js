import { Command } from "../deps.js";
import logger from "../logger.js";
import { setGlobalMode } from "../scope.js";

export default new Command()
  .description("Switch back to repository-scoped mode for connections")
  .action(() => {
    setGlobalMode(false);
    logger.info(
      "Global mode disabled. Connection commands will use repository scope.",
    );
  });
