import { Command } from "../deps.js";
import logger from "../logger.js";
import { setGlobalMode } from "../scope.js";

export default new Command()
  .description("Switch to global mode (ignore git repository scope by default)")
  .action(() => {
    setGlobalMode(true);
    logger.info(
      "Global mode enabled. All connection commands will use global scope by default.",
    );
  });
