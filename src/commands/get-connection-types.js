import { Command, Table } from "../deps.js";
import connectionTypes from "../connection-types.js";
import logger from "../logger.js";

const getConnectionTypes = () => {
  logger.info(
    new Table()
      .header(["Type"])
      .body(Object.keys(connectionTypes).map((type) => [type]))
      .maxColWidth(80)
      .padding(1)
      .indent(2)
      .border(true)
      .toString()
  );
};

export default new Command()
  .description(
    "Get available connection types to be used with '--type' parameter in 'add-connection' command"
  )
  .action(() => {
    getConnectionTypes();
  });
