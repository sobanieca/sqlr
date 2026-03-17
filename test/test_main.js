import { assertSnapshot, createTestRunner, run } from "./test-utils.js";

Deno.test("sqlr main", async (t) => {
  const test = createTestRunner(t);

  await test("sqlr");

  await test(
    `sqlr add -n conn-one -t postgresql -s "postgres://u:p@localhost:5432/db1"`,
  );

  await test(
    `sqlr add -n conn-two -t mysql -s "mysql://u:p@localhost:3306/db2"`,
  );

  await test("sqlr ls");

  await test("sqlr clear");

  await test("sqlr ls");
});

Deno.test("sqlr connection string encoding", async (t) => {
  const test = createTestRunner(t);

  await test(
    `sqlr add -n enc-mssql -t mssql -s "mssql://sa:World123!%3B@localhost:1433/db?encrypt=true&trustServerCertificate=false"`,
  );

  await test("sqlr get enc-mssql");

  await test(
    `sqlr add -n enc-pg -t postgresql -s "postgres://localhost:5432/db?user=admin&password=p%40ss%3Aw%23rd&application_name=sqlr&sslmode=prefer"`,
  );

  await test("sqlr get enc-pg");

  await test(
    `sqlr add -n enc-mysql -t mysql -s 'mysql://u:p%40ss@localhost:3306/db?ssl={"rejectUnauthorized":false}'`,
  );

  await test("sqlr get enc-mysql");

  await test(
    `sqlr add -n enc-ch -t clickhouse -s "https://user:p%40ss%3Bword@localhost:8443?database=mydb"`,
  );

  await test("sqlr get enc-ch");

  await test("sqlr clear");
});

const ENCRYPTION_ENV = {
  SQLR_ENCRYPTION_PASSWORD: "test-encryption-password123",
};

Deno.test("sqlr encrypted connection via env var", async (t) => {
  const test = createTestRunner(t);

  await test(
    `sqlr add -n encrypted-pg -t postgresql -s "postgres://u:p@localhost:5432/db1"`,
    undefined,
    ENCRYPTION_ENV,
  );

  await test("sqlr get encrypted-pg", undefined, ENCRYPTION_ENV);

  await test("sqlr clear");
});

Deno.test("sqlr scoped connections", async (t) => {
  const projectRoot = Deno.cwd().replace("/test", "");
  const mainPath = `${projectRoot}/main.js`;
  const sqlr = (cmd, cwd) =>
    run(cmd.replace("sqlr", `deno run -A ${mainPath}`), cwd);

  const tempDir = await Deno.makeTempDir();
  await Deno.mkdir(`${tempDir}/.git`);

  const tempDir2 = await Deno.makeTempDir();
  await Deno.mkdir(`${tempDir2}/.git`);

  try {
    await t.step(
      "connection added in repo scope is visible in that repo",
      async () => {
        await sqlr(
          `sqlr add -n repo-conn -t postgresql -s "postgres://u:p@localhost:5432/db1"`,
          tempDir,
        );
        const result = await sqlr("sqlr ls", tempDir);
        await assertSnapshot(t, {
          step: "repo-scoped connection visible",
          output: result.output,
        });
      },
    );

    await t.step(
      "repo-scoped connection is not visible in another repo",
      async () => {
        const result = await sqlr("sqlr ls", tempDir2);
        await assertSnapshot(t, {
          step: "repo-scoped connection not visible in other repo",
          output: result.output,
        });
      },
    );

    await t.step(
      "repo-scoped connection is not visible in global scope",
      async () => {
        const result = await sqlr("sqlr ls -g", tempDir);
        await assertSnapshot(t, {
          step: "repo-scoped connection not visible globally",
          output: result.output,
        });
      },
    );

    await t.step(
      "global connection added with -g is visible with -g from any repo",
      async () => {
        await sqlr(
          `sqlr add -n global-conn -t mysql -s "mysql://u:p@localhost:3306/db2" -g`,
          tempDir,
        );
        const result1 = await sqlr("sqlr ls -g", tempDir);
        const result2 = await sqlr("sqlr ls -g", tempDir2);
        await assertSnapshot(t, {
          step: "global connection visible from both repos",
          output1: result1.output,
          output2: result2.output,
        });
      },
    );

    await t.step(
      "global connection is not visible in repo scope",
      async () => {
        const result = await sqlr("sqlr ls", tempDir2);
        await assertSnapshot(t, {
          step: "global connection not visible in repo scope",
          output: result.output,
        });
      },
    );

    await t.step("clear only clears current scope", async () => {
      await sqlr("sqlr clear", tempDir);
      const repoResult = await sqlr("sqlr ls", tempDir);
      const globalResult = await sqlr("sqlr ls -g", tempDir);
      await assertSnapshot(t, {
        step: "clear only affects current scope",
        repoOutput: repoResult.output,
        globalOutput: globalResult.output,
      });
    });

    await sqlr("sqlr clear -g", tempDir);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
    await Deno.remove(tempDir2, { recursive: true });
  }
});

