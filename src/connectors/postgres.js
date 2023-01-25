const postgresConnector = {
  describe: async (connectionString) => {
    await Promise.resolve();
    console.log("Postgres connector running describe command with conn string:");
    console.log(connectionString); 
  }
}

export { postgresConnector }
