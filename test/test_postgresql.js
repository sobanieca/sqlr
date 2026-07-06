import {
  createTestRunner,
  POSTGRES_CONNECTION_STRING,
  POSTGRES_SSL_CONNECTION_STRING,
  startPostgres,
  startPostgresSsl,
  stopPostgres,
  stopPostgresSsl,
} from "./test-utils.js";

const CS = POSTGRES_CONNECTION_STRING;
const SSL_CS = POSTGRES_SSL_CONNECTION_STRING;

Deno.test("sqlr CLI", async (t) => {
  const test = createTestRunner(t);

  await test("sqlr --help");
  await test("sqlr ls-types");
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
      `sqlr query -t postgresql -s "${CS}" test-query.sql`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query-vars.sql -i "col: 1" -i "alias: test"`,
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
      `sqlr describe -t postgresql -s "${CS}" -f "country" --tables-only`,
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query-handlebars.sql`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" test-query-handlebars.sql --ignore-input-validation`,
      "test",
    );

    await test(
      `sqlr query -t postgresql -s "${CS}" "INVALID SQL QUERY"`,
    );
  } finally {
    await stopPostgres();
  }
});

Deno.test("sqlr CLI - PostgreSQL SSL", async (t) => {
  const test = createTestRunner(t);

  await startPostgresSsl();

  try {
    await test(
      `sqlr query -t postgresql -s "${SSL_CS}" "SELECT 1 as test"`,
    );

    await test(
      `sqlr query -t postgresql -s "${SSL_CS}" "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    await test(
      `sqlr describe -t postgresql -s "${SSL_CS}" -f "country"`,
    );
  } finally {
    await stopPostgresSsl();
  }
});
