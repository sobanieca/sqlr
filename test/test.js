import {
  CONNECTION_STRING,
  createTestRunner,
  startPostgres,
  stopPostgres,
} from "./test-utils.js";

const CS = CONNECTION_STRING;

Deno.test("sqlr CLI", async (t) => {
  const test = createTestRunner(t);

  // Commands that don't require a database
  await test("sqlr --help");
  await test("sqlr --version");
  await test("sqlr get-connection-types");
  await test("sqlr query --help");
  await test("sqlr describe --help");

  // Start postgres for database tests
  await startPostgres();

  try {
    // Basic query
    await test(`sqlr query -t postgresql -s "${CS}" -q "SELECT 1 as test"`);

    // Query with results
    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3"`,
    );

    // Table format
    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3" --table`,
    );

    // Compact format
    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT code, name FROM country ORDER BY code LIMIT 3" --compact`,
    );

    // Input file
    await test(
      `sqlr query -t postgresql -s "${CS}" -i test-query.sql`,
      "test",
    );

    // Output to file
    await test(
      `sqlr query -t postgresql -s "${CS}" -q "SELECT 1 as test" -o /tmp/sqlr-test-output.json`,
    );

    // Describe with filter
    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country"`,
    );

    // Describe with table format and filter
    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country" --table`,
    );

    // Describe with compact format and filter
    await test(
      `sqlr describe -t postgresql -s "${CS}" -f "country" --compact`,
    );

    // Error case: invalid query
    await test(
      `sqlr query -t postgresql -s "${CS}" -q "INVALID SQL QUERY"`,
    );
  } finally {
    await stopPostgres();
  }
});
