import { connectors } from "../connectors.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { maxTableColumnWidth } from "../const.js";
import {
  brightBlue,
  brightGreen,
  brightRed,
  brightYellow,
  Command,
  EnumType,
  gray,
  Table,
} from "../deps.js";

const describe = async (
  connectionName,
  table,
  compact,
  type,
  filter,
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
    let tables = await connectors[targetType].getTables(
      targetConnectionString,
    );

    if (tables) tables?.sort((a, b) => a.schema.localeCompare(b.schema));
    if (filter) {
      tables = tables.filter((table) =>
        `${table.schema}.${table.name}`.includes(filter)
      );
    }

    showTables(tables, table, compact);
  } catch (err) {
    logger.debug(err);
    logger.error(
      `Error occurred when executing query against database. Ensure connection string is valid. Use --debug option for details.`,
    );
  }
};

const showTables = (tables, table, compact) => {
  logger.info("Database schema:");
  if (compact) {
    showTablesCompact(tables);
    return;
  }

  if (table) {
    showTablesTable(tables);
    return;
  }

  tables.forEach((t) =>
    t.columns.forEach((c) => c.relation || delete c.relation)
  );
  logger.info(
    Deno.inspect(tables, {
      colors: true,
      strAbbreviateSize: 256000,
      iterableLimit: 20000,
      depth: 100,
    }),
  );
};

const showTablesTable = (tables) => {
  for (const table of tables) {
    logger.info(
      new Table()
        .header([getTableName(table)])
        .body(
          table.columns.map((column) => [getColumnDescription(column)]),
        )
        .maxColWidth(maxTableColumnWidth)
        .padding(1)
        .indent(2)
        .border(true)
        .toString(),
    );
  }
};

const showTablesCompact = (tables) => {
  for (const table of tables) {
    const columns = table.columns.reduce((acc, _, index) => {
      const column = table.columns[index];
      acc += getColumnDescription(column);
      if (index != table.columns.length - 1) {
        acc += ", ";
      }

      return acc;
    }, "");
    const tableLine = `${getTableName(table)} [ ${columns} ]`;
    logger.info(tableLine);
  }
};

const getTableName = (table) =>
  `${brightGreen(table.schema)}.${brightGreen(table.name)}`;

const getColumnDescription = (column) => {
  if (column.relation) {
    return `${brightYellow(column.name)} -> "${brightBlue(column.relation)}"`;
  } else {
    return `${column.name}${column.nullable ? brightRed("?") : ""} ${
      gray(
        column.type,
      )
    }`;
  }
};

export default new Command()
  .type("ConnectorType", new EnumType(Object.keys(connectors)))
  .option("-n, --name [name]", "Name of the connection", {
    conflicts: ["type", "connection-string"],
  })
  .option("-t, --type [type:ConnectorType]", "Type of the connection")
  .option("-s, --connection-string [connection-string]", "Connection string")
  .option("-f, --filter [filter]", "Filter results by schema/table name")
  .option("--table", "Display results in table")
  .option("--compact", "Display results in compact form")
  .description("Describe all tables and columns available in database")
  .action(async ({ name, table, compact, type, filter, connectionString }) => {
    await describe(name, table, compact, type, filter, connectionString);
  });
