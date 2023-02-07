import { Command } from "../deps.js";
import connectionTypes from "../connection-types.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";
import logger from "../logger.js";

const describe = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }

  const connection = await getConnection(connectionName);

  const description = await connectionTypes[connection.type].connector.getDescription(connection.connectionString);

  if (description)
    description?.tables.sort((a,b) => a.schema.localeCompare(b.schema));
  showDescription(description);
}

const showDescription = (description) => {
  logger.info("Database schema:");
  for (const table of description.tables) {
    const columns = table.columns.reduce((acc, _, index) => {
      const column = table.columns[index];
      if(column.relation) {
        acc += `${column.name} fk("${column.relation}")`
      } else {
        acc += `${column.name}${column.nullable ? '?' : '' } ${column.type}`;
      }
      if(index != table.columns.length - 1) {
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
