import { createTestRunner } from "./test-utils.js";

Deno.test("sqlr main", async (t) => {
  const test = createTestRunner(t);

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
