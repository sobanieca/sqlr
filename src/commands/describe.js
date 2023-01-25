import { Command } from "cliffy/command/mod.ts";
import connectionTypes from "../connection-types.js";
import { getConnection, getConnectionName } from "../connection-accessor.js";

const describe = async (connectionName) => {
  if (!connectionName) {
    connectionName = await getConnectionName();
  }

  const connection = await getConnection(connectionName);

  await connectionTypes[connection.type].connector.describe(connection.connectionString)
}

//TODO: implement
// read connection type from connection (this may block possibility to use 
// connection string, or at least demand providing type together)
// basing on connection type take proper db connector by reading connectionTypes
// and call it's describe() method, passing connection string there
// with result returned in proper format:
// { tables: [ { name: users, columns: [ { name: id, type: int, nullable: true } ] } ] }
//
//
// Pass it to function responsible for displaying data, respect compact flag, so it gets
// displayed in single line if requested
  export default new Command()
  .arguments("[connection]")
  .description("Describe all tables and columns available in database")
  .action(async (_, connection) => {
    await describe(connection);
  });
