const postgresConnector = {
  getDescription: async (connectionString) => {
    await Promise.resolve();
    console.log("Postgres connector running describe command with conn string:");
    console.log(connectionString);

    return {
      tables: [
        {
          schema: "dbo",
          name: "orders",
          columns: [
            {
              name: "id",
              type: "bigint"
            },
            {
              name: "user_id",
              type: "foreignKey",
              target: "dbo.users.id"
            }
          ]
        }
      ]
    }
  }
}

export { postgresConnector }
