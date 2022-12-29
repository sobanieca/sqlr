import * as log from "log/mod.ts";
import { LogLevels } from "log/mod.ts";
import { BaseHandler } from "log/handlers.ts?s=BaseHandler";
import { bold, brightRed, brightYellow, gray, white } from "fmt/colors.ts";

const colors = {
  gray,
  white,
  brightYellow,
  brightRed,
  bold,
};

class BrightConsoleHandler extends BaseHandler {
  format(logRecord) {
    let msg = super.format(logRecord);

    switch (logRecord.level) {
      case LogLevels.INFO:
        msg = colors.white(msg);
        break;
      case LogLevels.WARNING:
        msg = colors.brightYellow(msg);
        break;
      case LogLevels.ERROR:
        msg = colors.brightRed(msg);
        break;
      case LogLevels.CRITICAL:
        msg = colors.bold(colors.brightRed(msg));
        break;
      default:
        msg = colors.gray(msg);
        break;
    }

    return msg;
  }

  log(msg) {
    console.log(msg);
  }
}

const logLevel = Deno.args.includes("--debug") ? "DEBUG" : "INFO";

await log.setup({
  handlers: {
    console: new BrightConsoleHandler("DEBUG", {
      formatter: "{msg}",
    }),
  },
  loggers: {
    default: {
      level: logLevel,
      handlers: ["console"],
    },
  },
});

const logger = log.getLogger();

export default logger;
