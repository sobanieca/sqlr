import { brightGreen, Command, EnumType, EOL, gray, Table } from "../deps.js";
import { connectors } from "../connectors.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { maxTableColumnWidth } from "../const.js";
import { DatabaseError } from "../database-error.js";
import { formatConnectionName } from "../connection-style.js";

const parseInputVariables = (vars) => {
  const result = {};
  if (!vars) return result;
  for (const item of vars) {
    const colonIndex = item.indexOf(":");
    if (colonIndex === -1) {
      throw new Error(
        `Invalid input variable format: "${item}". Expected "key: value".`,
      );
    }
    const key = item.substring(0, colonIndex).trim();
    const value = item.substring(colonIndex + 1).trim();
    result[key] = value;
  }
  return result;
};

const replaceVariables = (query, variables, ignoreInputValidation) => {
  let result = query;
  for (const [key, value] of Object.entries(variables)) {
    result = result.replaceAll(`{{${key}}}`, value);
  }

  if (!ignoreInputValidation) {
    const unreplaced = result.match(/\{\{([^}]+)\}\}/g);
    if (unreplaced) {
      const names = [...new Set(unreplaced.map((m) => m.slice(2, -2)))];
      throw new Error(
        `Missing input variables: ${
          names.join(", ")
        }. Use -i "name: value" to provide them.`,
      );
    }
  }

  return result;
};

const isFilePath = async (input) => {
  try {
    const stat = await Deno.stat(input);
    return stat.isFile;
  } catch {
    return false;
  }
};

const runQuery = async (
  queryArg,
  inputVariables,
  outputFile,
  connectionName,
  table,
  compact,
  type,
  connectionString,
  ignoreInputValidation,
  isGlobal,
) => {
  if (!connectionName && !connectionString) {
    connectionName = await getConnectionName(isGlobal);
  }

  let targetType = type;
  let targetConnectionString = connectionString;

  if (connectionName) {
    const connection = await getConnection(connectionName, isGlobal);
    logger.info(formatConnectionName(connection));
    targetConnectionString = connection.connectionString;
    targetType = connection.type;
  }

  if (!queryArg) {
    throw new Error("No SQL query provided");
  }

  let query = queryArg;
  const isFile = await isFilePath(queryArg);
  if (isFile) {
    query = await Deno.readTextFile(queryArg);
  }

  const variables = parseInputVariables(inputVariables);
  query = replaceVariables(query, variables, ignoreInputValidation);

  if (isFile) {
    const lines = query.split(EOL);
    const preview = lines.length > 10
      ? [...lines.slice(0, 10), "..."].join(EOL)
      : query;
    logger.info(`${gray("SQL to execute:")}\n${preview}`);
  }

  try {
    const startTime = Date.now();
    const result = await connectors[targetType].query(
      targetConnectionString,
      query,
    );
    const endTime = Date.now();

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
    if (err instanceof DatabaseError) {
      logger.error(err.message);
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
  .arguments("[queryArg:string]")
  .option(
    "-i, --input-variable <var:string>",
    'Input variable in "key: value" format for {{key}} substitution in SQL',
    { collect: true },
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
  .option(
    "--ignore-input-validation",
    "Skip validation for missing input variables, allowing {{handlebars}} syntax to pass through to the database",
  )
  .description("Run query against specified database")
  .action(
    async (
      {
        inputVariable,
        outputFile,
        name,
        table,
        compact,
        type,
        connectionString,
        ignoreInputValidation,
        global: g,
      },
      queryArg,
    ) => {
      await runQuery(
        queryArg,
        inputVariable,
        outputFile,
        name,
        table,
        compact,
        type,
        connectionString,
        ignoreInputValidation,
        g,
      );
    },
  );
