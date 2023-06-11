import { Command, EnumType } from "../deps.js";
import { connectors } from "../connectors.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";

const query = async (
  query,
  inputFile,
  outputFile,
  connectionName,
  json,
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

  try {
    //TODO: implement query method for connectors
    const result = await connectors[targetType].query(query);

    //TODO: present results + affected rows (if > 0)
  } catch (err) {
    logger.debug(err);
    logger.error(
      `Error occurred when executing query against database. Ensure connection string is valid.`,
    );
  }
};

export default new Command()
  .type("ConnectorType", new EnumType(Object.keys(connectors)))
  .option("-q, --query [query]", "SQL query to be executed")
  .option("-i, --input-file [input-file]", "Input file for query", {
    conflicts: ["query"],
  })
  .option(
    "-o, --output-file [output-file]",
    "Output file where JSON results will be stored",
  )
  .option("-n, --name [name]", "Name of the connection", {
    conflicts: ["type", "connection-string"],
  })
  .option("-t, --type [type:ConnectorType]", "Type of the connection")
  .option("-s, --connection-string [connection-string]", "Connection string")
  .option("--json", "Display results as JSON")
  .option("--compact", "Display results in compact form")
  .description("Run provided SQL query against selected database")
  .description("Run query against specified database")
  .action(
    async (
      {
        query,
        inputFile,
        outputFile,
        name,
        json,
        compact,
        type,
        connectionString,
      },
    ) => {
      await query(
        query,
        inputFile,
        outputFile,
        name,
        json,
        compact,
        type,
        connectionString,
      );
    },
  );
