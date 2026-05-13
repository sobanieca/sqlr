import {
  createTestRunner,
  SQLITE_CONNECTION_STRING,
  startSqlite,
  stopSqlite,
} from "./test-utils.js";

const CS = SQLITE_CONNECTION_STRING;

Deno.test("sqlr SQLite", async (t) => {
  const test = createTestRunner(t);

  await startSqlite();

  try {
    await test(`sqlr query -t sqlite -s "${CS}" "SELECT 1 as test"`);

    await test(
      `sqlr query -t sqlite -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" test-query.sql -y`,
      "test",
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" test-query-vars.sql -i "col: 1" -i "alias: test" -y`,
      "test",
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" "SELECT 1 as test" -o /tmp/sqlr-test-output-sqlite.json`,
    );

    await test(
      `sqlr describe -t sqlite -s "${CS}" -f "country"`,
    );

    await test(
      `sqlr describe -t sqlite -s "${CS}" -f "country" --table`,
    );

    await test(
      `sqlr describe -t sqlite -s "${CS}" -f "country" --compact`,
    );

    await test(
      `sqlr describe -t sqlite -s "${CS}" -f "country" --tables-only`,
    );

    await test(
      `sqlr query -t sqlite -s "${CS}" "INVALID SQL QUERY"`,
    );
  } finally {
    await stopSqlite();
  }
});