Deno.test("sqlr set-global / unset-global", async (t) => {
  const projectRoot = Deno.cwd().replace("/test", "");
  const mainPath = `${projectRoot}/main.js`;
  const sqlr = (cmd, cwd) =>
    run(cmd.replace("sqlr", `deno run -A ${mainPath}`), cwd);

  const tempDir = await Deno.makeTempDir();
  await Deno.mkdir(`${tempDir}/.git`);

  try {
    await sqlr(
      `sqlr add -n global-db -t postgresql -s "postgres://u:p@localhost:5432/db1" -g`,
      tempDir,
    );

    await t.step(
      "global connection not visible in repo scope before set-global",
      async () => {
        const result = await sqlr("sqlr ls", tempDir);
        await assertSnapshot(t, {
          step: "before set-global",
          output: result.output,
        });
      },
    );

    await t.step("set-global makes global connections visible", async () => {
      await sqlr("sqlr set-global", tempDir);
      const result = await sqlr("sqlr ls", tempDir);
      await assertSnapshot(t, {
        step: "after set-global",
        output: result.output,
      });
    });

    await t.step(
      "unset-global restores repo scope",
      async () => {
        await sqlr("sqlr unset-global", tempDir);
        const result = await sqlr("sqlr ls", tempDir);
        await assertSnapshot(t, {
          step: "after unset-global",
          output: result.output,
        });
      },
    );

    await sqlr("sqlr clear -g", tempDir);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("sqlr set", async (t) => {
  const projectRoot = Deno.cwd().replace("/test", "");
  const mainPath = `${projectRoot}/main.js`;
  const sqlr = (cmd, cwd) =>
    run(cmd.replace("sqlr", `deno run -A ${mainPath}`), cwd);

  const tempDir = await Deno.makeTempDir();
  await Deno.mkdir(`${tempDir}/.git`);

  const tempDir2 = await Deno.makeTempDir();
  await Deno.mkdir(`${tempDir2}/.git`);

  try {
    await sqlr(
      `sqlr add -n repo-db -t postgresql -s "postgres://u:p@localhost:5432/db1"`,
      tempDir,
    );

    await t.step("set sets default", async () => {
      const setResult = await sqlr(
        "sqlr set -n repo-db",
        tempDir,
      );
      await assertSnapshot(t, {
        step: "set output",
        output: setResult.output,
      });
    });

    await t.step(
      "get uses default without -n",
      async () => {
        const result = await sqlr("sqlr get", tempDir);
        await assertSnapshot(t, {
          step: "get uses default",
          output: result.output,
        });
      },
    );

    await t.step(
      "default connection is scoped to repo",
      async () => {
        await sqlr(
          `sqlr add -n other-db -t mysql -s "mysql://u:p@localhost:3306/db2"`,
          tempDir2,
        );
        await sqlr("sqlr set -n other-db", tempDir2);
        const result = await sqlr("sqlr get", tempDir2);
        await assertSnapshot(t, {
          step: "default in other repo is other-db",
          output: result.output,
        });
      },
    );

    await t.step(
      "set with -g enables global mode",
      async () => {
        await sqlr(
          `sqlr add -n global-db -t postgresql -s "postgres://u:p@localhost:5432/db1" -g`,
          tempDir,
        );
        await sqlr("sqlr set -n global-db -g", tempDir);
        const result = await sqlr("sqlr get", tempDir2);
        await assertSnapshot(t, {
          step: "global default used from another repo",
          output: result.output,
        });
      },
    );

    await sqlr("sqlr unset-global", tempDir);
    await sqlr("sqlr clear", tempDir);
    await sqlr("sqlr clear", tempDir2);
    await sqlr("sqlr clear -g", tempDir);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
    await Deno.remove(tempDir2, { recursive: true });
  }
});
