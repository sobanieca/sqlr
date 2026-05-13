import {
  CLICKHOUSE_CONNECTION_STRING,
  createTestRunner,
  startClickhouse,
  stopClickhouse,
} from "./test-utils.js";

const CS = CLICKHOUSE_CONNECTION_STRING;

Deno.test("sqlr ClickHouse", async (t) => {
  const test = createTestRunner(t);

  await startClickhouse();

  try {
    await test(`sqlr query -t clickhouse -s "${CS}" "SELECT 1 as test"`);

    await test(
      `sqlr query -t clickhouse -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" test-query.sql -y`,
      "test",
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" test-query-vars.sql -i "col: 1" -i "alias: test" -y`,
      "test",
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" "SELECT 1 as test" -o /tmp/sqlr-test-output-clickhouse.json`,
    );

    await test(
      `sqlr describe -t clickhouse -s "${CS}" -f "country"`,
    );

    await test(
      `sqlr describe -t clickhouse -s "${CS}" -f "country" --table`,
    );

    await test(
      `sqlr describe -t clickhouse -s "${CS}" -f "country" --compact`,
    );

    await test(
      `sqlr describe -t clickhouse -s "${CS}" -f "country" --tables-only`,
    );

    await test(
      `sqlr query -t clickhouse -s "${CS}" "INVALID SQL QUERY"`,
    );
  } finally {
    await stopClickhouse();
  }
});
