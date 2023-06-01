import { Command } from "../deps.js";
import connectionTypes from "../connection-types.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";
import { brightBlue, brightGreen, brightRed, brightYellow, gray } from "../deps.js";

const describe = async (connectionName, json, compact) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }

  const connection = await getConnection(connectionName);

  try {
    const tables = await connectionTypes[connection.type].getTables(connection.connectionString);

    if (tables)
      tables?.sort((a,b) => a.schema.localeCompare(b.schema));

    showTables(tables, json, compact);
  } catch(err) {
    logger.debug(err);
    logger.error(`Error occurred when executing query against database. Ensure connection string is valid.`);
  }
}

const showTables = (tables, json, compact) => {
  logger.info("Database schema:");
  if (compact) {
    showTablesCompact(tables);
    return;
  }

  if (json) {
    console.log("json output goes here...");
    showTablesJson(tables);
    return;
  }

  console.log("default output goes here..");
}

const showTablesJson = (tables) => {
  tables.forEach(t => t.columns.forEach(c => c.relation || (delete c.relation)));
  logger.info(Deno.inspect(
    tables,
    {
      colors: true,
      strAbbreviateSize: 256000,
      iterableLimit: 20000,
      depth: 100,
    },
  ));
}

const showTablesCompact = (tables) => {
  for (const table of tables) {
    const columns = table.columns.reduce((acc, _, index) => {
      const column = table.columns[index];
      if (column.relation) {
        acc += `${brightYellow(column.name)} -> "${brightBlue(column.relation)}"`
      } else {
        acc += `${column.name}${column.nullable ? brightRed('?') : '' } ${gray(column.type)}`;
      }
      if (index != table.columns.length - 1) {
        acc += ', ';
      }

      return acc;
    }, '');
    const tableLine = `${brightGreen(table.schema)}.${brightGreen(table.name)} [ ${columns} ]`;
    logger.info(tableLine);
  } 
}

export default new Command()
  .arguments("[connection]")
  .option("--json", "Display results as JSON")
  .option("--compact", "Display results in compact form")
  .description("Describe all tables and columns available in database")
  .action(async ({json, compact}, connection) => {
    await describe(connection, json, compact);
  });
