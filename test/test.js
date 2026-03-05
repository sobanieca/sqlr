import {
  CONNECTION_STRING,
  createTestRunner,
  startPostgres,
  stopPostgres,
} from "./test-utils.js";

const CS = CONNECTION_STRING;

Deno.test("sqlr CLI", async (t) => {
  const test = createTestRunner(t);

  await test("sqlr --help");
  await test("sqlr get-connection-types");
  await test("sqlr query --help");
  await test("sqlr describe --help");

  await startPostgres();

  try {
    await test(`sqlr query -t postgresql -s "${CS}" -q "SELECT 1 as test"`);

    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" -i test-query.sql`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT 1 as test" -o /tmp/sqlr-test-output.json`,
    );

    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country"`,
    );

    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country" --table`,
    );

    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country" --compact`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" -q "INVALID SQL QUERY"`,
    );
  } finally {
    await stopPostgres();
  }
});
