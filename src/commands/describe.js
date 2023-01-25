import { Command } from "cliffy/command/mod.ts";
//import connectionTypes from "../connection-types.js";

const _describe = (_connection, _connectionString, _compact) => {

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
  .description("Describe all tables and columns available in database")
  .option("-s, --connection-string [value:string]", "Connection string", {
    conflicts: ["connection"],
  })
  .option("-c, --connection [value:string]", "Name of the connection", {
    conflicts: ["connection-string"],
  })
  .option("--compact [flag:boolean]", "Show every table in one line")
  .action(({ connection, connectionString, compact }) => {
    console.log("Connection: " + connection);
    console.log("Connection string: " + connectionString);
    console.log("Compact: " + compact);
  });
