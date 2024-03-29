import {
  brightGreen,
  Command,
  EnumType,
  gray,
  PostgresError,
  Table,
} from "../deps.js";
import { connectors } from "../connectors.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { maxTableColumnWidth } from "../const.js";

const runQuery = async (
  query,
  inputFile,
  outputFile,
  connectionName,
  table,
  compact,
  type,
  connectionString,
) => {
  if (!connectionName && !connectionString) {
    connectionName = await getConnectionName();
  }

  let targetType = type;
  let targetConnectionString = connectionString;

  if (connectionName) {
    const connection = await getConnection(connectionName);
    targetConnectionString = connection.connectionString;
    targetType = connection.type;
  }

  if (inputFile && query) {
    throw new Error("Option '--input-file' conflicts with option '--query'.");
  }

  if (inputFile) {
    query = await Deno.readTextFile(inputFile);
  }

  if (!query) {
    throw new Error("No SQL query provided");
  }

  try {
    const startTime = new Date();
    const result = await connectors[targetType].query(
      targetConnectionString,
      query,
    );
    const endTime = new Date();

    logger.info(`${gray("Query executed in:")} ${endTime - startTime}ms`);
    logger.info(`${gray("Rows affected:")} ${result.rowsAffected}`);

    if (result.rows?.length > 0) {
      if (outputFile) {
        await Deno.writeTextFile(
          outputFile,
          JSON.stringify(result.rows, null, 2),
        );
        logger.info(`Results saved to ${outputFile}`);
        return;
      }

      if (table) {
        logger.info(
          new Table()
            .header(
              Object.keys(result.rows[0]).map((column) => brightGreen(column)),
            )
            .body(result.rows.map((row) => Object.values(row)))
            .maxColWidth(maxTableColumnWidth)
            .padding(1)
            .indent(2)
            .border(true)
            .toString(),
        );
        return;
      }

      if (compact) {
        logger.info(brightGreen(Object.keys(result.rows[0]).join(",")));
        result.rows.forEach((row) => logger.info(Object.values(row).join(",")));
        return;
      }

      const jsonResult = Deno.inspect(
        result.rows,
        {
          colors: true,
          strAbbreviateSize: 256000,
          iterableLimit: 20000,
          depth: 100,
        },
      );
      logger.info(jsonResult);
    }
  } catch (err) {
    if (err instanceof PostgresError) {
      logger.error(err.toString());
    } else {
      logger.debug(err);
      logger.error(
        `Error occurred when executing query against database. Ensure that query or connection string is valid. Only simple queries are supported. Use --debug option for details.`,
      );
    }
  }
};

export default new Command()
  .type("ConnectorType", new EnumType(Object.keys(connectors)))
  .arguments("[queryArg]", "SQL query to be executed")
  .option("-q, --query [query]", "SQL query to be executed", { default: "" })
  .option(
    "-i, --input-file [input-file]",
    "Path to input file containing SQL query",
  )
  .option(
    "-o, --output-file [output-file]",
    "Output file where JSON results will be stored",
  )
  .option("-n, --name [name]", "Name of the connection", {
    conflicts: ["type", "connection-string"],
  })
  .option("-t, --type [type:ConnectorType]", "Type of the connection")
  .option("-s, --connection-string [connection-string]", "Connection string")
  .option("--table", "Display results as table", { conflicts: ["compact"] })
  .option("--compact", "Display results in compact form", {
    conflicts: ["table"],
  })
  .description("Run provided SQL query against selected database")
  .description("Run query against specified database")
  .action(
    async (
      {
        query,
        inputFile,
        outputFile,
        name,
        table,
        compact,
        type,
        connectionString,
      },
      queryArg,
    ) => {
      await runQuery(
        query || queryArg,
        inputFile,
        outputFile,
        name,
        table,
        compact,
        type,
        connectionString,
      );
    },
  );
