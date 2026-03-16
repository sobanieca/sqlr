import {
  createTestRunner,
  MYSQL_CONNECTION_STRING,
  startMysql,
  stopMysql,
} from "./test-utils.js";

const CS = MYSQL_CONNECTION_STRING;

Deno.test("sqlr MySQL", async (t) => {
  const test = createTestRunner(t);

  await startMysql();

  try {
    await test(`sqlr query -t mysql -s "${CS}" "SELECT 1 as test"`);

    await test(
      `sqlr query -t mysql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr query -t mysql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    await test(
      `sqlr query -t mysql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    await test(
      `sqlr query -t mysql -s "${CS}" test-query.sql`,
      "test",
    );

    await test(
      `sqlr query -t mysql -s "${CS}" test-query-vars.sql -i "col: 1" -i "alias: test"`,
      "test",
    );

    await test(
      `sqlr query -t mysql -s "${CS}" "SELECT 1 as test" -o /tmp/sqlr-test-output-mysql.json`,
    );

    await test(
      `sqlr describe -t mysql -s "${CS}" -f "country"`,
    );

    await test(
      `sqlr describe -t mysql -s "${CS}" -f "country" --table`,
    );

    await test(
      `sqlr describe -t mysql -s "${CS}" -f "country" --compact`,
    );

    await test(
      `sqlr query -t mysql -s "${CS}" "INVALID SQL QUERY"`,
    );

    await test(
      `sqlr add-connection -n test-mysql -t mysql -s "${CS}"`,
    );

    await test("sqlr get-connections");

    await test(
      `sqlr query -n test-mysql "SELECT 1 as test"`,
    );

    await test("sqlr rm-connection -n test-mysql");
  } finally {
    await stopMysql();
  }
});
