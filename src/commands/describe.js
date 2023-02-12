import { Command } from "../deps.js";
import connectionTypes from "../connection-types.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";

// TODO: introduce option to specify resulting format (one line, tables to visually represent data)
const describe = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }

  const connection = await getConnection(connectionName);

  try {
    const tables = await connectionTypes[connection.type].getTables(connection.connectionString);

    if (tables)
      tables?.sort((a,b) => a.schema.localeCompare(b.schema));

    showTables(tables);
  } catch(err) {
    logger.debug(err);
    logger.error(`Error occurred when executing query against database. Ensure connection string is valid.`);
  }
}

const showTables = (tables) => {
  logger.info("Database schema:");
  for (const table of tables) {
    const columns = table.columns.reduce((acc, _, index) => {
      const column = table.columns[index];
      if (column.relation) {
        acc += `${column.name} fk("${column.relation}")`
      } else {
        acc += `${column.name}${column.nullable ? '?' : '' } ${column.type}`;
      }
      if (index != table.columns.length - 1) {
        acc += ', ';
      }

      return acc;
    }, '');
    const tableLine = `${table.schema}.${table.name} [${columns}]`;
    logger.info(tableLine);
  }
}

export default new Command()
  .arguments("[connection]")
  .description("Describe all tables and columns available in database")
  .action(async (_, connection) => {
    await describe(connection);
  });
