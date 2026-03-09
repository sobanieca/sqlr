import {
  createTestRunner,
  MSSQL_CONNECTION_STRING,
  startMssql,
  stopMssql,
} from "./test-utils.js";

const CS = MSSQL_CONNECTION_STRING;

Deno.test("sqlr MSSQL", async (t) => {
  const test = createTestRunner(t);

  await startMssql();

  try {
    await test(`sqlr query -t mssql -s "${CS}" -q "SELECT 1 as test"`);

    await test(
      `sqlr query -t mssql -s "${CS}" -q "SELECT TOP 3 code, name FROM country ORDER BY code"`,
    );

    await test(
      `sqlr query -t mssql -s "${CS}" -q "SELECT TOP 3 code, name FROM country ORDER BY code" --table`,
    );

    await test(
      `sqlr query -t mssql -s "${CS}" -q "SELECT TOP 3 code, name FROM country ORDER BY code" --compact`,
    );

    await test(
      `sqlr query -t mssql -s "${CS}" -i test-query.sql`,
      "test",
    );

    await test(
      `sqlr query -t mssql -s "${CS}" -q "SELECT 1 as test" -o /tmp/sqlr-test-output-mssql.json`,
    );

    await test(
      `sqlr describe -t mssql -s "${CS}" -f "country"`,
    );

    await test(
      `sqlr describe -t mssql -s "${CS}" -f "country" --table`,
    );

    await test(
      `sqlr describe -t mssql -s "${CS}" -f "country" --compact`,
    );

    await test(
      `sqlr query -t mssql -s "${CS}" -q "INVALID SQL QUERY"`,
    );
  } finally {
    await stopMssql();
  }
});
