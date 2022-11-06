import { log, LogLevels, BaseHandler, colors } from "./deps.js";

class BrightConsoleHandler extends BaseHandler {
  format(logRecord) {

    let msg = super.format(logRecord);

    switch (logRecord.level) {
      case LogLevels.INFO:
        msg = colors.brightBlue(msg);
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
