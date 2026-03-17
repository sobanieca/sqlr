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
    await test(`sqlr query -t postgresql -s "${CS}" "SELECT 1 as test"`);

    await test(
      `sqlr query -t postgresql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query.sql -y`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query-vars.sql -i "col: 1" -i "alias: test" -y`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" "SELECT 1 as test" -o /tmp/sqlr-test-output.json`,
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
      `sqlr query -t postgresql -s "${CS}" test-query-handlebars.sql -y`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query-handlebars.sql --ignore-input-validation -y`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" "INVALID SQL QUERY"`,
    );

    await test(
      `sqlr add-connection -n test-pg -t postgresql -s "${CS}"`,
    );

    await test("sqlr get-connections");

    await test(
      `sqlr query -n test-pg "SELECT 1 as test"`,
    );

    await test("sqlr rm-connection -n test-pg");
  } finally {
    await stopPostgres();
  }
});
