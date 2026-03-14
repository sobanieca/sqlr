import { createTestRunner } from "./test-utils.js";

Deno.test("sqlr main", async (t) => {
  const test = createTestRunner(t);

  await test("sqlr");

  await test(
    `sqlr add-connection -n conn-one -t postgresql -s "postgres://u:p@localhost:5432/db1"`,
  );

  await test(
    `sqlr add-connection -n conn-two -t mysql -s "mysql://u:p@localhost:3306/db2"`,
  );

  await test("sqlr get-connections");

  await test("sqlr clear-connections");

  await test("sqlr get-connections");
});

Deno.test("sqlr connection string encoding", async (t) => {
  const test = createTestRunner(t);

  await test(
    `sqlr add-connection -n enc-mssql -t mssql -s "mssql://sa:World123!%3B@localhost:1433/db?encrypt=true&trustServerCertificate=false"`,
  );

  await test("sqlr get-connection enc-mssql");

  await test(
    `sqlr add-connection -n enc-pg -t postgresql -s "postgres://localhost:5432/db?user=admin&password=p%40ss%3Aw%23rd&application_name=sqlr&sslmode=prefer"`,
  );

  await test("sqlr get-connection enc-pg");

  await test(
    `sqlr add-connection -n enc-mysql -t mysql -s 'mysql://u:p%40ss@localhost:3306/db?ssl={"rejectUnauthorized":false}'`,
  );

  await test("sqlr get-connection enc-mysql");

  await test(
    `sqlr add-connection -n enc-ch -t clickhouse -s "https://user:p%40ss%3Bword@localhost:8443?database=mydb"`,
  );

  await test("sqlr get-connection enc-ch");

  await test("sqlr clear-connections");
});
