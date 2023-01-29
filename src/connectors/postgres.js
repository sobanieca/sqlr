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
              type: "bigint",
              nullable: false,
            },
            {
              name: "user_id",
              relation: "dbo.users.id"
            },
            {
              name: "created_on",
              type: "datetime",
              nullable: true
            }
          ]
        }
      ]
    }
  }
}

export { postgresConnector }
