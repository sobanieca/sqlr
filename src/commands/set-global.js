import { Command } from "../deps.js";
import logger from "../logger.js";
import { setGlobalMode } from "../scope.js";

export default new Command()
  .description(
    "Toggle global mode (true = ignore git repository scope, false = use repository scope)",
  )
  .arguments("<enabled:boolean>")
  .action((_options, enabled) => {
    setGlobalMode(enabled);
    if (enabled) {
      logger.info(
        "Global mode enabled. All connection commands will use global scope by default.",
      );
    } else {
      logger.info(
        "Global mode disabled. Connection commands will use repository scope.",
      );
    }
  });
